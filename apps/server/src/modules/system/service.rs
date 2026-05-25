use sqlx::SqlitePool;
use sysinfo::System;

use crate::error::AppError;

pub fn get_system_info() -> serde_json::Value {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu = sys
        .cpus()
        .first()
        .map(|c| c.cpu_usage() as f64)
        .unwrap_or(0.0);
    let mem_total = sys.total_memory();
    let mem_used = sys.used_memory();

    let (disk_total, disk_used) = {
        #[cfg(not(target_os = "linux"))]
        {
            let disks = sysinfo::Disks::new_with_refreshed_list();
            if let Some(disk) = disks.first() {
                (disk.total_space(), disk.total_space() - disk.available_space())
            } else {
                (0, 0)
            }
        }
        #[cfg(target_os = "linux")]
        (0, 0)
    };

    serde_json::json!({
        "cpu_usage": cpu,
        "memory_total": mem_total,
        "memory_used": mem_used,
        "disk_total": disk_total,
        "disk_used": disk_used,
        "os": std::env::consts::OS,
    })
}

pub async fn get_settings(pool: &SqlitePool) -> Result<serde_json::Value, AppError> {
    let rows = sqlx::query_as::<_, crate::models::setting::Setting>(
        "SELECT key, value FROM settings",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut map = serde_json::Map::new();
    for setting in rows {
        map.insert(setting.key, serde_json::Value::String(setting.value));
    }
    Ok(serde_json::Value::Object(map))
}

pub async fn update_settings(
    pool: &SqlitePool,
    settings: serde_json::Value,
) -> Result<(), AppError> {
    if let Some(obj) = settings.as_object() {
        for (key, value) in obj {
            let val_str = value.as_str().unwrap_or("").to_string();
            sqlx::query(
                "INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = ?",
            )
            .bind(key)
            .bind(&val_str)
            .bind(&val_str)
            .execute(pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }
    Ok(())
}
