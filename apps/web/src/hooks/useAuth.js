import { jsx as _jsx } from "react/jsx-runtime";
import { createContext, useContext, useState, useEffect, useCallback, } from "react";
import { api, setToken as saveToken } from "@/lib/api";
const AuthContext = createContext(null);
export function AuthProvider({ children }) {
    const [user, setUser] = useState(null);
    const [loading, setLoading] = useState(true);
    useEffect(() => {
        const stored = localStorage.getItem("lumina_token");
        if (stored) {
            saveToken(stored);
            api("/auth/me")
                .then(setUser)
                .catch(() => saveToken(null))
                .finally(() => setLoading(false));
        }
        else {
            setLoading(false);
        }
    }, []);
    const login = useCallback(async (username, password) => {
        const data = await api("/auth/login", {
            method: "POST",
            body: JSON.stringify({ username, password }),
        });
        saveToken(data.token);
        setUser(data.user);
    }, []);
    const logout = useCallback(() => {
        saveToken(null);
        setUser(null);
    }, []);
    return (_jsx(AuthContext.Provider, { value: { user, loading, login, logout }, children: children }));
}
export function useAuth() {
    return useContext(AuthContext);
}
