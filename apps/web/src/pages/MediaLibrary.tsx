import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useMovies, useSearch, useScanTrigger, useScanStatus } from "@/hooks/useMedia";
import type { IndexedFile } from "@lumina/shared";

function formatDuration(seconds: number | null | undefined): string {
  if (!seconds) return "";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (h > 0) return `${h}h${m}m`;
  return `${m}min`;
}

export default function MediaLibrary() {
  const [page, setPage] = useState(1);
  const [search, setSearch] = useState("");
  const { data, isLoading } = useMovies(page);
  const { data: searchResults } = useSearch(search);
  const { data: scanStatus } = useScanStatus();
  const scanTrigger = useScanTrigger();
  const navigate = useNavigate();

  const items = search ? searchResults : data?.items;
  const isScanning = scanStatus?.status === "scanning";

  if (isLoading) return <div className="text-sm text-gray-500 p-4">加载中...</div>;

  return (
    <div>
      <div className="flex items-center gap-4 mb-4 flex-wrap">
        <h2 className="text-lg font-bold">影视库</h2>
        <input
          type="text"
          placeholder="搜索..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="border rounded px-3 py-1 text-sm"
        />
        <button
          onClick={() => scanTrigger.mutate()}
          disabled={isScanning}
          className="text-sm px-3 py-1 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
        >
          {isScanning ? "扫描中..." : "扫描媒体库"}
        </button>
      </div>

      {scanStatus && scanStatus.status !== "idle" && (
        <div className="mb-4 p-3 bg-gray-50 rounded text-sm">
          {isScanning ? (
            <div>
              <div className="flex items-center gap-2 mb-1">
                <span className="text-blue-600 font-medium">正在扫描...</span>
                <span className="text-gray-500">
                  {scanStatus.processed}/{scanStatus.total}
                </span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-2">
                <div
                  className="bg-blue-600 h-2 rounded-full transition-all"
                  style={{
                    width: scanStatus.total > 0
                      ? `${(scanStatus.processed / scanStatus.total) * 100}%`
                      : "0%",
                  }}
                />
              </div>
              {scanStatus.current_file && (
                <p className="text-gray-400 text-xs mt-1 truncate">
                  {scanStatus.current_file}
                </p>
              )}
            </div>
          ) : scanStatus.status === "done" ? (
            <p className="text-green-600">扫描完成，共处理 {scanStatus.total} 个文件</p>
          ) : scanStatus.status === "error" ? (
            <p className="text-red-600">扫描出错</p>
          ) : null}
        </div>
      )}

      <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-4">
        {items?.map((movie) => (
          <div
            key={movie.id}
            className="cursor-pointer group"
            onClick={() => navigate(`/player/${movie.id}`)}
          >
            <div className="aspect-[2/3] bg-gray-200 rounded flex items-center justify-center text-gray-400 overflow-hidden relative">
              {movie.thumb_path ? (
                <img
                  src={`/api/files/thumbnail?path=${encodeURIComponent(movie.thumb_path)}&size=256`}
                  alt={movie.title}
                  className="w-full h-full object-cover rounded"
                />
              ) : (
                "无海报"
              )}
              {movie.duration != null && movie.duration > 0 && (
                <span className="absolute bottom-1 right-1 bg-black/70 text-white text-xs px-1 rounded">
                  {formatDuration(movie.duration)}
                </span>
              )}
            </div>
            <p className="text-sm mt-1 truncate">{movie.title}</p>
            {movie.resolution && (
              <p className="text-xs text-gray-400">{movie.resolution}</p>
            )}
          </div>
        ))}
      </div>

      {items?.length === 0 && (
        <div className="py-12 text-center text-gray-400">
          {search ? "未找到匹配的媒体" : "媒体库为空，请先扫描媒体库"}
        </div>
      )}

      {!search && data?.next_cursor && (
        <div className="flex justify-center mt-4 gap-2">
          <button
            onClick={() => setPage((p) => Math.max(1, p - 1))}
            disabled={page === 1}
            className="px-3 py-1 border rounded text-sm disabled:opacity-30"
          >
            上一页
          </button>
          <button
            onClick={() => setPage((p) => p + 1)}
            className="px-3 py-1 border rounded text-sm"
          >
            下一页
          </button>
        </div>
      )}
    </div>
  );
}
