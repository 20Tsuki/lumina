import { NavLink } from "react-router-dom";
import {
  Folder,
  Film,
  Download,
  Settings,
} from "lucide-react";

const navItems = [
  { to: "/files", label: "文件管理", icon: Folder },
  { to: "/media", label: "影视库", icon: Film },
  { to: "/downloads", label: "下载", icon: Download },
  { to: "/settings", label: "设置", icon: Settings },
];

export default function Sidebar() {
  return (
    <aside className="w-56 h-screen border-r bg-gray-50 flex flex-col py-4">
      <div className="px-4 mb-6">
        <h1 className="text-xl font-bold">Lumina</h1>
        <p className="text-xs text-gray-500">NAS 管理系统</p>
      </div>
      <nav className="flex flex-col gap-1 px-2">
        {navItems.map(({ to, label, icon: Icon }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) =>
              `flex items-center gap-3 px-3 py-2 rounded-md text-sm ${
                isActive ? "bg-gray-200 font-medium" : "hover:bg-gray-100"
              }`
            }
          >
            <Icon className="w-4 h-4" />
            {label}
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}
