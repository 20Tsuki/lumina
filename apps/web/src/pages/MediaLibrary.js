import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useMovies, useSearch } from "@/hooks/useMedia";
export default function MediaLibrary() {
    const [page, setPage] = useState(1);
    const [search, setSearch] = useState("");
    const { data, isLoading } = useMovies(page);
    const { data: searchResults } = useSearch(search);
    const navigate = useNavigate();
    const items = search ? searchResults : data?.items;
    if (isLoading)
        return _jsx("div", { children: "\u52A0\u8F7D\u4E2D..." });
    return (_jsxs("div", { children: [_jsxs("div", { className: "flex items-center gap-4 mb-4", children: [_jsx("h2", { className: "text-lg font-bold", children: "\u5F71\u89C6\u5E93" }), _jsx("input", { type: "text", placeholder: "\u641C\u7D22...", value: search, onChange: (e) => setSearch(e.target.value), className: "border rounded px-3 py-1 text-sm" })] }), _jsx("div", { className: "grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-4", children: items?.map((movie) => (_jsxs("div", { className: "cursor-pointer", onClick: () => navigate(`/player/${movie.id}`), children: [_jsx("div", { className: "aspect-[2/3] bg-gray-200 rounded flex items-center justify-center text-gray-400", children: movie.thumb_path ? (_jsx("img", { src: movie.thumb_path, alt: movie.title, className: "w-full h-full object-cover rounded" })) : ("无海报") }), _jsx("p", { className: "text-sm mt-1 truncate", children: movie.title })] }, movie.id))) }), !search && data?.next_cursor && (_jsxs("div", { className: "flex justify-center mt-4 gap-2", children: [_jsx("button", { onClick: () => setPage((p) => Math.max(1, p - 1)), disabled: page === 1, className: "px-3 py-1 border rounded text-sm", children: "\u4E0A\u4E00\u9875" }), _jsx("button", { onClick: () => setPage((p) => p + 1), className: "px-3 py-1 border rounded text-sm", children: "\u4E0B\u4E00\u9875" })] }))] }));
}
