import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
export default function Settings() {
    const { data: info, isLoading } = useQuery({
        queryKey: ["system", "info"],
        queryFn: () => api("/system/info"),
    });
    if (isLoading)
        return _jsx("div", { children: "\u52A0\u8F7D\u4E2D..." });
    return (_jsxs("div", { children: [_jsx("h2", { className: "text-lg font-bold mb-4", children: "\u7CFB\u7EDF\u8BBE\u7F6E" }), _jsxs("div", { className: "grid grid-cols-2 gap-4 max-w-md", children: [_jsxs("div", { className: "bg-gray-50 p-4 rounded", children: [_jsx("p", { className: "text-sm text-gray-500", children: "\u64CD\u4F5C\u7CFB\u7EDF" }), _jsx("p", { className: "text-lg font-medium", children: info?.os ?? "-" })] }), _jsxs("div", { className: "bg-gray-50 p-4 rounded", children: [_jsx("p", { className: "text-sm text-gray-500", children: "CPU \u4F7F\u7528\u7387" }), _jsx("p", { className: "text-lg font-medium", children: info?.cpu_usage != null ? `${info.cpu_usage.toFixed(1)}%` : "-" })] }), _jsxs("div", { className: "bg-gray-50 p-4 rounded", children: [_jsx("p", { className: "text-sm text-gray-500", children: "\u5185\u5B58\u4F7F\u7528" }), _jsxs("p", { className: "text-lg font-medium", children: [info ? formatBytes(info.memory_used) : "-", " /", " ", info ? formatBytes(info.memory_total) : "-"] })] }), _jsxs("div", { className: "bg-gray-50 p-4 rounded", children: [_jsx("p", { className: "text-sm text-gray-500", children: "\u78C1\u76D8\u4F7F\u7528" }), _jsxs("p", { className: "text-lg font-medium", children: [info ? formatBytes(info.disk_used) : "-", " /", " ", info ? formatBytes(info.disk_total) : "-"] })] })] })] }));
}
function formatBytes(bytes) {
    if (bytes < 1024)
        return `${bytes} B`;
    if (bytes < 1024 * 1024)
        return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024)
        return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}
