import { Routes, Route, Navigate } from "react-router-dom";
import { useAuth } from "@/hooks/useAuth";
import Layout from "./components/Layout";
import LoginPage from "./components/LoginPage";
import FileBrowser from "@/pages/FileBrowser";
import MediaLibrary from "@/pages/MediaLibrary";
import PlayerPage from "@/pages/PlayerPage";
import Downloads from "@/pages/Downloads";
import Settings from "@/pages/Settings";

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { user, loading } = useAuth();

  if (loading) {
    return (
      <div className="flex h-screen items-center justify-center">
        <p className="text-gray-500">加载中...</p>
      </div>
    );
  }

  if (!user) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
}

export default function App() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route
        path="/"
        element={
          <ProtectedRoute>
            <Layout />
          </ProtectedRoute>
        }
      >
        <Route index element={<Navigate to="/files" replace />} />
        <Route path="files" element={<FileBrowser />} />
        <Route path="media" element={<MediaLibrary />} />
        <Route path="downloads" element={<Downloads />} />
        <Route path="settings" element={<Settings />} />
      </Route>
      <Route
        path="player/:id"
        element={
          <ProtectedRoute>
            <PlayerPage />
          </ProtectedRoute>
        }
      />
    </Routes>
  );
}
