import { BrowserRouter as Router, Routes, Route, Link, Outlet } from 'react-router-dom';
import { LayoutDashboard, Cpu, History, Settings, UserCheck, UserCircle } from 'lucide-react';
import DashboardPage from './pages/DashboardPage';
import AgentsPage from './pages/AgentsPage';
import DevicesPage from './pages/DevicesPage';
import WorkspaceSettingsPage from './pages/WorkspaceSettingsPage';
import ProfilePage from './pages/ProfilePage';
import WorkspaceSwitcher from './components/WorkspaceSwitcher';

function Layout() {
  return (
    <div className="flex min-h-screen bg-slate-50 text-slate-900 font-inter">
      {/* Sidebar */}
      <aside className="w-64 bg-white border-r border-slate-200 flex flex-col sticky top-0 h-screen">
        <div className="p-6">
          <div className="flex items-center gap-3 px-2 mb-8">
            <div className="bg-blue-600 p-2 rounded-xl shadow-lg shadow-blue-200">
              <Cpu className="text-white w-6 h-6" />
            </div>
            <span className="font-bold text-xl tracking-tight">LeeAttend</span>
          </div>

          <WorkspaceSwitcher />
        </div>

        <nav className="flex-1 px-4 space-y-1">
          <Link to="/" className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group">
            <LayoutDashboard className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Bảng điều khiển</span>
          </Link>
          <Link to="/agents" className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group">
            <UserCheck className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Quản lý Agent</span>
          </Link>
          <Link to="/devices" className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group">
            <Cpu className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Thiết bị</span>
          </Link>
          <Link to="/workspace/settings" className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group">
            <Settings className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Cài đặt</span>
          </Link>
          <Link to="/profile" className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group">
            <UserCircle className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Hồ sơ</span>
          </Link>
          <Link to="/logs" className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group">
            <History className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Nhật ký</span>
          </Link>
        </nav>

        <div className="p-6 border-t border-slate-100">
          <Link to="/settings" className="flex items-center gap-3 px-4 py-2 text-slate-400 hover:text-slate-600 transition-colors">
            <Settings className="w-5 h-5" />
            <span className="text-sm font-medium">Cài đặt</span>
          </Link>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-y-auto">
        <Outlet />
      </main>
    </div>
  );
}

function App() {
  return (
    <Router>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<DashboardPage />} />
          <Route path="agents" element={<AgentsPage />} />
          <Route path="devices" element={<DevicesPage />} />
          <Route path="workspace/settings" element={<WorkspaceSettingsPage />} />
          <Route path="profile" element={<ProfilePage />} />
          {/* Fallback for routes not yet implemented */}
          <Route path="logs" element={<div className="p-8">Tính năng Nhật ký đang được phát triển.</div>} />
          <Route path="settings" element={<div className="p-8">Tính năng Cài đặt đang được phát triển.</div>} />
        </Route>
      </Routes>
    </Router>
  );
}

export default App;
