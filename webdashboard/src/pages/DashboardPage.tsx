import {
  Monitor,
  ShieldCheck,
  ShieldAlert,
  Activity,
  Cpu,
  Users,
} from "lucide-react";
import { Link } from "react-router-dom";
import { useAgentsStatus } from "@/hooks/use-agents";
import { useDevices } from "@/hooks/use-devices";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

export default function DashboardPage() {
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

  const isLoading = agentsLoading || devicesLoading;
  const error = agentsError || devicesError;

  const onlineAgents = agents.filter((a) => a.is_online).length;
  const activeDevices = devices.filter((d) => d.is_enabled).length;

  return (
    <div className="p-8">
      <div className="max-w-7xl mx-auto">
        <header className="mb-8">
          <h1 className="text-2xl font-bold text-slate-900 tracking-tight">
            Tổng quan hệ thống
          </h1>
          <p className="text-slate-500">
            Theo dõi trạng thái thời gian thực của các chi nhánh
          </p>
        </header>

        {/* Stats Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-10">
          <div className="bg-white p-6 rounded-3xl shadow-sm border border-slate-100">
            <div className="bg-blue-50 w-12 h-12 rounded-2xl flex items-center justify-center mb-4 text-blue-600">
              <Users className="w-6 h-6" />
            </div>
            <p className="text-sm font-bold text-slate-400 uppercase tracking-widest">
              Tổng Agents
            </p>
            <p className="text-3xl font-black text-slate-900 mt-1">
              {isLoading ? "..." : agents.length}
            </p>
          </div>

          <div className="bg-white p-6 rounded-3xl shadow-sm border border-slate-100">
            <div className="bg-emerald-50 w-12 h-12 rounded-2xl flex items-center justify-center mb-4 text-emerald-600">
              <Activity className="w-6 h-6" />
            </div>
            <p className="text-sm font-bold text-slate-400 uppercase tracking-widest">
              Agents Online
            </p>
            <p className="text-3xl font-black text-slate-900 mt-1">
              {isLoading ? "..." : onlineAgents}
            </p>
          </div>

          <div className="bg-white p-6 rounded-3xl shadow-sm border border-slate-100">
            <div className="bg-purple-50 w-12 h-12 rounded-2xl flex items-center justify-center mb-4 text-purple-600">
              <Cpu className="w-6 h-6" />
            </div>
            <p className="text-sm font-bold text-slate-400 uppercase tracking-widest">
              Tổng thiết bị
            </p>
            <p className="text-3xl font-black text-slate-900 mt-1">
              {isLoading ? "..." : devices.length}
            </p>
          </div>

          <div className="bg-white p-6 rounded-3xl shadow-sm border border-slate-100">
            <div className="bg-amber-50 w-12 h-12 rounded-2xl flex items-center justify-center mb-4 text-amber-600">
              <ShieldCheck className="w-6 h-6" />
            </div>
            <p className="text-sm font-bold text-slate-400 uppercase tracking-widest">
              Thiết bị hoạt động
            </p>
            <p className="text-3xl font-black text-slate-900 mt-1">
              {isLoading ? "..." : activeDevices}
            </p>
          </div>
        </div>

        {error && (
          <div className="bg-rose-50 border border-rose-100 p-6 rounded-3xl flex items-center gap-4 text-rose-700 mb-10">
            <ShieldAlert className="w-6 h-6" />
            <p className="font-bold">
              Không thể kết nối tới API Server. Hãy đảm bảo apiserver đang chạy.
            </p>
          </div>
        )}

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          {/* Recent Agents */}
          <section>
            <div className="flex justify-between items-center mb-6">
              <h2 className="text-xl font-bold text-slate-800">Edge Agents</h2>
              <span className="text-xs font-bold text-blue-600 bg-blue-50 px-3 py-1 rounded-full uppercase tracking-tighter">
                Thời gian thực
              </span>
            </div>
            <div className="space-y-4">
              {isLoading ? (
                <div className="bg-white p-12 text-center rounded-3xl border border-dashed border-slate-200 text-slate-400 font-medium animate-pulse">
                  Đang tải...
                </div>
              ) : (
                <>
                  {agents.map((agent) => (
                    <div
                      key={agent.agent_id}
                      className="bg-white p-5 rounded-3xl border border-slate-100 flex items-center justify-between hover:shadow-md transition-shadow"
                    >
                      <div className="flex items-center gap-4">
                        <div className="bg-slate-50 p-3 rounded-2xl text-slate-400">
                          <Monitor className="w-6 h-6" />
                        </div>
                        <div>
                          <h3 className="font-bold text-slate-900">
                            {agent.agent_id}
                          </h3>
                          <p className="text-xs text-slate-400 italic">
                            Last seen:{" "}
                            {formatTime(agent.last_seen)}
                          </p>
                        </div>
                      </div>
                      <span
                        className={`px-4 py-1.5 rounded-2xl text-[10px] font-black uppercase tracking-widest ${
                          agent.is_online
                            ? "bg-emerald-100 text-emerald-700"
                            : "bg-slate-100 text-slate-500"
                        }`}
                      >
                        {agent.is_online ? "Connected" : "Disconnected"}
                      </span>
                    </div>
                  ))}
                  {agents.length === 0 && (
                    <div className="bg-white p-12 text-center rounded-3xl border border-dashed border-slate-200 text-slate-400 font-medium">
                      Chưa có Agent nào kết nối
                    </div>
                  )}
                </>
              )}
            </div>
          </section>

          {/* Quick Config Link */}
          <section>
            <div className="flex justify-between items-center mb-6">
              <h2 className="text-xl font-bold text-slate-800">
                Cấu hình nhanh
              </h2>
              <Link
                to="/devices"
                className="text-sm font-bold text-blue-600 hover:underline"
              >
                Tất cả →
              </Link>
            </div>
            <div className="bg-slate-900 rounded-3xl p-8 text-white relative overflow-hidden shadow-2xl shadow-blue-200">
              <div className="relative z-10">
                <h3 className="text-2xl font-bold mb-2">Quản lý Thiết bị</h3>
                <p className="text-slate-400 mb-6 text-sm leading-relaxed">
                  Thêm hoặc xóa máy chấm công tại các chi nhánh. Hệ thống sẽ tự
                  động đồng bộ cấu hình tới các Agent ngay lập tức.
                </p>
                <Link
                  to="/devices"
                  className="bg-blue-600 hover:bg-blue-500 text-white px-6 py-3 rounded-2xl font-bold transition-colors inline-block"
                >
                  Mở trình quản lý
                </Link>
              </div>
              <Cpu className="absolute -right-8 -bottom-8 w-40 h-40 text-white opacity-5 rotate-12" />
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
