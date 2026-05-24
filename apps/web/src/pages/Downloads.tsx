import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import type { DownloadTask } from "@lumina/shared";

export default function Downloads() {
  const qc = useQueryClient();
  const [url, setUrl] = useState("");
  const [savePath, setSavePath] = useState("");
  const [error, setError] = useState("");

  const { data: tasks, isLoading } = useQuery({
    queryKey: ["downloads"],
    queryFn: () => api<DownloadTask[]>("/download/list"),
    refetchInterval: 3000,
  });

  const addMutation = useMutation({
    mutationFn: (body: { url: string; save_path: string }) =>
      api("/download/add", { method: "POST", body: JSON.stringify(body) }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["downloads"] });
      setUrl("");
      setSavePath("");
      setError("");
    },
    onError: (err: Error) => setError(err.message),
  });

  const pauseMutation = useMutation({
    mutationFn: (id: number) =>
      api(`/download/${id}/pause`, { method: "POST" }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["downloads"] }),
  });

  const resumeMutation = useMutation({
    mutationFn: (id: number) =>
      api(`/download/${id}/resume`, { method: "POST" }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["downloads"] }),
  });

  const removeMutation = useMutation({
    mutationFn: (id: number) =>
      api(`/download/${id}/remove`, { method: "POST" }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["downloads"] }),
  });

  const handleAdd = (e: React.FormEvent) => {
    e.preventDefault();
    if (!url || !savePath) return;
    addMutation.mutate({ url, save_path: savePath });
  };

  return (
    <div>
      <h2 className="text-lg font-bold mb-4">下载管理</h2>

      <form onSubmit={handleAdd} className="flex gap-2 mb-4">
        <input
          type="text"
          placeholder="下载链接"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          className="border rounded px-3 py-1 text-sm flex-1"
        />
        <input
          type="text"
          placeholder="保存路径"
          value={savePath}
          onChange={(e) => setSavePath(e.target.value)}
          className="border rounded px-3 py-1 text-sm w-40"
        />
        <button
          type="submit"
          className="px-3 py-1 bg-blue-600 text-white rounded text-sm"
        >
          添加
        </button>
      </form>

      {error && <p className="text-red-500 text-sm mb-3">{error}</p>}

      {isLoading ? (
        <div>加载中...</div>
      ) : (
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b text-left">
              <th className="py-2">文件名</th>
              <th className="py-2">进度</th>
              <th className="py-2">速度</th>
              <th className="py-2">状态</th>
              <th className="py-2">操作</th>
            </tr>
          </thead>
          <tbody>
            {tasks?.map((t) => (
              <tr key={t.id} className="border-b hover:bg-gray-50">
                <td className="py-2">{t.file_name || t.url}</td>
                <td className="py-2">{t.progress.toFixed(1)}%</td>
                <td className="py-2">{formatSpeed(t.speed)}</td>
                <td className="py-2">{t.status}</td>
                <td className="py-2 flex gap-1">
                  {t.status === "downloading" && (
                    <button
                      onClick={() => pauseMutation.mutate(t.id)}
                      className="text-xs text-yellow-600"
                    >
                      暂停
                    </button>
                  )}
                  {t.status === "paused" && (
                    <button
                      onClick={() => resumeMutation.mutate(t.id)}
                      className="text-xs text-green-600"
                    >
                      继续
                    </button>
                  )}
                  <button
                    onClick={() => removeMutation.mutate(t.id)}
                    className="text-xs text-red-500"
                  >
                    删除
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec === 0) return "-";
  if (bytesPerSec < 1024) return `${bytesPerSec} B/s`;
  if (bytesPerSec < 1024 * 1024)
    return `${(bytesPerSec / 1024).toFixed(1)} KB/s`;
  return `${(bytesPerSec / (1024 * 1024)).toFixed(1)} MB/s`;
}
