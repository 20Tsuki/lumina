import { Routes, Route, Navigate } from "react-router-dom";
import Layout from "./components/Layout";

export default function App() {
  return (
    <Routes>
      <Route path="/" element={<Layout />}>
        <Route index element={<Navigate to="/files" replace />} />
      </Route>
    </Routes>
  );
}
