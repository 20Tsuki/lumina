use std::collections::HashMap;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Aria2Manager {
    rpc_url: String,
    secret: String,
    process: Mutex<Option<Child>>,
    /// task_id → aria2 GID
    pub gid_map: Mutex<HashMap<i64, String>>,
    available: AtomicBool,
}

impl Aria2Manager {
    pub fn new(rpc_port: u16, secret: &str) -> Arc<Self> {
        Arc::new(Self {
            rpc_url: format!("http://127.0.0.1:{}/jsonrpc", rpc_port),
            secret: secret.to_string(),
            process: Mutex::new(None),
            gid_map: Mutex::new(HashMap::new()),
            available: AtomicBool::new(false),
        })
    }

    pub fn is_available(&self) -> bool {
        self.available.load(Ordering::SeqCst)
    }

    pub fn is_magnet_or_torrent(url: &str) -> bool {
        url.starts_with("magnet:") || url.ends_with(".torrent")
    }

    /// Try to start aria2c daemon. Returns true if aria2 is usable.
    pub async fn start(&self) -> bool {
        // Check if aria2 RPC is already running
        if self.ping().await {
            tracing::info!("aria2c RPC already running on {}", self.rpc_url);
            self.available.store(true, Ordering::SeqCst);
            return true;
        }

        // Try to spawn aria2c
        match Command::new("aria2c")
            .args([
                "--enable-rpc",
                "--rpc-listen-port=6800",
                &format!("--rpc-secret={}", self.secret),
                "--quiet",
            ])
            .spawn()
        {
            Ok(child) => {
                tracing::info!("aria2c started (pid: {})", child.id());
                *self.process.lock().await = Some(child);
                // Wait for aria2 to be ready
                for _ in 0..10 {
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                    if self.ping().await {
                        self.available.store(true, Ordering::SeqCst);
                        return true;
                    }
                }
                tracing::warn!("aria2c started but RPC not responding");
                false
            }
            Err(e) => {
                tracing::warn!("aria2c not found ({}) — BT/magnet disabled, HTTP only", e);
                false
            }
        }
    }

    /// Gracefully shutdown aria2c
    #[allow(dead_code)]
    pub async fn shutdown(&self) {
        if let Some(mut child) = self.process.lock().await.take() {
            let _ = child.kill();
            let _ = child.wait();
            tracing::info!("aria2c stopped");
        }
    }

    async fn ping(&self) -> bool {
        self.rpc_call::<serde_json::Value>("system.listMethods", &serde_json::json!([]))
            .await
            .is_ok()
    }

    /// Add a download URI to aria2. Returns the GID.
    pub async fn add_uri(
        &self,
        url: &str,
        save_path: &str,
    ) -> Result<String, String> {
        let params = serde_json::json!([
            format!("token:{}", self.secret),
            [url],
            {"dir": save_path}
        ]);

        let resp: Aria2Response<String> = self.rpc_call("aria2.addUri", &params).await?;
        Ok(resp.result)
    }

    /// Tell aria2 to pause a download (remove from active, move to waiting/paused)
    pub async fn pause(&self, gid: &str) -> Result<(), String> {
        let params = serde_json::json!([format!("token:{}", self.secret), gid]);
        self.rpc_call::<serde_json::Value>("aria2.pause", &params).await?;
        Ok(())
    }

    /// Tell aria2 to unpause/resume a download
    pub async fn unpause(&self, gid: &str) -> Result<(), String> {
        let params = serde_json::json!([format!("token:{}", self.secret), gid]);
        self.rpc_call::<serde_json::Value>("aria2.unpause", &params).await?;
        Ok(())
    }

    /// Tell aria2 to remove a download
    pub async fn remove(&self, gid: &str) -> Result<(), String> {
        let params = serde_json::json!([format!("token:{}", self.secret), gid]);
        self.rpc_call::<serde_json::Value>("aria2.remove", &params).await?;
        // Clean up result so it doesn't clog memory
        let _ = self.rpc_call::<serde_json::Value>(
            "aria2.removeDownloadResult",
            &serde_json::json!([format!("token:{}", self.secret), gid]),
        )
        .await;
        Ok(())
    }

    /// Get status of a single download
    #[allow(dead_code)]
    pub async fn tell_status(&self, gid: &str) -> Result<Aria2Status, String> {
        let params = serde_json::json!([format!("token:{}", self.secret), gid]);
        let resp: Aria2Response<Aria2Status> = self.rpc_call("aria2.tellStatus", &params).await?;
        Ok(resp.result)
    }

    /// Get all active downloads
    pub async fn tell_active(&self) -> Result<Vec<Aria2Status>, String> {
        let params = serde_json::json!([format!("token:{}", self.secret)]);
        let resp: Aria2Response<Vec<Aria2Status>> = self.rpc_call("aria2.tellActive", &params).await?;
        Ok(resp.result)
    }

    /// Get all waiting/paused downloads
    pub async fn tell_waiting(&self) -> Result<Vec<Aria2Status>, String> {
        let params = serde_json::json!([format!("token:{}", self.secret), 0, 1000]);
        let resp: Aria2Response<Vec<Aria2Status>> = self.rpc_call("aria2.tellWaiting", &params).await?;
        Ok(resp.result)
    }

    async fn rpc_call<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: &serde_json::Value,
    ) -> Result<Aria2Response<T>, String> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "lumina",
            "method": method,
            "params": params,
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("aria2 rpc: {}", e))?;

        let text = resp.text().await.map_err(|e| format!("aria2 response: {}", e))?;
        serde_json::from_str::<Aria2Response<T>>(&text)
            .map_err(|e| format!("aria2 parse: {} — body: {}", e, text))
    }
}

#[derive(serde::Deserialize)]
struct Aria2Response<T> {
    result: T,
}

#[allow(dead_code)]
#[derive(serde::Deserialize, Clone)]
pub struct Aria2Status {
    #[serde(rename = "gid")]
    pub gid: String,
    #[serde(default)]
    pub status: String,
    #[serde(rename = "totalLength", default)]
    pub total_length: String,
    #[serde(rename = "completedLength", default)]
    pub completed_length: String,
    #[serde(rename = "downloadSpeed", default)]
    pub download_speed: String,
    #[serde(rename = "uploadSpeed", default)]
    pub upload_speed: String,
    #[serde(default)]
    pub files: Vec<Aria2File>,
    #[serde(rename = "bittorrent", default)]
    pub bittorrent: Option<serde_json::Value>,
    #[serde(rename = "errorMessage", default)]
    pub error_message: Option<String>,
    #[serde(rename = "followedBy", default)]
    pub followed_by: Option<Vec<String>>,
}

#[allow(dead_code)]
#[derive(serde::Deserialize, Clone)]
pub struct Aria2File {
    pub path: String,
    pub length: String,
    #[serde(rename = "completedLength", default)]
    pub completed_length: String,
    pub selected: Option<String>,
}

impl Aria2Status {
    pub fn total_bytes(&self) -> i64 {
        self.total_length.parse().unwrap_or(0)
    }

    pub fn completed_bytes(&self) -> i64 {
        self.completed_length.parse().unwrap_or(0)
    }

    pub fn speed_bytes(&self) -> i64 {
        self.download_speed.parse().unwrap_or(0)
    }

    pub fn progress(&self) -> f64 {
        let total = self.total_bytes();
        if total > 0 {
            (self.completed_bytes() as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn eta(&self) -> i64 {
        let speed = self.speed_bytes();
        let remaining = self.total_bytes() - self.completed_bytes();
        if speed > 0 {
            remaining / speed
        } else {
            0
        }
    }

    pub fn file_name(&self) -> Option<String> {
        self.files.iter().find_map(|f| {
            if f.selected.as_deref() == Some("true") || self.files.len() == 1 {
                std::path::Path::new(&f.path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(String::from)
            } else {
                None
            }
        })
    }

    /// Map aria2 status to our download task status
    pub fn mapped_status(&self) -> &str {
        match self.status.as_str() {
            "active" => "downloading",
            "waiting" => "queued",
            "paused" => "paused",
            "error" => "failed",
            "complete" => "completed",
            "removed" => "removed",
            _ => "queued",
        }
    }
}
