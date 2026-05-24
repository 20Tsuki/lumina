pub struct DownloadState;
impl DownloadState {
    pub fn new() -> std::sync::Arc<Self> { std::sync::Arc::new(Self) }
}
