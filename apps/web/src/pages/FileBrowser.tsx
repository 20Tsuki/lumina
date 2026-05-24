import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useFiles, useMkdir, useDelete } from "@/hooks/useFiles";

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

export default function FileBrowser() {
  const [searchParams, setSearchParams] = useSearchParams();
  const path = searchParams.get("path") || "/";
  const { data: files, isLoading } = useFiles(path);
  const mkdir = useMkdir();
  const del = useDelete();
  const [mkdirName, setMkdirName] = useState("");
  const [showMkdir, setShowMkdir] = useState(false);

  const navigateTo = (p: string) => setSearchParams({ path: p });
  const goUp = () => {
    const parts = path.split("/").filter(Boolean);
    parts.pop();
    navigateTo(parts.length ? "/" + parts.join("/") : "/");
  };

  if (isLoading) return <div>加载中...</div>;

  return (
    <div>
      <div className="flex items-center gap-2 mb-4">
        <button
          onClick={goUp}
          className="text-sm px-2 py-1 border rounded"
        >
          上一级
        </button>
        <span className="text-sm text-gray-500">{path}</span>
        <button
          onClick={() => setShowMkdir(true)}
          className="text-sm px-2 py-1 border rounded ml-auto"
        >
          新建文件夹
        </button>
      </div>

      {showMkdir && (
        <div className="flex gap-2 mb-4">
          <input
            value={mkdirName}
            onChange={(e) => setMkdirName(e.target.value)}
            placeholder="文件夹名称"
            className="border px-2 py-1 text-sm rounded"
          />
          <button
            onClick={() => {
              mkdir.mutate({ path, name: mkdirName });
              setMkdirName("");
              setShowMkdir(false);
            }}
            className="text-sm px-2 py-1 bg-blue-600 text-white rounded"
          >
            创建
          </button>
        </div>
      )}

      <table className="w-full text-sm">
        <thead>
          <tr className="border-b text-left">
            <th className="py-2">名称</th>
            <th className="py-2">类型</th>
            <th className="py-2">大小</th>
            <th className="py-2">操作</th>
          </tr>
        </thead>
        <tbody>
          {files?.map((f) => (
            <tr key={f.path} className="border-b hover:bg-gray-50">
              <td className="py-2">
                {f.is_dir ? (
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
                <button
                  onClick={() => del.mutate({ path: f.path })}
                  className="text-red-500 text-xs"
                >
                  删除
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
