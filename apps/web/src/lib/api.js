const BASE = "/api";
let token = localStorage.getItem("lumina_token");
export function setToken(t) {
    token = t;
    if (t)
        localStorage.setItem("lumina_token", t);
    else
        localStorage.removeItem("lumina_token");
}
export function getToken() {
    return token;
}
export async function api(path, options = {}) {
    const headers = {
        ...options.headers,
    };
    if (token) {
        headers["Authorization"] = `Bearer ${token}`;
    }
    if (options.body && typeof options.body === "string") {
        headers["Content-Type"] = "application/json";
    }
    const res = await fetch(`${BASE}${path}`, { ...options, headers });
    if (!res.ok) {
        const err = await res.json().catch(() => ({ error: { message: "request failed" } }));
        throw new Error(err.error?.message ?? "request failed");
    }
    return res.json();
}
