import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
export function useFiles(path) {
    return useQuery({
        queryKey: ["files", path],
        queryFn: () => api(`/files/list?path=${encodeURIComponent(path)}`),
    });
}
export function useMkdir() {
    const qc = useQueryClient();
    return useMutation({
        mutationFn: (body) => api("/files/mkdir", { method: "POST", body: JSON.stringify(body) }),
        onSuccess: () => qc.invalidateQueries({ queryKey: ["files"] }),
    });
}
export function useDelete() {
    const qc = useQueryClient();
    return useMutation({
        mutationFn: (body) => api("/files/delete", { method: "POST", body: JSON.stringify(body) }),
        onSuccess: () => qc.invalidateQueries({ queryKey: ["files"] }),
    });
}
export function useMove() {
    const qc = useQueryClient();
    return useMutation({
        mutationFn: (body) => api("/files/move", { method: "POST", body: JSON.stringify(body) }),
        onSuccess: () => qc.invalidateQueries({ queryKey: ["files"] }),
    });
}
