import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useMovies, useSearch } from "@/hooks/useMedia";
import type { IndexedFile } from "@lumina/shared";

export default function MediaLibrary() {
  const [page, setPage] = useState(1);
  const [search, setSearch] = useState("");
  const { data, isLoading } = useMovies(page);
  const { data: searchResults } = useSearch(search);
  const navigate = useNavigate();

  const items = search ? searchResults : data?.items;

  if (isLoading) return <div>加载中...</div>;

  return (
    <div>
      <div className="flex items-center gap-4 mb-4">
        <h2 className="text-lg font-bold">影视库</h2>
        <input
          type="text"
          placeholder="搜索..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="border rounded px-3 py-1 text-sm"
        />
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-4">
        {items?.map((movie) => (
          <div
            key={movie.id}
            className="cursor-pointer"
            onClick={() => navigate(`/player/${movie.id}`)}
          >
            <div className="aspect-[2/3] bg-gray-200 rounded flex items-center justify-center text-gray-400">
              {movie.thumb_path ? (
                <img
                  src={movie.thumb_path}
                  alt={movie.title}
                  className="w-full h-full object-cover rounded"
                />
              ) : (
                "无海报"
              )}
            </div>
            <p className="text-sm mt-1 truncate">{movie.title}</p>
          </div>
        ))}
      </div>

      {!search && data?.next_cursor && (
        <div className="flex justify-center mt-4 gap-2">
          <button
            onClick={() => setPage((p) => Math.max(1, p - 1))}
            disabled={page === 1}
            className="px-3 py-1 border rounded text-sm"
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
