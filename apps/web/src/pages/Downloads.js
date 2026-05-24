import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
export default function Downloads() {
    const qc = useQueryClient();
    const [url, setUrl] = useState("");
    const [savePath, setSavePath] = useState("");
    const [error, setError] = useState("");
    const { data: tasks, isLoading } = useQuery({
        queryKey: ["downloads"],
        queryFn: () => api("/download/list"),
        refetchInterval: 3000,
    });
    const addMutation = useMutation({
        mutationFn: (body) => api("/download/add", { method: "POST", body: JSON.stringify(body) }),
        onSuccess: () => {
            qc.invalidateQueries({ queryKey: ["downloads"] });
            setUrl("");
            setSavePath("");
            setError("");
        },
        onError: (err) => setError(err.message),
    });
    const pauseMutation = useMutation({
        mutationFn: (id) => api(`/download/${id}/pause`, { method: "POST" }),
        onSuccess: () => qc.invalidateQueries({ queryKey: ["downloads"] }),
    });
    const resumeMutation = useMutation({
        mutationFn: (id) => api(`/download/${id}/resume`, { method: "POST" }),
        onSuccess: () => qc.invalidateQueries({ queryKey: ["downloads"] }),
    });
    const removeMutation = useMutation({
        mutationFn: (id) => api(`/download/${id}/remove`, { method: "POST" }),
        onSuccess: () => qc.invalidateQueries({ queryKey: ["downloads"] }),
    });
    const handleAdd = (e) => {
        e.preventDefault();
        if (!url || !savePath)
            return;
        addMutation.mutate({ url, save_path: savePath });
    };
    return (_jsxs("div", { children: [_jsx("h2", { className: "text-lg font-bold mb-4", children: "\u4E0B\u8F7D\u7BA1\u7406" }), _jsxs("form", { onSubmit: handleAdd, className: "flex gap-2 mb-4", children: [_jsx("input", { type: "text", placeholder: "\u4E0B\u8F7D\u94FE\u63A5", value: url, onChange: (e) => setUrl(e.target.value), className: "border rounded px-3 py-1 text-sm flex-1" }), _jsx("input", { type: "text", placeholder: "\u4FDD\u5B58\u8DEF\u5F84", value: savePath, onChange: (e) => setSavePath(e.target.value), className: "border rounded px-3 py-1 text-sm w-40" }), _jsx("button", { type: "submit", className: "px-3 py-1 bg-blue-600 text-white rounded text-sm", children: "\u6DFB\u52A0" })] }), error && _jsx("p", { className: "text-red-500 text-sm mb-3", children: error }), isLoading ? (_jsx("div", { children: "\u52A0\u8F7D\u4E2D..." })) : (_jsxs("table", { className: "w-full text-sm", children: [_jsx("thead", { children: _jsxs("tr", { className: "border-b text-left", children: [_jsx("th", { className: "py-2", children: "\u6587\u4EF6\u540D" }), _jsx("th", { className: "py-2", children: "\u8FDB\u5EA6" }), _jsx("th", { className: "py-2", children: "\u901F\u5EA6" }), _jsx("th", { className: "py-2", children: "\u72B6\u6001" }), _jsx("th", { className: "py-2", children: "\u64CD\u4F5C" })] }) }), _jsx("tbody", { children: tasks?.map((t) => (_jsxs("tr", { className: "border-b hover:bg-gray-50", children: [_jsx("td", { className: "py-2", children: t.file_name || t.url }), _jsxs("td", { className: "py-2", children: [t.progress.toFixed(1), "%"] }), _jsx("td", { className: "py-2", children: formatSpeed(t.speed) }), _jsx("td", { className: "py-2", children: t.status }), _jsxs("td", { className: "py-2 flex gap-1", children: [t.status === "downloading" && (_jsx("button", { onClick: () => pauseMutation.mutate(t.id), className: "text-xs text-yellow-600", children: "\u6682\u505C" })), t.status === "paused" && (_jsx("button", { onClick: () => resumeMutation.mutate(t.id), className: "text-xs text-green-600", children: "\u7EE7\u7EED" })), _jsx("button", { onClick: () => removeMutation.mutate(t.id), className: "text-xs text-red-500", children: "\u5220\u9664" })] })] }, t.id))) })] }))] }));
}
function formatSpeed(bytesPerSec) {
    if (bytesPerSec === 0)
        return "-";
    if (bytesPerSec < 1024)
        return `${bytesPerSec} B/s`;
    if (bytesPerSec < 1024 * 1024)
        return `${(bytesPerSec / 1024).toFixed(1)} KB/s`;
    return `${(bytesPerSec / (1024 * 1024)).toFixed(1)} MB/s`;
}
