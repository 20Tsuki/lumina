import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import type { FileEntry } from "@lumina/shared";

interface FolderPickerProps {
  value: string;
  onChange: (path: string) => void;
  placeholder?: string;
}

export default function FolderPicker({ value, onChange, placeholder }: FolderPickerProps) {
  const [open, setOpen] = useState(false);
  const [currentPath, setCurrentPath] = useState("/");

  const { data: entries = [], isLoading, isError, error } = useQuery({
    queryKey: ["files", currentPath],
    queryFn: () => api<FileEntry[]>(`/files/list?path=${encodeURIComponent(currentPath)}`),
    enabled: open,
    staleTime: 30_000,
  });

  const dirs = entries.filter((f) => f.is_dir);

  function openPicker() {
    setCurrentPath(value || "/");
    setOpen(true);
  }

  function navigateTo(dir: FileEntry) {
    setCurrentPath(dir.path);
  }

  function goUp() {
    const parent = currentPath.replace(/\/[^/]+$/, "") || "/";
    setCurrentPath(parent);
  }

  function selectFolder() {
    onChange(currentPath);
    setOpen(false);
  }

  const pathSegments = currentPath === "/" ? [] : currentPath.split("/").filter(Boolean);

  return (
    <div className="flex gap-1">
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder ?? "文件夹路径"}
        className="border rounded px-3 py-2 text-sm flex-1"
      />
      <button
        type="button"
        onClick={openPicker}
        className="px-3 py-2 border rounded text-sm hover:bg-gray-50 whitespace-nowrap"
      >
        浏览
      </button>

      {open && (
        <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50" onClick={() => setOpen(false)}>
          <div className="bg-white rounded-lg p-6 w-[500px] max-w-[90vw] max-h-[70vh] flex flex-col" onClick={(e) => e.stopPropagation()}>
            <h3 className="text-lg font-bold mb-3">选择文件夹</h3>

            <div className="flex items-center gap-1 mb-3 text-sm flex-wrap">
              <button onClick={() => setCurrentPath("/")} className="text-blue-600 hover:underline">
                /
              </button>
              {pathSegments.map((seg, i) => {
                const p = "/" + pathSegments.slice(0, i + 1).join("/");
                return (
                  <span key={p} className="flex items-center gap-1">
                    <span className="text-gray-300">/</span>
                    <button onClick={() => setCurrentPath(p)} className="text-blue-600 hover:underline">
                      {seg}
                    </button>
                  </span>
                );
              })}
            </div>

            <div className="flex-1 overflow-y-auto border rounded mb-3 min-h-[200px]">
              {isLoading ? (
                <div className="p-4 text-sm text-gray-400">加载中...</div>
              ) : isError ? (
                <div className="p-4 text-sm text-red-500">
                  {(error as Error)?.message ?? "加载失败"}
                </div>
              ) : dirs.length === 0 ? (
                <div className="p-4 text-sm text-gray-400">此目录下没有子文件夹</div>
              ) : (
                <div className="divide-y">
                  {currentPath !== "/" && (
                    <button
                      onClick={goUp}
                      className="w-full text-left px-3 py-2 text-sm text-gray-500 hover:bg-gray-50 flex items-center gap-2"
                    >
                      📁 ..
                    </button>
                  )}
                  {dirs.map((d) => (
                    <button
                      key={d.path}
                      onClick={() => navigateTo(d)}
                      className={`w-full text-left px-3 py-2 text-sm hover:bg-gray-50 flex items-center gap-2 ${
                        d.path === currentPath ? "bg-blue-50" : ""
                      }`}
                    >
                      📁 {d.name}
                    </button>
                  ))}
                </div>
              )}
            </div>

            <div className="flex justify-between items-center">
              <span className="text-xs text-gray-400 truncate max-w-[300px]">
                当前: {currentPath}
              </span>
              <div className="flex gap-2">
                <button
                  onClick={() => setOpen(false)}
                  className="px-4 py-2 border rounded text-sm hover:bg-gray-50"
                >
                  取消
                </button>
                <button
                  onClick={selectFolder}
                  className="px-4 py-2 bg-blue-600 text-white rounded text-sm hover:bg-blue-700"
                >
                  选择此目录
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
