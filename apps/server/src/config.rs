use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub data: DataConfig,
    #[serde(default)]
    pub auth: AuthConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DataConfig {
    #[serde(default = "default_db_path")]
    pub db_path: PathBuf,
    #[serde(default = "default_thumbnail_dir")]
    pub thumbnail_dir: PathBuf,
    #[serde(default = "default_download_dir")]
    pub download_dir: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthConfig {
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiry")]
    pub jwt_expiry_hours: u32,
}

fn default_host() -> String { "0.0.0.0".into() }
fn default_port() -> u16 { 8080 }
fn default_db_path() -> PathBuf { PathBuf::from("data/lumina.db") }
fn default_thumbnail_dir() -> PathBuf { PathBuf::from("data/thumbnails") }
fn default_download_dir() -> PathBuf { PathBuf::from("data/downloads") }
fn default_jwt_secret() -> String { "change-me-in-production".into() }
fn default_jwt_expiry() -> u32 { 24 }

impl Config {
    pub fn server_addr(&self) -> String { format!("{}:{}", self.server.host, self.server.port) }
    pub fn db_path(&self) -> &PathBuf { &self.data.db_path }
    pub fn thumbnail_dir(&self) -> &PathBuf { &self.data.thumbnail_dir }
    pub fn download_dir(&self) -> &PathBuf { &self.data.download_dir }
    pub fn jwt_secret(&self) -> &str { &self.auth.jwt_secret }
    pub fn jwt_expiry_hours(&self) -> u32 { self.auth.jwt_expiry_hours }
}

pub fn load() -> anyhow::Result<Config> {
    let config_file = std::env::var("LUMINA_CONFIG").unwrap_or_else(|_| "config.toml".to_string());
    let mut config: Config = if std::path::Path::new(&config_file).exists() {
        let content = std::fs::read_to_string(&config_file)?;
        toml::from_str(&content)?
    } else {
        tracing::warn!("config file not found at {}, using defaults", config_file);
        Config::default()
    };
    if let Ok(host) = std::env::var("LUMINA_HOST") { config.server.host = host; }
    if let Ok(port) = std::env::var("LUMINA_PORT") { if let Ok(p) = port.parse() { config.server.port = p; } }
    if let Ok(db) = std::env::var("LUMINA_DB_PATH") { config.data.db_path = db.into(); }
    if let Ok(secret) = std::env::var("LUMINA_JWT_SECRET") { config.auth.jwt_secret = secret; }
    Ok(config)
}

impl Default for ServerConfig {
    fn default() -> Self { ServerConfig { host: default_host(), port: default_port() } }
}

impl Default for DataConfig {
    fn default() -> Self { DataConfig { db_path: default_db_path(), thumbnail_dir: default_thumbnail_dir(), download_dir: default_download_dir() } }
}

impl Default for AuthConfig {
    fn default() -> Self { AuthConfig { jwt_secret: default_jwt_secret(), jwt_expiry_hours: default_jwt_expiry() } }
}
