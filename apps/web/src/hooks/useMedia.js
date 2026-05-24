import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
export function useMovies(page) {
    return useQuery({
        queryKey: ["media", "movies", page],
        queryFn: () => api(`/media/movies?page=${page}&size=20`),
    });
}
export function useMediaDetail(id) {
    return useQuery({
        queryKey: ["media", "detail", id],
        queryFn: () => api(`/media/${id}`),
        enabled: id != null,
    });
}
export function useSearch(query) {
    return useQuery({
        queryKey: ["media", "search", query],
        queryFn: () => api(`/media/search?q=${encodeURIComponent(query)}`),
        enabled: query.length > 0,
    });
}
