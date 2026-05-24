import { Routes, Route, Navigate } from "react-router-dom";
import { useAuth } from "@/hooks/useAuth";
import Layout from "./components/Layout";
import LoginPage from "./components/LoginPage";

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
        <Route path="files" element={<div>Files (coming soon)</div>} />
        <Route path="media" element={<div>Media (coming soon)</div>} />
        <Route path="downloads" element={<div>Downloads (coming soon)</div>} />
        <Route path="settings" element={<div>Settings (coming soon)</div>} />
      </Route>
    </Routes>
  );
}
