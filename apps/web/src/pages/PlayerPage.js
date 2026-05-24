import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useParams, useNavigate } from "react-router-dom";
import { useMediaDetail } from "@/hooks/useMedia";
export default function PlayerPage() {
    const { id } = useParams();
    const { data: media, isLoading } = useMediaDetail(id ? parseInt(id) : null);
    const navigate = useNavigate();
    if (isLoading)
        return _jsx("div", { children: "\u52A0\u8F7D\u4E2D..." });
    if (!media)
        return _jsx("div", { children: "\u672A\u627E\u5230\u5A92\u4F53" });
    return (_jsxs("div", { className: "fixed inset-0 bg-black z-50", children: [_jsx("button", { onClick: () => navigate(-1), className: "absolute top-4 left-4 text-white z-10 text-sm", children: "\u8FD4\u56DE" }), _jsx("video", { src: `/api/stream/${media.id}/file`, controls: true, autoPlay: true, className: "w-full h-full" })] }));
}
