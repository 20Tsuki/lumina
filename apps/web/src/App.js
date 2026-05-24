import { jsx as _jsx, Fragment as _Fragment, jsxs as _jsxs } from "react/jsx-runtime";
import { Routes, Route, Navigate } from "react-router-dom";
import { useAuth } from "@/hooks/useAuth";
import Layout from "./components/Layout";
import LoginPage from "./components/LoginPage";
import FileBrowser from "@/pages/FileBrowser";
import MediaLibrary from "@/pages/MediaLibrary";
import PlayerPage from "@/pages/PlayerPage";
import Downloads from "@/pages/Downloads";
import Settings from "@/pages/Settings";
function ProtectedRoute({ children }) {
    const { user, loading } = useAuth();
    if (loading) {
        return (_jsx("div", { className: "flex h-screen items-center justify-center", children: _jsx("p", { className: "text-gray-500", children: "\u52A0\u8F7D\u4E2D..." }) }));
    }
    if (!user) {
        return _jsx(Navigate, { to: "/login", replace: true });
    }
    return _jsx(_Fragment, { children: children });
}
export default function App() {
    return (_jsxs(Routes, { children: [_jsx(Route, { path: "/login", element: _jsx(LoginPage, {}) }), _jsxs(Route, { path: "/", element: _jsx(ProtectedRoute, { children: _jsx(Layout, {}) }), children: [_jsx(Route, { index: true, element: _jsx(Navigate, { to: "/files", replace: true }) }), _jsx(Route, { path: "files", element: _jsx(FileBrowser, {}) }), _jsx(Route, { path: "media", element: _jsx(MediaLibrary, {}) }), _jsx(Route, { path: "downloads", element: _jsx(Downloads, {}) }), _jsx(Route, { path: "settings", element: _jsx(Settings, {}) })] }), _jsx(Route, { path: "player/:id", element: _jsx(ProtectedRoute, { children: _jsx(PlayerPage, {}) }) })] }));
}
