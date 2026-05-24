import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useState } from "react";
import { useAuth } from "@/hooks/useAuth";
export default function LoginPage() {
    const { login } = useAuth();
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");
    const [error, setError] = useState("");
    const handleSubmit = async (e) => {
        e.preventDefault();
        setError("");
        try {
            await login(username, password);
        }
        catch (err) {
            setError(err instanceof Error ? err.message : "登录失败");
        }
    };
    return (_jsx("div", { className: "flex h-screen items-center justify-center bg-gray-100", children: _jsxs("form", { onSubmit: handleSubmit, className: "bg-white p-8 rounded-lg shadow-md w-80", children: [_jsx("h1", { className: "text-xl font-bold mb-4", children: "Lumina NAS" }), error && _jsx("p", { className: "text-red-500 text-sm mb-3", children: error }), _jsx("input", { type: "text", placeholder: "\u7528\u6237\u540D", value: username, onChange: (e) => setUsername(e.target.value), className: "w-full border rounded px-3 py-2 mb-3 text-sm" }), _jsx("input", { type: "password", placeholder: "\u5BC6\u7801", value: password, onChange: (e) => setPassword(e.target.value), className: "w-full border rounded px-3 py-2 mb-4 text-sm" }), _jsx("button", { type: "submit", className: "w-full bg-blue-600 text-white py-2 rounded text-sm", children: "\u767B\u5F55" })] }) }));
}
