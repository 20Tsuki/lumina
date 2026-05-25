use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

use sqlx::SqlitePool;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::models::download::DownloadTask;
use crate::modules::download::aria2::Aria2Manager;
use crate::modules::download::service;

pub async fn run_worker(pool: SqlitePool, state: Arc<service::DownloadState>) {
    // Spawn aria2 progress sync if aria2 is available
    if state.aria2.is_available() {
        let sync_pool = pool.clone();
        let sync_state = state.clone();
        tokio::spawn(async move {
            aria2_progress_sync(sync_pool, sync_state).await;
        });
    }

    loop {
        let task = sqlx::query_as::<_, DownloadTask>(
            "SELECT * FROM download_tasks WHERE status = 'queued' ORDER BY created_at ASC LIMIT 1",
        )
        .fetch_optional(&pool)
        .await;

        match task {
            Ok(Some(task)) => {
                if state.aria2.is_available() {
                    download_via_aria2(&pool, &state, task).await;
                } else if Aria2Manager::is_magnet_or_torrent(&task.url) {
                    let _ = service::fail_task(&pool, task.id, "aria2 not available — install aria2c for BT/magnet support").await;
                } else {
                    download_via_http(&pool, &state, task).await;
                }
            }
            _ => {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

async fn download_via_aria2(
    pool: &SqlitePool,
    state: &service::DownloadState,
    task: DownloadTask,
) {
    let now = chrono::Utc::now().timestamp_millis();
    let _ = sqlx::query(
        "UPDATE download_tasks SET status = 'downloading', updated_at = ? WHERE id = ?",
    )
    .bind(now)
    .bind(task.id)
    .execute(pool)
    .await;

    match state.aria2.add_uri(&task.url, &task.save_path).await {
        Ok(gid) => {
            state.aria2.gid_map.lock().await.insert(task.id, gid);
            tracing::info!(task_id = task.id, "aria2 download started");
        }
        Err(e) => {
            tracing::error!(task_id = task.id, %e, "aria2 add_uri failed");
            let _ = service::fail_task(pool, task.id, &e).await;
        }
    }
}

/// Background loop: poll aria2 statuses and sync to DB
async fn aria2_progress_sync(pool: SqlitePool, state: Arc<service::DownloadState>) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let gid_map = state.aria2.gid_map.lock().await.clone();
        if gid_map.is_empty() {
            continue;
        }

        // Poll active, waiting, stopped
        let active = state.aria2.tell_active().await.unwrap_or_default();
        let waiting = state.aria2.tell_waiting().await.unwrap_or_default();

        let all: Vec<_> = active.into_iter().chain(waiting).collect();

        for (task_id, gid) in &gid_map {
            if let Some(status) = all.iter().find(|s| &s.gid == gid) {
                let s = status;
                let progress = s.progress();
                let speed = s.speed_bytes();
                let size = s.total_bytes();
                let eta = s.eta();
                let mapped = s.mapped_status();

                // Update file_name from aria2
                if let Some(ref name) = s.file_name() {
                    let now = chrono::Utc::now().timestamp_millis();
                    let _ = sqlx::query(
                        "UPDATE download_tasks SET file_name = ?, updated_at = ? WHERE id = ? AND file_name IS NULL",
                    )
                    .bind(name)
                    .bind(now)
                    .bind(task_id)
                    .execute(&pool)
                    .await;
                }

                // Update progress
                let _ = service::update_progress(&pool, *task_id, progress, speed, size, eta).await;

                // Handle terminal states
                if mapped == "completed" || mapped == "failed" {
                    if mapped == "completed" {
                        let _ = service::complete_task(&pool, *task_id).await;
                    } else if let Some(ref err) = s.error_message {
                        let _ = service::fail_task(&pool, *task_id, err).await;
                    }
                    // Keep GID for a bit, then clean up
                }

                // Handle BT metadata phase: if waiting and has followedBy, update GID
                if s.status == "waiting" && s.followed_by.as_ref().map_or(false, |v| !v.is_empty()) {
                    if let Some(ref followed) = s.followed_by {
                        if let Some(new_gid) = followed.first() {
                            state.aria2.gid_map.lock().await.insert(*task_id, new_gid.clone());
                        }
                    }
                }
            }
        }
    }
}

async fn download_via_http(
    pool: &SqlitePool,
    state: &service::DownloadState,
    task: DownloadTask,
) {
    let task_id = task.id;
    let cancel_flag = state.register_task(task_id).await;

    let now = chrono::Utc::now().timestamp_millis();
    let _ = sqlx::query(
        "UPDATE download_tasks SET status = 'downloading', updated_at = ? WHERE id = ?",
    )
    .bind(now)
    .bind(task_id)
    .execute(pool)
    .await;

    let result = execute_http(pool, &task, &cancel_flag).await;

    state.unregister_task(task_id).await;

    match result {
        Ok(()) => {
            let _ = service::complete_task(pool, task_id).await;
            tracing::info!(task_id, "download completed");
        }
        Err(e) => {
            if cancel_flag.load(Ordering::SeqCst) {
                tracing::info!(task_id, "download paused by user");
            } else {
                let _ = service::fail_task(pool, task_id, &e).await;
                tracing::error!(task_id, %e, "download failed");
            }
        }
    }
}

async fn execute_http(
    pool: &SqlitePool,
    task: &DownloadTask,
    cancel_flag: &Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), String> {
    let url = &task.url;
    let save_dir = Path::new(&task.save_path);

    fs::create_dir_all(save_dir)
        .await
        .map_err(|e| format!("create dir: {}", e))?;

    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| format!("client: {}", e))?;

    let file_name = Path::new(url)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download");
    let file_path = save_dir.join(file_name);

    let mut downloaded: u64 = 0;
    if file_path.exists() {
        if let Ok(meta) = fs::metadata(&file_path).await {
            downloaded = meta.len();
        }
    }

    let mut request = client.get(url);
    if downloaded > 0 {
        request = request.header("Range", format!("bytes={}-", downloaded));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("request: {}", e))?;

    let status = response.status();
    if !status.is_success() && status.as_u16() != 206 {
        return Err(format!("HTTP {}", status));
    }

    let total_size = if downloaded > 0 {
        response
            .headers()
            .get("Content-Range")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split('/').last())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    } else {
        response.content_length().unwrap_or(0)
    };

    if let Some(cd) = response
        .headers()
        .get("Content-Disposition")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(name) = parse_content_disposition(cd) {
            let now = chrono::Utc::now().timestamp_millis();
            let _ = sqlx::query(
                "UPDATE download_tasks SET file_name = ?, updated_at = ? WHERE id = ?",
            )
            .bind(&name)
            .bind(now)
            .bind(task.id)
            .execute(pool)
            .await;
        }
    }

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .await
        .map_err(|e| format!("open file: {}", e))?;

    let mut stream = response.bytes_stream();
    let mut last_update = Instant::now();
    let start = Instant::now();

    while let Some(chunk) = futures::StreamExt::next(&mut stream).await {
        if cancel_flag.load(Ordering::SeqCst) {
            return Err("cancelled".into());
        }

        let chunk = chunk.map_err(|e| format!("chunk: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("write: {}", e))?;
        downloaded += chunk.len() as u64;

        if last_update.elapsed().as_millis() >= 500 {
            let elapsed = start.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                ((downloaded as f64) / elapsed) as i64
            } else {
                0
            };
            let progress = if total_size > 0 {
                (downloaded as f64 / total_size as f64) * 100.0
            } else {
                0.0
            };
            let eta = if speed > 0 && total_size > 0 {
                ((total_size - downloaded) as f64 / speed as f64) as i64
            } else {
                0
            };
            let _ = service::update_progress(pool, task.id, progress, speed, total_size as i64, eta).await;
            last_update = Instant::now();
        }
    }

    let _ = service::update_progress(pool, task.id, 100.0, 0, total_size as i64, 0).await;
    Ok(())
}

fn parse_content_disposition(header: &str) -> Option<String> {
    for part in header.split(';') {
        let part = part.trim();
        if let Some(name) = part.strip_prefix("filename=") {
            return Some(name.trim_matches('"').to_string());
        }
        if let Some(name) = part.strip_prefix("filename*=") {
            if let Some(idx) = name.find("''") {
                return Some(name[idx + 2..].to_string());
            }
        }
    }
    None
}
