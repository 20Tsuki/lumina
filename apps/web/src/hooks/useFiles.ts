import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import type { FileEntry } from "@lumina/shared";

export function useFiles(path: string) {
  return useQuery({
    queryKey: ["files", path],
    queryFn: () =>
      api<FileEntry[]>(`/files/list?path=${encodeURIComponent(path)}`),
  });
}

export function useMkdir() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: { path: string; name: string }) =>
      api("/files/mkdir", { method: "POST", body: JSON.stringify(body) }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["files"] }),
  });
}

export function useDelete() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: { path: string }) =>
      api("/files/delete", { method: "POST", body: JSON.stringify(body) }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["files"] }),
  });
}

export function useMove() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: { from: string; to: string }) =>
      api("/files/move", { method: "POST", body: JSON.stringify(body) }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["files"] }),
  });
}
