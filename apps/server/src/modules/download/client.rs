use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

use sqlx::SqlitePool;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::models::download::DownloadTask;
use crate::modules::download::bittorrent::BtManager;
use librqbit::TorrentStatsState;
use crate::modules::download::service;

pub async fn run_worker(pool: SqlitePool, state: Arc<service::DownloadState>) {
    // Spawn BT progress sync loop (always available since librqbit is built-in)
    let sync_pool = pool.clone();
    let sync_state = state.clone();
    tokio::spawn(async move {
        bt_progress_sync(sync_pool, sync_state).await;
    });

    loop {
        let task = sqlx::query_as::<_, DownloadTask>(
            "SELECT * FROM download_tasks WHERE status = 'queued' ORDER BY created_at ASC LIMIT 1",
        )
        .fetch_optional(&pool)
        .await;

        match task {
            Ok(Some(task)) => {
                if BtManager::is_magnet_or_torrent(&task.url) {
                    download_via_bt(&pool, &state, task).await;
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

async fn download_via_bt(
    pool: &SqlitePool,
    state: &Arc<service::DownloadState>,
    task: DownloadTask,
) {
    // Mark as "connecting" while resolving magnet metadata via DHT
    let now = chrono::Utc::now().timestamp_millis();
    let _ = sqlx::query(
        "UPDATE download_tasks SET status = 'connecting', updated_at = ? WHERE id = ?",
    )
    .bind(now)
    .bind(task.id)
    .execute(pool)
    .await;

    // Spawn metadata resolution in background — resolve_magnet() can take minutes.
    // Append well-known trackers to magnet links so metadata resolution works even
    // when DHT is unreachable (common on macOS due to IPv6-mapped IPv4 issues).
    let bt = state.bt.clone();
    let bt_state = state.clone();
    let task_id = task.id;
    let mut url = task.url.clone();
    let save_path = task.save_path.clone();
    let bg_pool = pool.clone();

    if url.starts_with("magnet:") && !url.contains("&tr=") {
        for tr in &[
            "udp://tracker.opentrackr.org:1337/announce",
            "udp://tracker.openbittorrent.com:6969/announce",
            "udp://tracker.torrent.eu.org:451/announce",
            "http://tracker.opentrackr.org:1337/announce",
        ] {
            url.push_str("&tr=");
            url.push_str(&urlencoding::encode(tr));
        }
    }

    tokio::spawn(async move {
        tracing::debug!(task_id, url = %url, "BT add_torrent: resolving magnet metadata (may take a while)...");
        match tokio::time::timeout(
            std::time::Duration::from_secs(120),
            bt.add_torrent(&url, &save_path),
        )
        .await
        {
            Ok(Ok(torrent_id)) => {
                bt_state.bt.torrent_map.lock().await.insert(task_id, torrent_id);
                let now = chrono::Utc::now().timestamp_millis();
                let _ = sqlx::query(
                    "UPDATE download_tasks SET status = 'downloading', updated_at = ? WHERE id = ?",
                )
                .bind(now)
                .bind(task_id)
                .execute(&bg_pool)
                .await;
                tracing::info!(task_id, torrent_id, "BT metadata resolved, downloading");
            }
            Ok(Err(e)) => {
                tracing::error!(task_id, %e, "BT add_torrent failed");
                let _ = service::fail_task(&bg_pool, task_id, &e).await;
            }
            Err(_) => {
                tracing::error!(task_id, "BT add_torrent timed out after 120s");
                let _ = service::fail_task(&bg_pool, task_id, "metadata resolution timed out — no peers found for this magnet link").await;
            }
        }
    });
}

/// Background loop: poll librqbit torrent stats and sync to DB
async fn bt_progress_sync(pool: SqlitePool, state: Arc<service::DownloadState>) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let torrent_map = state.bt.torrent_map.lock().await.clone();
        if torrent_map.is_empty() {
            continue;
        }

        // Build reverse map: torrent_id → task_id
        let reverse: HashMap<usize, i64> =
            torrent_map.iter().map(|(task_id, torrent_id)| (*torrent_id, *task_id)).collect();

        let all = state.bt.get_all_torrents();

        for (torrent_id, stats, name) in &all {
            if let Some(task_id) = reverse.get(torrent_id) {
                let total = stats.total_bytes;
                let downloaded = stats.progress_bytes;
                let state_label = stats.state.to_string();

                tracing::debug!(
                    task_id,
                    torrent_id,
                    state = %state_label,
                    total,
                    downloaded,
                    finished = stats.finished,
                    error = ?stats.error,
                    name = ?name,
                    "BT torrent status"
                );

                let progress = if total > 0 {
                    (downloaded as f64 / total as f64) * 100.0
                } else {
                    0.0
                };

                let speed: i64 = stats
                    .live
                    .as_ref()
                    .map(|l| (l.download_speed.mbps * 1_048_576.0) as i64)
                    .unwrap_or(0);
                let eta: i64 = if speed > 0 && total > downloaded {
                    ((total - downloaded) / speed as u64) as i64
                } else {
                    0
                };

                // Update file_name from torrent metadata
                if let Some(ref name) = name {
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

                // Reflect actual BT state in DB status
                let mapped_status = match stats.state {
                    TorrentStatsState::Initializing => "connecting",
                    TorrentStatsState::Live => "downloading",
                    TorrentStatsState::Paused => "paused",
                    TorrentStatsState::Error => "failed",
                };
                let now = chrono::Utc::now().timestamp_millis();
                let _ = sqlx::query(
                    "UPDATE download_tasks SET status = ?, updated_at = ? WHERE id = ? AND status != 'completed' AND status != 'failed'",
                )
                .bind(mapped_status)
                .bind(now)
                .bind(task_id)
                .execute(&pool)
                .await;

                let _ = service::update_progress(
                    &pool,
                    *task_id,
                    progress,
                    speed,
                    total as i64,
                    eta,
                )
                .await;

                // Handle terminal states
                if stats.finished {
                    let _ = service::complete_task(&pool, *task_id).await;
                    let _ = state.bt.remove(*torrent_id).await;
                    state.bt.torrent_map.lock().await.remove(task_id);
                } else if matches!(stats.state, TorrentStatsState::Error) {
                    let err = stats.error.clone().unwrap_or_else(|| "unknown error".to_string());
                    let _ = service::fail_task(&pool, *task_id, &err).await;
                    let _ = state.bt.remove(*torrent_id).await;
                    state.bt.torrent_map.lock().await.remove(task_id);
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
