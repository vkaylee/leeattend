import { useState } from "react";
import {
  Monitor,
  ShieldCheck,
  ShieldAlert,
  Activity,
  Cpu,
  Users,
  UserCheck,
  CalendarClock,
  Banknote,
  Users2,
  ArrowRight,
} from "lucide-react";
import { Link } from "react-router-dom";
import { useAgentsStatus } from "@/hooks/use-agents";
import { useDevices } from "@/hooks/use-devices";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useDashboardStats } from "@/hooks/use-dashboard";

export default function DashboardPage() {
  const [activeTab, setActiveTab] = useState<"system" | "hr">("hr");
  const { formatTime } = useDateTimeFormatter();
  const {
    agents,
    isLoading: agentsLoading,
    error: agentsError,
  } = useAgentsStatus();
  const {
    devices,
    isLoading: devicesLoading,
    error: devicesError,
  } = useDevices();
  const {
    data: hrStats,
    isLoading: hrLoading,
    error: hrError,
  } = useDashboardStats();

  const isLoading = agentsLoading || devicesLoading || hrLoading;
  const error = agentsError || devicesError || hrError;

  const onlineAgents = agents.filter((a) => a.is_online).length;
  const activeDevices = devices.filter((d) => d.is_enabled).length;

  const formatCurrency = (value: string | number) => {
    return new Intl.NumberFormat("vi-VN", {
      style: "currency",
      currency: "VND",
    }).format(Number(value));
  };

  return (
    <div className="space-y-10">
      <header className="flex flex-col md:flex-row md:items-end justify-between gap-6">
        <div>
          <h1 className="text-4xl font-black text-slate-900 tracking-tight">
            Dashboard
          </h1>
          <p className="text-slate-500 mt-2 text-lg">
            Chào mừng trở lại! Đây là tóm tắt hoạt động trong workspace của bạn.
          </p>
        </div>

        {/* Tab Switcher */}
        <div className="flex bg-slate-100 p-1.5 rounded-2xl w-fit border border-slate-200 shadow-inner">
          <button
            onClick={() => setActiveTab("hr")}
            className={`px-6 py-2.5 rounded-xl text-sm font-bold transition-all ${
              activeTab === "hr"
                ? "bg-white text-violet-600 shadow-premium-sm"
                : "text-slate-500 hover:text-slate-700"
            }`}
          >
            Nhân sự & Công lương
          </button>
          <button
            onClick={() => setActiveTab("system")}
            className={`px-6 py-2.5 rounded-xl text-sm font-bold transition-all ${
              activeTab === "system"
                ? "bg-white text-violet-600 shadow-premium-sm"
                : "text-slate-500 hover:text-slate-700"
            }`}
          >
            Hệ thống kỹ thuật
          </button>
        </div>
      </header>

      {activeTab === "hr" ? (
        <div className="space-y-12">
          {/* HR Stats Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <div className="bg-white p-7 rounded-[2.5rem] border border-slate-100 shadow-premium-sm hover:shadow-premium-lg transition-all group overflow-hidden relative">
              <div className="bg-violet-50 w-14 h-14 rounded-2xl flex items-center justify-center mb-6 text-violet-600 group-hover:scale-110 transition-transform relative z-10">
                <Users2 className="w-7 h-7" />
              </div>
              <p className="text-xs font-black text-slate-400 uppercase tracking-[0.2em] relative z-10">
                Tổng nhân sự
              </p>
              <p className="text-4xl font-black text-slate-900 mt-2 relative z-10">
                {isLoading ? "..." : hrStats?.total_employees || 0}
              </p>
              <Users2 className="absolute -right-4 -bottom-4 w-32 h-32 text-slate-50 opacity-[0.03] -rotate-12" />
            </div>

            <div className="bg-white p-7 rounded-[2.5rem] border border-slate-100 shadow-premium-sm hover:shadow-premium-lg transition-all group overflow-hidden relative">
              <div className="bg-emerald-50 w-14 h-14 rounded-2xl flex items-center justify-center mb-6 text-emerald-600 group-hover:scale-110 transition-transform relative z-10">
                <UserCheck className="w-7 h-7" />
              </div>
              <p className="text-xs font-black text-slate-400 uppercase tracking-[0.2em] relative z-10">
                Đi làm hôm nay
              </p>
              <p className="text-4xl font-black text-slate-900 mt-2 relative z-10">
                {isLoading ? "..." : hrStats?.present_today || 0}
              </p>
              <UserCheck className="absolute -right-4 -bottom-4 w-32 h-32 text-slate-50 opacity-[0.03] -rotate-12" />
            </div>

            <div className="bg-white p-7 rounded-[2.5rem] border border-slate-100 shadow-premium-sm hover:shadow-premium-lg transition-all group overflow-hidden relative">
              <div className="bg-amber-50 w-14 h-14 rounded-2xl flex items-center justify-center mb-6 text-amber-600 group-hover:scale-110 transition-transform relative z-10">
                <CalendarClock className="w-7 h-7" />
              </div>
              <p className="text-xs font-black text-slate-400 uppercase tracking-[0.2em] relative z-10">
                Đơn chờ duyệt
              </p>
              <p className="text-4xl font-black text-slate-900 mt-2 relative z-10">
                {isLoading ? "..." : hrStats?.pending_leave_requests || 0}
              </p>
              <CalendarClock className="absolute -right-4 -bottom-4 w-32 h-32 text-slate-50 opacity-[0.03] -rotate-12" />
            </div>

            <div className="bg-white p-7 rounded-[2.5rem] border border-slate-100 shadow-premium-sm hover:shadow-premium-lg transition-all group overflow-hidden relative">
              <div className="bg-rose-50 w-14 h-14 rounded-2xl flex items-center justify-center mb-6 text-rose-600 group-hover:scale-110 transition-transform relative z-10">
                <Banknote className="w-7 h-7" />
              </div>
              <p className="text-xs font-black text-slate-400 uppercase tracking-[0.2em] relative z-10">
                Quỹ lương ước tính
              </p>
              <p className="text-2xl font-black text-slate-900 mt-2 relative z-10 truncate">
                {isLoading ? "..." : formatCurrency(hrStats?.estimated_monthly_payroll || 0)}
              </p>
              <Banknote className="absolute -right-4 -bottom-4 w-32 h-32 text-slate-50 opacity-[0.03] -rotate-12" />
            </div>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-10">
             <div className="bg-slate-900 rounded-[3rem] p-12 text-white relative overflow-hidden group shadow-2xl shadow-violet-200">
                <div className="relative z-10 h-full flex flex-col">
                  <span className="text-violet-400 text-xs font-black uppercase tracking-widest mb-4">Nhân sự</span>
                  <h3 className="text-4xl font-bold mb-6 tracking-tight leading-tight">Quản lý hồ sơ<br/>nhân viên tập trung</h3>
                  <p className="text-slate-400 mb-10 text-lg leading-relaxed max-w-sm">
                    Theo dõi hợp đồng, bảo hiểm, quá trình công tác và lương bổng của toàn bộ đội ngũ.
                  </p>
                  <Link
                    to="/employees"
                    className="bg-white text-slate-900 px-8 py-4 rounded-2xl font-bold transition-all inline-flex items-center gap-3 w-fit hover:gap-5"
                  >
                    Mở danh sách nhân viên <ArrowRight className="w-5 h-5" />
                  </Link>
                </div>
                <Users2 className="absolute -right-16 -bottom-16 w-64 h-64 text-violet-500 opacity-10 -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
             </div>

             <div className="bg-violet-600 rounded-[3rem] p-12 text-white relative overflow-hidden group shadow-2xl shadow-violet-400/20">
                <div className="relative z-10 h-full flex flex-col">
                  <span className="text-violet-200 text-xs font-black uppercase tracking-widest mb-4">Công lương</span>
                  <h3 className="text-4xl font-bold mb-6 tracking-tight leading-tight">Bảng công &<br/>Tính lương tự động</h3>
                  <p className="text-violet-100 mb-10 text-lg leading-relaxed max-w-sm">
                    Đồng bộ dữ liệu chấm công thời gian thực để tạo bảng lương chính xác chỉ trong vài giây.
                  </p>
                  <Link
                    to="/time-tracking"
                    className="bg-slate-900 text-white px-8 py-4 rounded-2xl font-bold transition-all inline-flex items-center gap-3 w-fit hover:gap-5 shadow-xl shadow-black/20"
                  >
                    Xem bảng công <ArrowRight className="w-5 h-5" />
                  </Link>
                </div>
                <Banknote className="absolute -right-16 -bottom-16 w-64 h-64 text-white opacity-10 -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
             </div>
          </div>
        </div>
      ) : (
        <div className="space-y-12 animate-in fade-in duration-500">
          {/* System Stats Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <div className="bg-white p-7 rounded-3xl border border-slate-100 shadow-sm hover:shadow-md transition-all group">
              <div className="bg-violet-50 w-14 h-14 rounded-2xl flex items-center justify-center mb-6 text-violet-600 group-hover:scale-110 transition-transform">
                <Users className="w-7 h-7" />
              </div>
              <p className="text-xs font-bold text-slate-400 uppercase tracking-[0.2em]">
                Tổng Agents
              </p>
              <p className="text-4xl font-black text-slate-900 mt-2">
                {isLoading ? "..." : agents.length}
              </p>
            </div>

            <div className="bg-white p-7 rounded-3xl border border-slate-100 shadow-sm hover:shadow-md transition-all group">
              <div className="bg-emerald-50 w-14 h-14 rounded-2xl flex items-center justify-center mb-6 text-emerald-600 group-hover:scale-110 transition-transform">
                <Activity className="w-7 h-7" />
              </div>
              <p className="text-xs font-bold text-slate-400 uppercase tracking-[0.2em]">
                Agents Online
              </p>
              <p className="text-4xl font-black text-slate-900 mt-2">
                {isLoading ? "..." : onlineAgents}
              </p>
            </div>

            <div className="bg-white p-7 rounded-3xl border border-slate-100 shadow-sm hover:shadow-md transition-all group">
              <div className="bg-violet-50 w-14 h-14 rounded-2xl flex items-center justify-center mb-6 text-violet-600 group-hover:scale-110 transition-transform">
                <Cpu className="w-7 h-7" />
              </div>
              <p className="text-xs font-bold text-slate-400 uppercase tracking-[0.2em]">
                Tổng thiết bị
              </p>
              <p className="text-4xl font-black text-slate-900 mt-2">
                {isLoading ? "..." : devices.length}
              </p>
            </div>

            <div className="bg-white p-7 rounded-3xl border border-slate-100 shadow-sm hover:shadow-md transition-all group">
              <div className="bg-amber-50 w-14 h-14 rounded-2xl flex items-center justify-center mb-6 text-amber-600 group-hover:scale-110 transition-transform">
                <ShieldCheck className="w-7 h-7" />
              </div>
              <p className="text-xs font-bold text-slate-400 uppercase tracking-[0.2em]">
                Thiết bị hoạt động
              </p>
              <p className="text-4xl font-black text-slate-900 mt-2">
                {isLoading ? "..." : activeDevices}
              </p>
            </div>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-10">
            {/* Recent Agents */}
            <section>
              <div className="flex justify-between items-center mb-6 px-2">
                <h2 className="text-2xl font-bold text-slate-800 tracking-tight">Edge Agents</h2>
                <span className="text-[10px] font-black text-violet-600 bg-violet-100 px-3 py-1.5 rounded-xl uppercase tracking-widest">
                  Live Data
                </span>
              </div>
              <div className="space-y-4">
                {agents.map((agent) => (
                  <div
                    key={agent.agent_id}
                    className="bg-white p-6 rounded-3xl border border-slate-50 flex items-center justify-between hover:shadow-premium-lg transition-all duration-300 group"
                  >
                    <div className="flex items-center gap-5">
                      <div className="bg-slate-50 p-4 rounded-2xl text-slate-400 group-hover:bg-violet-50 group-hover:text-violet-500 transition-colors">
                        <Monitor className="w-7 h-7" />
                      </div>
                      <div>
                        <h3 className="font-bold text-slate-900 text-lg">
                          {agent.agent_id}
                        </h3>
                        <p className="text-sm text-slate-400 mt-0.5">
                          Seen: {formatTime(agent.last_seen)}
                        </p>
                      </div>
                    </div>
                    <span
                      className={`px-5 py-2 rounded-2xl text-[10px] font-black uppercase tracking-[0.2em] transition-colors ${
                        agent.is_online
                          ? "bg-emerald-100 text-emerald-700"
                          : "bg-slate-100 text-slate-500"
                      }`}
                    >
                      {agent.is_online ? "Connected" : "Disconnected"}
                    </span>
                  </div>
                ))}
                {agents.length === 0 && !isLoading && (
                  <div className="bg-white p-16 text-center rounded-[2rem] border border-dashed border-slate-200 text-slate-400 font-medium">
                    Chưa có Agent nào kết nối
                  </div>
                )}
              </div>
            </section>

            {/* Quick Config Link */}
            <section>
              <div className="flex justify-between items-center mb-6 px-2">
                <h2 className="text-2xl font-bold text-slate-800 tracking-tight">
                  Hành động nhanh
                </h2>
              </div>
              <div className="bg-slate-900 rounded-[2.5rem] p-10 text-white relative overflow-hidden shadow-2xl shadow-violet-200 group">
                <div className="relative z-10">
                  <h3 className="text-3xl font-bold mb-4 tracking-tight">Quản lý Thiết bị</h3>
                  <p className="text-slate-400 mb-8 text-lg leading-relaxed max-w-md">
                    Thêm hoặc xóa máy chấm công tại các chi nhánh. Hệ thống tự động đồng bộ tức thì.
                  </p>
                  <Link
                    to="/devices"
                    className="bg-violet-600 hover:bg-violet-500 text-white px-8 py-4 rounded-2xl font-bold transition-all inline-block hover:scale-105 active:scale-95 shadow-xl shadow-violet-900/20"
                  >
                    Mở trình quản lý
                  </Link>
                </div>
                <Cpu className="absolute -right-12 -bottom-12 w-56 h-56 text-violet-400 opacity-10 -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
              </div>
            </section>
          </div>
        </div>
      )}

      {error && (
        <div className="bg-rose-50 border border-rose-100 p-6 rounded-3xl flex items-center gap-4 text-rose-700 animate-in slide-in-from-top-4 duration-500">
          <ShieldAlert className="w-7 h-7 flex-shrink-0" />
          <div>
            <p className="font-bold text-lg">Lỗi kết nối</p>
            <p className="opacity-90">Không thể kết nối tới API Server hoặc dữ liệu không hợp lệ.</p>
          </div>
        </div>
      )}
    </div>
  );
}
