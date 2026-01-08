import {
  BrowserRouter as Router,
  Routes,
  Route,
  Link,
  Outlet,
} from "react-router-dom";
import {
  LayoutDashboard,
  Cpu,
  History,
  Settings,
  UserCheck,
  UserCircle,
  Users,
  Clock,
  CalendarDays,
  FileBarChart,
  UserRound,
} from "lucide-react"; // Added new icons
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import DashboardPage from "./pages/DashboardPage";
import AgentsPage from "./pages/AgentsPage";
import DevicesPage from "./pages/DevicesPage";
import MembersPage from "./pages/MembersPage";
import WorkspaceSettingsPage from "./pages/WorkspaceSettingsPage";
import ProfilePage from "./pages/ProfilePage";
import LoginPage from "./pages/LoginPage";
import RegisterPage from "./pages/RegisterPage";
import OnboardingPage from "./pages/OnboardingPage";
import EmployeesPage from "./pages/EmployeesPage"; // New pages
import TimeTrackingPage from "./pages/TimeTrackingPage";
import LeaveRequestsPage from "./pages/LeaveRequestsPage";
import ReportsPage from "./pages/ReportsPage";
import WorkspaceSwitcher from "./components/WorkspaceSwitcher";
import AuthGuard from "./components/AuthGuard";
import WorkspaceGuard from "./components/WorkspaceGuard";
import { LogOut } from "lucide-react";

console.log("🚀 [LeeAttend] Bundle initialized at:", new Date().toISOString());

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 1000 * 60 * 5, // 5 minutes
    },
  },
});

function Layout() {
  const handleLogout = () => {
    localStorage.clear();
    window.location.href = "/login";
  };

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

        <nav className="flex-1 px-4 space-y-1 overflow-y-auto">
          <Link
            to="/"
            className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group"
          >
            <LayoutDashboard className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Bảng điều khiển</span>
          </Link>
          
          <div className="pt-4 pb-2 px-4 text-xs font-bold text-slate-400 uppercase tracking-wider">
            Quản lý tổ chức
          </div>
          <Link
            to="/members"
            className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group"
          >
            <Users className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Thành viên</span>
          </Link>
          <Link
            to="/employees"
            className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group"
          >
            <UserRound className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Nhân viên</span>
          </Link>

          <div className="pt-4 pb-2 px-4 text-xs font-bold text-slate-400 uppercase tracking-wider">
            Chấm công & Nghỉ phép
          </div>
          <Link
            to="/time-tracking"
            className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group"
          >
            <Clock className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Chấm công</span>
          </Link>
          <Link
            to="/leave-requests"
            className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group"
          >
            <CalendarDays className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Nghỉ phép</span>
          </Link>
          <Link
            to="/reports"
            className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group"
          >
            <FileBarChart className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Báo cáo</span>
          </Link>

          <div className="pt-4 pb-2 px-4 text-xs font-bold text-slate-400 uppercase tracking-wider">
            Hệ thống
          </div>
          <Link
            to="/agents"
            className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group"
          >
            <UserCheck className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Quản lý Agent</span>
          </Link>
          <Link
            to="/devices"
            className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group"
          >
            <Cpu className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Thiết bị</span>
          </Link>
          <Link
            to="/workspace/settings"
            className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group"
          >
            <Settings className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Cài đặt</span>
          </Link>
        </nav>

        <div className="p-4 border-t border-slate-100">
          <Link
            to="/profile"
            className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group mb-1"
          >
            <UserCircle className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Hồ sơ</span>
          </Link>
          <button
            onClick={handleLogout}
            className="w-full flex items-center gap-3 px-4 py-3 text-red-600 hover:bg-red-50 rounded-2xl transition-all group"
          >
            <LogOut className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold">Đăng xuất</span>
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-y-auto">
        <div className="p-8 max-w-7xl mx-auto">
          <Outlet />
        </div>
      </main>
    </div>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <Router>
        <Routes>
          {/* Public routes */}
          <Route path="/login" element={<LoginPage />} />
          <Route path="/register" element={<RegisterPage />} />

          {/* Protected routes */}
          <Route element={<AuthGuard />}>
            <Route path="/onboarding" element={<OnboardingPage />} />
            <Route element={<WorkspaceGuard />}>
              <Route element={<Layout />}>
                <Route path="/" element={<DashboardPage />} />
                <Route path="/agents" element={<AgentsPage />} />
                <Route path="/devices" element={<DevicesPage />} />
                <Route path="/members" element={<MembersPage />} />
                <Route path="/employees" element={<EmployeesPage />} />
                <Route path="/time-tracking" element={<TimeTrackingPage />} />
                <Route path="/leave-requests" element={<LeaveRequestsPage />} />
                <Route path="/reports" element={<ReportsPage />} />
                <Route
                  path="/workspace/settings"
                  element={<WorkspaceSettingsPage />} 
                />
                <Route path="/profile" element={<ProfilePage />} />
              </Route>
            </Route>
          </Route>
        </Routes>
      </Router>
    </QueryClientProvider>
  );
}

export default App;
