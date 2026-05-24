export interface User {
  id: number;
  username: string;
  role: "admin" | "viewer";
  created_at: number;
}

export interface Library {
  id: number;
  name: string;
  path: string;
  library_type: "movies" | "series" | "mixed";
  created_at: number;
}

export type FileType = "video" | "image" | "audio" | "document" | "archive" | "other";

export interface IndexedFile {
  id: number;
  library_id: number;
  file_path: string;
  file_type: FileType;
  title: string;
  size: number;
  codec: string | null;
  resolution: string | null;
  duration: number | null;
  bitrate: number | null;
  thumb_path: string | null;
  metadata_json: string | null;
  status: "pending" | "ready" | "error";
  created_at: number;
  updated_at: number;
}

export interface Series {
  id: number;
  title: string;
  year: number | null;
  plot: string | null;
  poster_url: string | null;
  tmdb_id: number | null;
}

export interface Season {
  id: number;
  series_id: number;
  season_number: number;
}

export interface Episode {
  id: number;
  season_id: number;
  episode_number: number;
  title: string;
  file_id: number;
}

export type DownloadStatus = "queued" | "downloading" | "seeding" | "paused" | "done" | "error";

export interface DownloadTask {
  id: number;
  url: string;
  save_path: string;
  file_name: string | null;
  progress: number;
  speed: number;
  size: number;
  eta: number;
  status: DownloadStatus;
  error_msg: string | null;
  created_at: number;
  updated_at: number;
}

export interface FileEntry {
  name: string;
  path: string;
  file_type: FileType;
  size: number;
  is_dir: boolean;
  modified_at: number;
  thumbnail_url: string | null;
}

export interface ScanStatus {
  total: number;
  processed: number;
  current_file: string;
  status: "idle" | "scanning" | "done" | "error";
}

export interface SystemInfo {
  cpu_usage: number;
  memory_total: number;
  memory_used: number;
  disk_total: number;
  disk_used: number;
  os: string;
}

export interface ApiError {
  error: {
    code: string;
    message: string;
  };
}

export interface PaginatedResponse<T> {
  items: T[];
  next_cursor: string | null;
  total: number;
}
