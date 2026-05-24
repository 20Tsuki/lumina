export const API_BASE = "/api";

export const FILE_TYPE_LABELS: Record<string, string> = {
  video: "视频",
  image: "图片",
  audio: "音频",
  document: "文档",
  archive: "压缩包",
  other: "其他",
};

export const DOWNLOAD_STATUS_LABELS: Record<string, string> = {
  queued: "排队中",
  downloading: "下载中",
  seeding: "做种中",
  paused: "已暂停",
  done: "已完成",
  error: "失败",
};

export const VIDEO_EXTENSIONS = new Set(["mp4", "mkv", "avi", "mov", "m2ts", "ts", "wmv", "flv"]);
export const IMAGE_EXTENSIONS = new Set(["jpg", "jpeg", "png", "gif", "webp", "heic", "bmp"]);
export const AUDIO_EXTENSIONS = new Set(["mp3", "flac", "aac", "wav", "ogg"]);
export const ARCHIVE_EXTENSIONS = new Set(["zip", "rar", "7z", "tar", "gz", "bz2"]);
export const DOCUMENT_EXTENSIONS = new Set(["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "md"]);
