import { useState, useRef } from "react";
import { useSearchParams, Link } from "react-router-dom";
import { useFiles, useMkdir, useDelete, useMove } from "@/hooks/useFiles";
import { api } from "@/lib/api";

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function Breadcrumb({ path, onNavigate }: { path: string; onNavigate: (p: string) => void }) {
  const parts = path.split("/").filter(Boolean);
  const crumbs = [{ name: "根目录", path: "/" }];
  let current = "";
  for (const p of parts) {
    current += "/" + p;
    crumbs.push({ name: p, path: current });
  }

  return (
    <div className="flex items-center gap-1 text-sm">
      {crumbs.map((c, i) => (
        <span key={c.path} className="flex items-center gap-1">
          {i > 0 && <span className="text-gray-400">/</span>}
          {i < crumbs.length - 1 ? (
            <button
              onClick={() => onNavigate(c.path)}
              className="text-blue-600 hover:underline"
            >
              {c.name}
            </button>
          ) : (
            <span className="text-gray-700 font-medium">{c.name}</span>
          )}
        </span>
      ))}
    </div>
  );
}

export default function FileBrowser() {
  const [searchParams, setSearchParams] = useSearchParams();
  const path = searchParams.get("path") || "/";
  const { data: files, isLoading, refetch } = useFiles(path);
  const mkdir = useMkdir();
  const del = useDelete();
  const move = useMove();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [mkdirName, setMkdirName] = useState("");
  const [showMkdir, setShowMkdir] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [renaming, setRenaming] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");

  const navigateTo = (p: string) => setSearchParams({ path: p });
  const goUp = () => {
    const parts = path.split("/").filter(Boolean);
    parts.pop();
    navigateTo(parts.length ? "/" + parts.join("/") : "/");
  };

  const handleUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    setUploading(true);
    try {
      const form = new FormData();
      form.append("path", path);
      form.append("file", file);
      const token = localStorage.getItem("lumina_token");
      const headers: Record<string, string> = {};
      if (token) headers["Authorization"] = `Bearer ${token}`;
      const res = await fetch("/api/files/upload", { method: "POST", body: form, headers });
      if (!res.ok) throw new Error("上传失败");
      refetch();
    } catch (err) {
      alert(err instanceof Error ? err.message : "上传失败");
    } finally {
      setUploading(false);
      if (fileInputRef.current) fileInputRef.current.value = "";
    }
  };

  const startRename = (name: string, currentPath: string) => {
    setRenaming(currentPath);
    setRenameValue(name);
  };

  const submitRename = (fromPath: string) => {
    if (!renameValue.trim()) return;
    const parts = fromPath.split("/");
    parts.pop();
    const toPath = (parts.length ? parts.join("/") : "") + "/" + renameValue;
    move.mutate({ from: fromPath, to: toPath }, { onSuccess: () => setRenaming(null) });
  };

  if (isLoading) return <div className="text-sm text-gray-500 p-4">加载中...</div>;

  return (
    <div>
      <div className="flex items-center gap-2 mb-3 flex-wrap">
        <button onClick={goUp} className="text-sm px-2 py-1 border rounded hover:bg-gray-100">
          ⬆ 上一级
        </button>
        <Breadcrumb path={path} onNavigate={navigateTo} />
        <div className="ml-auto flex gap-2">
          <button
            onClick={() => fileInputRef.current?.click()}
            disabled={uploading}
            className="text-sm px-3 py-1 bg-green-600 text-white rounded hover:bg-green-700 disabled:opacity-50"
          >
            {uploading ? "上传中..." : "上传文件"}
          </button>
          <input
            ref={fileInputRef}
            type="file"
            className="hidden"
            onChange={handleUpload}
          />
          <button
            onClick={() => setShowMkdir(true)}
            className="text-sm px-3 py-1 border rounded hover:bg-gray-100"
          >
            新建文件夹
          </button>
        </div>
      </div>

      {showMkdir && (
        <div className="flex gap-2 mb-4">
          <input
            value={mkdirName}
            onChange={(e) => setMkdirName(e.target.value)}
            placeholder="文件夹名称"
            className="border px-2 py-1 text-sm rounded"
            autoFocus
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                mkdir.mutate({ path, name: mkdirName });
                setMkdirName("");
                setShowMkdir(false);
              }
              if (e.key === "Escape") setShowMkdir(false);
            }}
          />
          <button
            onClick={() => {
              if (!mkdirName.trim()) return;
              mkdir.mutate({ path, name: mkdirName.trim() });
              setMkdirName("");
              setShowMkdir(false);
            }}
            className="text-sm px-2 py-1 bg-blue-600 text-white rounded"
          >
            创建
          </button>
          <button
            onClick={() => setShowMkdir(false)}
            className="text-sm px-2 py-1 border rounded"
          >
            取消
          </button>
        </div>
      )}

      <table className="w-full text-sm">
        <thead>
          <tr className="border-b text-left">
            <th className="py-2">名称</th>
            <th className="py-2">类型</th>
            <th className="py-2">大小</th>
            <th className="py-2 w-32">操作</th>
          </tr>
        </thead>
        <tbody>
          {files?.map((f) => (
            <tr key={f.path} className="border-b hover:bg-gray-50">
              <td className="py-2">
                {renaming === f.path ? (
                  <input
                    value={renameValue}
                    onChange={(e) => setRenameValue(e.target.value)}
                    className="border px-1 py-0.5 text-sm rounded w-40"
                    autoFocus
                    onKeyDown={(e) => {
                      if (e.key === "Enter") submitRename(f.path);
                      if (e.key === "Escape") setRenaming(null);
                    }}
                    onBlur={() => setRenaming(null)}
                  />
                ) : f.is_dir ? (
                  <button
                    onClick={() => navigateTo(f.path)}
                    className="text-blue-600 hover:underline"
                  >
                    {f.name}
                  </button>
                ) : (
                  <span>{f.name}</span>
                )}
              </td>
              <td className="py-2 text-gray-500">{f.file_type}</td>
              <td className="py-2 text-gray-500">
                {f.is_dir ? "-" : formatSize(f.size)}
              </td>
              <td className="py-2">
                <div className="flex gap-1">
                  {!f.is_dir && (
                    <Link
                      to={`/api/files/download?path=${encodeURIComponent(f.path)}`}
                      className="text-blue-500 text-xs hover:underline"
                    >
                      下载
                    </Link>
                  )}
                  <button
                    onClick={() => startRename(f.name, f.path)}
                    className="text-gray-500 text-xs hover:underline"
                  >
                    重命名
                  </button>
                  <button
                    onClick={() => del.mutate({ path: f.path })}
                    className="text-red-500 text-xs hover:underline"
                  >
                    删除
                  </button>
                </div>
              </td>
            </tr>
          ))}
          {files?.length === 0 && (
            <tr>
              <td colSpan={4} className="py-8 text-center text-gray-400">
                空目录
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
