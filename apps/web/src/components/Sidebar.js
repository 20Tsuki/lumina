import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { NavLink } from "react-router-dom";
import { Folder, Film, Download, Settings, } from "lucide-react";
const navItems = [
    { to: "/files", label: "文件管理", icon: Folder },
    { to: "/media", label: "影视库", icon: Film },
    { to: "/downloads", label: "下载", icon: Download },
    { to: "/settings", label: "设置", icon: Settings },
];
export default function Sidebar() {
    return (_jsxs("aside", { className: "w-56 h-screen border-r bg-gray-50 flex flex-col py-4", children: [_jsxs("div", { className: "px-4 mb-6", children: [_jsx("h1", { className: "text-xl font-bold", children: "Lumina" }), _jsx("p", { className: "text-xs text-gray-500", children: "NAS \u7BA1\u7406\u7CFB\u7EDF" })] }), _jsx("nav", { className: "flex flex-col gap-1 px-2", children: navItems.map(({ to, label, icon: Icon }) => (_jsxs(NavLink, { to: to, className: ({ isActive }) => `flex items-center gap-3 px-3 py-2 rounded-md text-sm ${isActive ? "bg-gray-200 font-medium" : "hover:bg-gray-100"}`, children: [_jsx(Icon, { className: "w-4 h-4" }), label] }, to))) })] }));
}
