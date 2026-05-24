import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useFiles, useMkdir, useDelete } from "@/hooks/useFiles";
function formatSize(bytes) {
    if (bytes < 1024)
        return `${bytes} B`;
    if (bytes < 1024 * 1024)
        return `${(bytes / 1024).toFixed(1)} KB`;
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
    const navigateTo = (p) => setSearchParams({ path: p });
    const goUp = () => {
        const parts = path.split("/").filter(Boolean);
        parts.pop();
        navigateTo(parts.length ? "/" + parts.join("/") : "/");
    };
    if (isLoading)
        return _jsx("div", { children: "\u52A0\u8F7D\u4E2D..." });
    return (_jsxs("div", { children: [_jsxs("div", { className: "flex items-center gap-2 mb-4", children: [_jsx("button", { onClick: goUp, className: "text-sm px-2 py-1 border rounded", children: "\u4E0A\u4E00\u7EA7" }), _jsx("span", { className: "text-sm text-gray-500", children: path }), _jsx("button", { onClick: () => setShowMkdir(true), className: "text-sm px-2 py-1 border rounded ml-auto", children: "\u65B0\u5EFA\u6587\u4EF6\u5939" })] }), showMkdir && (_jsxs("div", { className: "flex gap-2 mb-4", children: [_jsx("input", { value: mkdirName, onChange: (e) => setMkdirName(e.target.value), placeholder: "\u6587\u4EF6\u5939\u540D\u79F0", className: "border px-2 py-1 text-sm rounded" }), _jsx("button", { onClick: () => {
                            mkdir.mutate({ path, name: mkdirName });
                            setMkdirName("");
                            setShowMkdir(false);
                        }, className: "text-sm px-2 py-1 bg-blue-600 text-white rounded", children: "\u521B\u5EFA" })] })), _jsxs("table", { className: "w-full text-sm", children: [_jsx("thead", { children: _jsxs("tr", { className: "border-b text-left", children: [_jsx("th", { className: "py-2", children: "\u540D\u79F0" }), _jsx("th", { className: "py-2", children: "\u7C7B\u578B" }), _jsx("th", { className: "py-2", children: "\u5927\u5C0F" }), _jsx("th", { className: "py-2", children: "\u64CD\u4F5C" })] }) }), _jsx("tbody", { children: files?.map((f) => (_jsxs("tr", { className: "border-b hover:bg-gray-50", children: [_jsx("td", { className: "py-2", children: f.is_dir ? (_jsx("button", { onClick: () => navigateTo(f.path), className: "text-blue-600 hover:underline", children: f.name })) : (_jsx("span", { children: f.name })) }), _jsx("td", { className: "py-2 text-gray-500", children: f.file_type }), _jsx("td", { className: "py-2 text-gray-500", children: f.is_dir ? "-" : formatSize(f.size) }), _jsx("td", { className: "py-2", children: _jsx("button", { onClick: () => del.mutate({ path: f.path }), className: "text-red-500 text-xs", children: "\u5220\u9664" }) })] }, f.path))) })] })] }));
}
