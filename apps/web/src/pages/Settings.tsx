import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import type { SystemInfo } from "@lumina/shared";

export default function Settings() {
  const { data: info, isLoading } = useQuery({
    queryKey: ["system", "info"],
    queryFn: () => api<SystemInfo>("/system/info"),
  });

  if (isLoading) return <div>加载中...</div>;

  return (
    <div>
      <h2 className="text-lg font-bold mb-4">系统设置</h2>

      <div className="grid grid-cols-2 gap-4 max-w-md">
        <div className="bg-gray-50 p-4 rounded">
          <p className="text-sm text-gray-500">操作系统</p>
          <p className="text-lg font-medium">{info?.os ?? "-"}</p>
        </div>
        <div className="bg-gray-50 p-4 rounded">
          <p className="text-sm text-gray-500">CPU 使用率</p>
          <p className="text-lg font-medium">
            {info?.cpu_usage != null ? `${info.cpu_usage.toFixed(1)}%` : "-"}
          </p>
        </div>
        <div className="bg-gray-50 p-4 rounded">
          <p className="text-sm text-gray-500">内存使用</p>
          <p className="text-lg font-medium">
            {info ? formatBytes(info.memory_used) : "-"} /{" "}
            {info ? formatBytes(info.memory_total) : "-"}
          </p>
        </div>
        <div className="bg-gray-50 p-4 rounded">
          <p className="text-sm text-gray-500">磁盘使用</p>
          <p className="text-lg font-medium">
            {info ? formatBytes(info.disk_used) : "-"} /{" "}
            {info ? formatBytes(info.disk_total) : "-"}
          </p>
        </div>
      </div>
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}
