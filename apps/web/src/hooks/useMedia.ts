import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import type { IndexedFile, PaginatedResponse } from "@lumina/shared";

export function useMovies(page: number) {
  return useQuery({
    queryKey: ["media", "movies", page],
    queryFn: () =>
      api<PaginatedResponse<IndexedFile>>(`/media/movies?page=${page}&size=20`),
  });
}

export function useMediaDetail(id: number | null) {
  return useQuery({
    queryKey: ["media", "detail", id],
    queryFn: () => api<IndexedFile>(`/media/${id}`),
    enabled: id != null,
  });
}

export function useSearch(query: string) {
  return useQuery({
    queryKey: ["media", "search", query],
    queryFn: () =>
      api<IndexedFile[]>(`/media/search?q=${encodeURIComponent(query)}`),
    enabled: query.length > 0,
  });
}
