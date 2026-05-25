use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use librqbit::api::TorrentIdOrHash;
use librqbit::{
    AddTorrent, AddTorrentOptions, ManagedTorrent, Session, SessionOptions, TorrentStats,
};

pub struct BtManager {
    session: Arc<Session>,
    /// task_id → torrent numeric id (usize)
    pub torrent_map: Mutex<HashMap<i64, usize>>,
    /// Store handles for pause/unpause/delete operations
    handles: Mutex<HashMap<usize, Arc<ManagedTorrent>>>,
}

impl BtManager {
    pub async fn new(download_dir: &str) -> Result<Arc<Self>, String> {
        let mut trackers = HashSet::new();
        // Well-known public trackers as fallback when DHT is unreachable
        for url_str in [
            "udp://tracker.opentrackr.org:1337/announce",
            "udp://tracker.openbittorrent.com:6969/announce",
            "udp://open.demonii.com:1337/announce",
            "udp://tracker.torrent.eu.org:451/announce",
            "http://tracker.opentrackr.org:1337/announce",
        ] {
            if let Ok(u) = url::Url::parse(url_str) {
                trackers.insert(u);
            }
        }

        let opts = SessionOptions {
            trackers,
            ..Default::default()
        };
        let session = Session::new_with_opts(PathBuf::from(download_dir), opts)
            .await
            .map_err(|e| format!("failed to create torrent session: {}", e))?;
        Ok(Arc::new(Self {
            session,
            torrent_map: Mutex::new(HashMap::new()),
            handles: Mutex::new(HashMap::new()),
        }))
    }

    pub fn is_magnet_or_torrent(url: &str) -> bool {
        url.starts_with("magnet:") || url.ends_with(".torrent")
    }

    pub async fn add_torrent(&self, url: &str, save_path: &str) -> Result<usize, String> {
        let opts = AddTorrentOptions {
            output_folder: Some(save_path.to_string()),
            ..Default::default()
        };
        let resp = self
            .session
            .add_torrent(AddTorrent::from_url(url), Some(opts))
            .await
            .map_err(|e| format!("add torrent: {}", e))?;

        match resp.into_handle() {
            Some(handle) => {
                let id = handle.id();
                self.handles.lock().await.insert(id, handle);
                Ok(id)
            }
            None => Err("failed to get torrent handle (list_only?)".to_string()),
        }
    }

    pub async fn pause(&self, torrent_id: usize) -> Result<(), String> {
        let handle = self
            .handles
            .lock()
            .await
            .get(&torrent_id)
            .cloned()
            .or_else(|| self.session.get(TorrentIdOrHash::Id(torrent_id)))
            .ok_or_else(|| "torrent not found".to_string())?;
        self.session
            .pause(&handle)
            .await
            .map_err(|e| format!("pause: {}", e))
    }

    pub async fn unpause(&self, torrent_id: usize) -> Result<(), String> {
        let handle = self
            .handles
            .lock()
            .await
            .get(&torrent_id)
            .cloned()
            .or_else(|| self.session.get(TorrentIdOrHash::Id(torrent_id)))
            .ok_or_else(|| "torrent not found".to_string())?;
        self.session
            .unpause(&handle)
            .await
            .map_err(|e| format!("unpause: {}", e))
    }

    pub async fn remove(&self, torrent_id: usize) -> Result<(), String> {
        self.session
            .delete(TorrentIdOrHash::Id(torrent_id), false)
            .await
            .map_err(|e| format!("remove: {}", e))?;
        self.handles.lock().await.remove(&torrent_id);
        Ok(())
    }

    /// Get stats for all managed torrents: (torrent_id, stats, name)
    pub fn get_all_torrents(&self) -> Vec<(usize, TorrentStats, Option<String>)> {
        self.session.with_torrents(|torrents| {
            torrents
                .map(|(id, handle)| {
                    let stats = handle.stats();
                    let name = handle.name();
                    (id, stats, name)
                })
                .collect()
        })
    }
}
