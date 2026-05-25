use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

use sqlx::SqlitePool;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::models::download::DownloadTask;
use crate::modules::download::service;

pub async fn run_worker(pool: SqlitePool, state: Arc<service::DownloadState>) {
    loop {
        let task = sqlx::query_as::<_, DownloadTask>(
            "SELECT * FROM download_tasks WHERE status = 'queued' ORDER BY created_at ASC LIMIT 1",
        )
        .fetch_optional(&pool)
        .await;

        match task {
            Ok(Some(task)) => {
                download_one(&pool, &state, task).await;
            }
            _ => {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

async fn download_one(pool: &SqlitePool, state: &service::DownloadState, task: DownloadTask) {
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

    let result = execute_download(pool, &task, &cancel_flag).await;

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

async fn execute_download(
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

    // Determine file name from URL
    let file_name = Path::new(url)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download");
    let file_path = save_dir.join(file_name);

    // Check for partial download (for resume)
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
        // For resumed downloads, parse Content-Range for total size
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

    // Update file name in DB if we got a Content-Disposition header
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

        // Update progress every 500ms
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
            let _ = service::update_progress(
                pool,
                task.id,
                progress,
                speed,
                total_size as i64,
                eta,
            )
            .await;
            last_update = Instant::now();
        }
    }

    // Final progress update
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
