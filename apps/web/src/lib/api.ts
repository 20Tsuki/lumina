const BASE = "/api";

let token: string | null = localStorage.getItem("lumina_token");

export function setToken(t: string | null) {
  token = t;
  if (t) localStorage.setItem("lumina_token", t);
  else localStorage.removeItem("lumina_token");
}

export function getToken(): string | null {
  return token;
}

export async function api<T>(path: string, options: RequestInit = {}): Promise<T> {
  const headers: Record<string, string> = {
    ...(options.headers as Record<string, string>),
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
