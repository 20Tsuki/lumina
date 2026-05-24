CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'viewer',
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS libraries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    library_type TEXT NOT NULL DEFAULT 'mixed',
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS indexed_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    file_type TEXT NOT NULL DEFAULT 'other',
    title TEXT NOT NULL,
    size INTEGER NOT NULL DEFAULT 0,
    codec TEXT,
    resolution TEXT,
    duration INTEGER,
    bitrate INTEGER,
    thumb_path TEXT,
    metadata_json TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(library_id, file_path)
);

CREATE TABLE IF NOT EXISTS series (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    year INTEGER,
    plot TEXT,
    poster_url TEXT,
    tmdb_id INTEGER
);

CREATE TABLE IF NOT EXISTS seasons (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    series_id INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    season_number INTEGER NOT NULL,
    UNIQUE(series_id, season_number)
);

CREATE TABLE IF NOT EXISTS episodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    season_id INTEGER NOT NULL REFERENCES seasons(id) ON DELETE CASCADE,
    episode_number INTEGER NOT NULL,
    title TEXT NOT NULL,
    file_id INTEGER REFERENCES indexed_files(id) ON DELETE SET NULL,
    UNIQUE(season_id, episode_number)
);

CREATE TABLE IF NOT EXISTS download_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    save_path TEXT NOT NULL,
    file_name TEXT,
    progress REAL NOT NULL DEFAULT 0.0,
    speed INTEGER NOT NULL DEFAULT 0,
    size INTEGER NOT NULL DEFAULT 0,
    eta INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'queued',
    error_msg TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_indexed_files_library ON indexed_files(library_id);
CREATE INDEX IF NOT EXISTS idx_indexed_files_type ON indexed_files(file_type);
CREATE INDEX IF NOT EXISTS idx_indexed_files_status ON indexed_files(status);
CREATE INDEX IF NOT EXISTS idx_episodes_season ON episodes(season_id);
CREATE INDEX IF NOT EXISTS idx_seasons_series ON seasons(series_id);
CREATE INDEX IF NOT EXISTS idx_download_tasks_status ON download_tasks(status);
