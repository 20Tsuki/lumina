import { useParams, useNavigate } from "react-router-dom";
import { useMediaDetail } from "@/hooks/useMedia";

export default function PlayerPage() {
  const { id } = useParams();
  const { data: media, isLoading } = useMediaDetail(id ? parseInt(id) : null);
  const navigate = useNavigate();

  if (isLoading) return <div>加载中...</div>;
  if (!media) return <div>未找到媒体</div>;

  return (
    <div className="fixed inset-0 bg-black z-50">
      <button
        onClick={() => navigate(-1)}
        className="absolute top-4 left-4 text-white z-10 text-sm"
      >
        返回
      </button>
      <video
        src={`/api/stream/${media.id}/file`}
        controls
        autoPlay
        className="w-full h-full"
      />
    </div>
  );
}
