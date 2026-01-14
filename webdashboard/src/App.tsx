import { useState, useEffect } from "react";
import {
  BrowserRouter as Router,
  Routes,
  Route,
  Link,
  Outlet,
  useLocation,
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
  Banknote,
  CalendarRange,
  Building2,
  Menu,
  X,
  LogOut,
  ListOrdered,
  ChevronDown,
  ChevronRight,
  ChevronRight as BreadcrumbSeparator,
} from "lucide-react"; 
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
import EmployeesPage from "./pages/EmployeesPage"; 
import DepartmentsPage from "./pages/DepartmentsPage";
import TimeTrackingPage from "./pages/TimeTrackingPage";
import LeaveRequestsPage from "./pages/LeaveRequestsPage";
import ReportsPage from "./pages/ReportsPage";
import PayrollPage from "./pages/PayrollPage";
import ShiftsPage from "./pages/ShiftsPage";
import WorkspaceSwitcher from "./components/WorkspaceSwitcher";
import AuthGuard from "./components/AuthGuard";
import WorkspaceGuard from "./components/WorkspaceGuard";
import AttendanceAdjustmentsPage from "./pages/AttendanceAdjustmentsPage";
import AttendanceLogsPage from "./pages/AttendanceLogsPage";
import { useWorkspaces } from "./hooks/use-workspace";
import { getCurrentWorkspaceId } from "./lib/api";
import { useTheme } from "./hooks/use-theme";

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
  useTheme();
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);
  const [showSwitcher, setShowSwitcher] = useState(false);
  const [openSections, setOpenSections] = useState<Record<string, boolean>>({
    org: false,
    time: false,
    payroll: false,
    system: false,
  });
  
  const { workspaces } = useWorkspaces();
  const currentWsId = getCurrentWorkspaceId();
  const currentWorkspace = workspaces.find(ws => ws.id === currentWsId);
  
  const location = useLocation();

  const toggleSection = (section: string) => {
    setOpenSections(prev => ({ 
      org: false, 
      time: false, 
      payroll: false, 
      system: false, 
      [section]: !prev[section] 
    }));
  };

  const getPageName = (path: string) => {
    const map: Record<string, string> = {
      "/": "Bảng điều khiển",
      "/members": "Thành viên",
      "/employees": "Nhân viên",
      "/departments": "Phòng ban",
      "/time-tracking": "Chấm công",
      "/attendance-logs": "Nhật ký",
      "/leave-requests": "Nghỉ phép",
      "/attendance-adjustments": "Xác nhận công",
      "/reports": "Báo cáo",
      "/shifts": "Lịch trình & Ca",
      "/payroll": "Tính lương",
      "/agents": "Agents",
      "/devices": "Thiết bị",
      "/workspace/settings": "Cài đặt",
      "/profile": "Hồ sơ",
    };
    return map[path] || path.split('/').pop()?.replace(/-/g, ' ') || "";
  };

  const handleLogout = () => {
    localStorage.clear();
    window.location.href = "/login";
  };

  // Auto-expand section if a child is active
  useEffect(() => {
    const path = location.pathname;
    if (["/members", "/employees", "/departments"].includes(path)) setOpenSections(p => ({...p, org: true}));
    if (["/time-tracking", "/attendance-logs", "/leave-requests", "/attendance-adjustments", "/reports"].includes(path)) setOpenSections(p => ({...p, time: true}));
    if (["/shifts", "/payroll"].includes(path)) setOpenSections(p => ({...p, payroll: true}));
    if (["/agents", "/devices", "/workspace/settings"].includes(path)) setOpenSections(p => ({...p, system: true}));
  }, [location.pathname]);

  // Close sidebar when route changes on mobile
  useEffect(() => {
    setIsSidebarOpen(false);
  }, [location]);

  return (
    <div className="flex min-h-screen bg-[#fcfcfd] text-slate-900 font-sans relative overflow-x-hidden">
      {/* Mobile Overlay */}
      {isSidebarOpen && (
        <div 
          className="fixed inset-0 bg-slate-900/50 backdrop-blur-sm z-40 lg:hidden transition-opacity"
          onClick={() => setIsSidebarOpen(false)}
        />
      )}

      {/* Sidebar */}
      <aside className={`
        fixed inset-y-0 left-0 w-72 bg-white border-r border-slate-100 flex flex-col z-50 transition-transform duration-300 ease-in-out
        lg:translate-x-0 lg:static lg:h-screen
        ${isSidebarOpen ? "translate-x-0" : "-translate-x-full"}
      `}>
        <div className="p-8 pb-4">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-3 px-2">
              <div className="bg-primary-600 p-2 rounded-xl shadow-lg shadow-primary-200 rotate-3">
                <Cpu className="text-white w-5 h-5" />
              </div>
              <div className="flex flex-col">
                <span className="font-bold text-lg tracking-tight text-slate-900 leading-none">LeeAttend</span>
                <span className="text-[9px] font-semibold text-slate-400 uppercase tracking-[0.2em] mt-1">HRM System</span>
              </div>
            </div>
            <button 
              className="lg:hidden p-2 text-slate-400 hover:text-slate-600"
              onClick={() => setIsSidebarOpen(false)}
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        <nav className="flex-1 px-6 space-y-1 overflow-y-auto scrollbar-hide pb-10 scroll-smooth">
          <Link
            to="/"
            className={`flex items-center gap-3 px-4 py-3 rounded-2xl transition-all duration-300 group relative ${
              location.pathname === "/" 
                ? "bg-primary-50 text-primary-600 shadow-sm shadow-primary-100/50" 
                : "text-slate-500 hover:bg-primary-50 hover:text-primary-600"
            }`}
          >
            <LayoutDashboard className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold text-sm">Bảng điều khiển</span>
          </Link>
          
          {/* Section: Tổ chức */}
          <div className="pt-4">
            <button 
              onClick={() => toggleSection('org')}
              className="w-full flex items-center justify-between px-4 py-2 text-[11px] font-bold text-slate-400 uppercase tracking-[0.15em] hover:text-slate-600 transition-colors"
            >
              <span>Tổ chức</span>
              {openSections.org ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronRight className="w-3.5 h-3.5" />}
            </button>
            <div className={`space-y-1 mt-1 overflow-hidden transition-all duration-300 ${openSections.org ? "max-h-96 opacity-100" : "max-h-0 opacity-0"}`}>
              <Link
                to="/members"
                className={`flex items-center gap-3 px-4 py-2.5 rounded-xl transition-all group ${
                  location.pathname === "/members" ? "bg-primary-50 text-primary-600 shadow-sm shadow-primary-100/50" : "text-slate-500 hover:bg-primary-50 hover:text-primary-600"
                }`}
              >
                <Users className="w-4 h-4 group-hover:scale-110 transition-transform" />
                <span className="font-semibold text-sm">Thành viên</span>
              </Link>
              <Link
                to="/employees"
                className={`flex items-center gap-3 px-4 py-2.5 rounded-xl transition-all group ${
                  location.pathname === "/employees" ? "bg-primary-50 text-primary-600 shadow-sm shadow-primary-100/50" : "text-slate-500 hover:bg-primary-50 hover:text-primary-600"
                }`}
              >
                <UserRound className="w-4 h-4 group-hover:scale-110 transition-transform" />
                <span className="font-semibold text-sm">Nhân viên</span>
              </Link>
              <Link
                to="/departments"
                className={`flex items-center gap-3 px-4 py-2.5 rounded-xl transition-all group ${
                  location.pathname === "/departments" ? "bg-primary-50 text-primary-600 shadow-sm shadow-primary-100/50" : "text-slate-500 hover:bg-primary-50 hover:text-primary-600"
                }`}
              >
                <Building2 className="w-4 h-4 group-hover:scale-110 transition-transform" />
                <span className="font-semibold text-sm">Phòng ban</span>
              </Link>
            </div>
          </div>

          {/* Section: Thời gian & Nghỉ phép */}
          <div className="pt-2">
            <button 
              onClick={() => toggleSection('time')}
              className="w-full flex items-center justify-between px-4 py-2 text-[11px] font-bold text-slate-400 uppercase tracking-[0.15em] hover:text-slate-600 transition-colors"
            >
              <span>Thời gian & Nghỉ phép</span>
              {openSections.time ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronRight className="w-3.5 h-3.5" />}
            </button>
            <div className={`space-y-1 mt-1 overflow-hidden transition-all duration-300 ${openSections.time ? "max-h-[500px] opacity-100" : "max-h-0 opacity-0"}`}>
              <Link
                to="/time-tracking"
                className={`flex items-center gap-3 px-4 py-2.5 rounded-xl transition-all group ${
                  location.pathname === "/time-tracking" ? "bg-primary-50 text-primary-600 shadow-sm shadow-primary-100/50" : "text-slate-500 hover:bg-primary-50 hover:text-primary-600"
                }`}
              >
                <Clock className="w-4 h-4 group-hover:scale-110 transition-transform" />
                <span className="font-semibold text-sm">Chấm công</span>
              </Link>
              <Link
                to="/attendance-logs"
                className={`flex items-center gap-3 px-4 py-2.5 rounded-xl transition-all group ${
                  location.pathname === "/attendance-logs" ? "bg-primary-50 text-primary-600 shadow-sm shadow-primary-100/50" : "text-slate-500 hover:bg-primary-50 hover:text-primary-600"
                }`}
              >
                <ListOrdered className="w-4 h-4 group-hover:scale-110 transition-transform" />
                <span className="font-semibold text-sm">Nhật ký</span>
              </Link>
              <Link
                to="/leave-requests"
                className={`flex items-center gap-3 px-4 py-2.5 rounded-xl transition-all group ${
                  location.pathname === "/leave-requests" ? "bg-primary-50 text-primary-600 shadow-sm shadow-primary-100/50" : "text-slate-500 hover:bg-primary-50 hover:text-primary-600"
                }`}
              >
                <CalendarDays className="w-4 h-4 group-hover:scale-110 transition-transform" />
                <span className="font-semibold text-sm">Nghỉ phép</span>
              </Link>
              <Link
                to="/attendance-adjustments"
                className={`flex items-center gap-3 px-4 py-2.5 rounded-xl transition-all group ${
                  location.pathname === "/attendance-adjustments" ? "bg-primary-50 text-primary-600 shadow-sm shadow-primary-100/50" : "text-slate-500 hover:bg-primary-50 hover:text-primary-600"
                }`}
              >
                <History className="w-4 h-4 group-hover:scale-110 transition-transform" />
                <span className="font-semibold text-sm">Xác nhận công</span>
              </Link>
              <Link
                to="/reports"
                className={`flex items-center gap-3 px-4 py-2.5 rounded-xl transition-all group ${
                  location.pathname === "/reports" ? "bg-primary-50 text-primary-600 shadow-sm shadow-primary-100/50" : "text-slate-500 hover:bg-primary-50 hover:text-primary-600"
                }`}
              >
                <FileBarChart className="w-4 h-4 group-hover:scale-110 transition-transform" />
                <span className="font-semibold text-sm">Báo cáo</span>
              </Link>
            </div>
          </div>

          {/* Section: Lương & Ca làm */}
          <div className="pt-2">
            <button 
              onClick={() => toggleSection('payroll')}
              className="w-full flex items-center justify-between px-4 py-2 text-[11px] font-bold text-slate-400 uppercase tracking-[0.15em] hover:text-slate-600 transition-colors"
            >
              <span>Lương & Ca làm</span>
              {openSections.payroll ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronRight className="w-3.5 h-3.5" />}
            </button>
            <div className={`space-y-1 mt-1 overflow-hidden transition-all duration-300 ${openSections.payroll ? "max-h-64 opacity-100" : "max-h-0 opacity-0"}`}>
              <Link
                to="/shifts"
                className={`flex items-center gap-3 px-4 py-2.5 rounded-xl transition-all group ${
                  location.pathname === "/shifts" ? "bg-primary-50 text-primary-600 shadow-sm shadow-primary-100/50" : "text-slate-500 hover:bg-primary-50 hover:text-primary-600"
                }`}
              >
                <CalendarRange className="w-4 h-4 group-hover:scale-110 transition-transform" />
                <span className="font-semibold text-sm">Lịch trình & Ca</span>
              </Link>
              <Link
                to="/payroll"
                className={`flex items-center gap-3 px-4 py-2.5 rounded-xl transition-all group ${
                  location.pathname === "/payroll" ? "bg-primary-50 text-primary-600 shadow-sm shadow-primary-100/50" : "text-slate-500 hover:bg-primary-50 hover:text-primary-600"
                }`}
              >
                <Banknote className="w-4 h-4 group-hover:scale-110 transition-transform" />
                <span className="font-semibold text-sm">Tính lương</span>
              </Link>
            </div>
          </div>

          {/* Section: Hệ thống */}
          <div className="pt-2">
            <button 
              onClick={() => toggleSection('system')}
              className="w-full flex items-center justify-between px-4 py-2 text-[11px] font-bold text-slate-400 uppercase tracking-[0.15em] hover:text-slate-600 transition-colors"
            >
              <span>Hệ thống</span>
              {openSections.system ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronRight className="w-3.5 h-3.5" />}
            </button>
            <div className={`space-y-1 mt-1 overflow-hidden transition-all duration-300 ${openSections.system ? "max-h-64 opacity-100" : "max-h-0 opacity-0"}`}>
              <Link
                to="/agents"
                className={`flex items-center gap-3 px-4 py-2.5 rounded-xl transition-all group ${
                  location.pathname === "/agents" ? "bg-primary-50 text-primary-600 shadow-sm shadow-primary-100/50" : "text-slate-500 hover:bg-primary-50 hover:text-primary-600"
                }`}
              >
                <UserCheck className="w-4 h-4 group-hover:scale-110 transition-transform" />
                <span className="font-semibold text-sm">Agents</span>
              </Link>
              <Link
                to="/devices"
                className={`flex items-center gap-3 px-4 py-2.5 rounded-xl transition-all group ${
                  location.pathname === "/devices" ? "bg-primary-50 text-primary-600 shadow-sm shadow-primary-100/50" : "text-slate-500 hover:bg-primary-50 hover:text-primary-600"
                }`}
              >
                <Cpu className="w-4 h-4 group-hover:scale-110 transition-transform" />
                <span className="font-semibold text-sm">Thiết bị</span>
              </Link>
              <Link
                to="/workspace/settings"
                className={`flex items-center gap-3 px-4 py-2.5 rounded-xl transition-all group ${
                  location.pathname === "/workspace/settings" ? "bg-primary-50 text-primary-600 shadow-sm shadow-primary-100/50" : "text-slate-500 hover:bg-primary-50 hover:text-primary-600"
                }`}
              >
                <Settings className="w-4 h-4 group-hover:scale-110 transition-transform" />
                <span className="font-semibold text-sm">Cài đặt</span>
              </Link>
            </div>
          </div>
        </nav>

        <div className="p-6 border-t border-slate-50 space-y-1">
          <Link
            to="/profile"
            className={`flex items-center gap-3 px-4 py-3 rounded-2xl transition-all group ${
              location.pathname === "/profile" 
                ? "bg-primary-50 text-primary-600" 
                : "text-slate-500 hover:bg-primary-50 hover:text-primary-600"
            }`}
          >
            <UserCircle className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span className="font-semibold text-sm">Hồ sơ</span>
          </Link>
          <button
            onClick={() => setShowSwitcher(true)}
            className="w-full flex items-center gap-3 px-4 py-3 text-slate-500 hover:bg-primary-50 hover:text-primary-600 rounded-2xl transition-all group font-semibold text-sm"
          >
            <Building2 className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span>Chuyển cơ sở</span>
          </button>
          <button
            onClick={handleLogout}
            className="w-full flex items-center gap-3 px-4 py-3 text-red-500 hover:bg-red-50 rounded-2xl transition-all group font-semibold text-sm"
          >
            <LogOut className="w-5 h-5 group-hover:scale-110 transition-transform" />
            <span>Đăng xuất</span>
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <div className="flex-1 flex flex-col min-w-0 h-screen overflow-hidden relative">
        <header className="h-16 border-b border-slate-100 bg-white/80 backdrop-blur-md flex items-center justify-between px-4 lg:px-8 gap-4 flex-shrink-0 z-30">
          <div className="flex items-center gap-4">
            <button 
              className="lg:hidden p-2 text-slate-500 hover:bg-slate-50 rounded-xl"
              onClick={() => setIsSidebarOpen(true)}
            >
              <Menu className="w-6 h-6" />
            </button>
            
            {/* Breadcrumbs */}
            <nav className="flex items-center gap-2 text-sm font-medium">
              <span className="hidden sm:inline text-slate-400 font-semibold">{currentWorkspace?.name || "Workspace"}</span>
              <BreadcrumbSeparator className="w-3.5 h-3.5 text-slate-300" />
              <span className="text-slate-900 font-bold whitespace-nowrap">
                {getPageName(location.pathname)}
              </span>
            </nav>
          </div>
          
          <div className="flex items-center gap-4 ml-auto">
            <button className="p-2 text-slate-400 hover:text-primary-600 hover:bg-primary-50 rounded-xl transition-all relative">
              <div className="absolute top-2 right-2.5 w-2 h-2 bg-red-500 rounded-full border-2 border-white"></div>
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" /></svg>
            </button>

            <div className="h-8 w-[1px] bg-slate-100 mx-1 hidden sm:block"></div>

            <div className="flex items-center gap-3 pl-1 group cursor-pointer">
              <div className="hidden sm:block text-right">
                <p className="text-sm font-bold text-slate-900 leading-none group-hover:text-primary-600 transition-colors">Admin</p>
                <p className="text-[10px] font-medium text-slate-400 mt-1 uppercase tracking-wider">Workspace Owner</p>
              </div>
              <div className="w-9 h-9 rounded-xl bg-primary-100 text-primary-600 flex items-center justify-center font-bold text-sm shadow-sm border border-primary-200 group-hover:bg-primary-600 group-hover:text-white transition-all duration-300">
                AD
              </div>
            </div>
          </div>
        </header>

        <main className="flex-1 overflow-y-auto p-4 md:p-8 lg:p-10 relative">
          <div className="max-w-7xl mx-auto animate-enter">
            <Outlet />
          </div>
        </main>
      </div>

      {/* Switcher Modal - Moved to TOP LEVEL of the viewport */}
      {showSwitcher && (
        <div className="modal-backdrop">
          <div className="modal-content">
            <div className="flex items-center justify-between p-8 border-b border-slate-50">
              <div className="flex items-center gap-4">
                <div className="bg-primary-50 p-3 rounded-2xl text-primary-600">
                  <Building2 className="w-6 h-6" />
                </div>
                <div>
                  <h2 className="text-xl font-bold text-slate-900 tracking-tight">Chuyển cơ sở</h2>
                  <p className="text-[10px] text-slate-400 font-bold uppercase tracking-widest mt-0.5">Văn phòng / Chi nhánh</p>
                </div>
              </div>
              <button 
                onClick={() => setShowSwitcher(false)} 
                className="p-2 hover:bg-slate-100 rounded-xl transition-colors group"
              >
                <X className="w-5 h-5 text-slate-300 group-hover:text-slate-600" />
              </button>
            </div>
            
            <div className="p-8">
              <p className="text-sm text-slate-500 mb-8 font-medium">Chọn văn phòng hoặc chi nhánh bạn muốn làm việc.</p>
              <WorkspaceSwitcher />
              <div className="mt-10 pt-6 border-t border-slate-50 flex justify-end">
                <button 
                  onClick={() => setShowSwitcher(false)}
                  className="px-8 py-3 text-sm font-bold text-slate-500 hover:bg-slate-50 rounded-xl transition-all"
                >
                  Đóng
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
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
                <Route path="/departments" element={<DepartmentsPage />} />
                <Route path="/time-tracking" element={<TimeTrackingPage />} />
                <Route path="/attendance-logs" element={<AttendanceLogsPage />} />
                <Route path="/leave-requests" element={<LeaveRequestsPage />} />
                <Route path="/attendance-adjustments" element={<AttendanceAdjustmentsPage />} />
                <Route path="/reports" element={<ReportsPage />} />
                <Route path="/shifts" element={<ShiftsPage />} />
                <Route path="/payroll" element={<PayrollPage />} />
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
