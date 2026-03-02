import { useState } from "react";
import { useTranslation } from "react-i18next";
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
  Clock,
  UserX,
  AlertTriangle,
  LogOut,
  Timer,
} from "lucide-react";
import { Link } from "react-router-dom";
import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip } from "recharts";
import AttendanceTrendChart from "@/components/features/dashboard/AttendanceTrendChart";
import DepartmentAttendanceChart from "@/components/features/dashboard/DepartmentAttendanceChart";
import OvertimeTrendChart from "@/components/features/dashboard/OvertimeTrendChart";
import PeakTimesChart from "@/components/features/dashboard/PeakTimesChart";
import LeaveTypesChart from "@/components/features/dashboard/LeaveTypesChart";
import LatenessByDayChart from "@/components/features/dashboard/LatenessByDayChart";
import { useAgentsStatus } from "@/hooks/use-agents";
import { useDevices } from "@/hooks/use-devices";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useDashboardStats } from "@/hooks/use-dashboard";
import { cn } from "@/lib/utils";

const DONUT_COLORS = {
  on_time: "#10b981",   // emerald-500
  late: "#f59e0b",      // amber-500
  absent: "#ef4444",    // rose-500
  on_leave: "#6366f1",  // indigo-500
};

export default function DashboardPage() {
  const { t, i18n } = useTranslation();
  const [activeTab, setActiveTab] = useState<"system" | "hr">("hr");
  const { formatTime } = useDateTimeFormatter();
  const { agents, isLoading: agentsLoading, error: agentsError } = useAgentsStatus();
  const { devices, isLoading: devicesLoading, error: devicesError } = useDevices();
  const { data: hrStats, isLoading: hrLoading, error: hrError } = useDashboardStats();

  const isLoading = agentsLoading || devicesLoading || hrLoading;
  const error = agentsError || devicesError || hrError;

  const onlineAgents = agents.filter((a) => a.is_online).length;
  const activeDevices = devices.filter((d) => d.is_enabled).length;

  const formatCurrency = (value: string | number) => {
    const locale = i18n.language === "vi" ? "vi-VN" : i18n.language === "zh" ? "zh-CN" : "en-US";
    return new Intl.NumberFormat(locale, { style: "currency", currency: i18n.language === "vi" ? "VND" : i18n.language === "zh" ? "CNY" : "USD", maximumFractionDigits: 0 }).format(Number(value));
  };

  // Compute donut data from hrStats
  const onTimeCount = Math.max(0, (hrStats?.present_today ?? 0) - (hrStats?.late_today ?? 0));
  const donutData = [
    { name: t("dashboard.hr.on_time"), value: onTimeCount, color: DONUT_COLORS.on_time },
    { name: t("dashboard.hr.late_today"), value: hrStats?.late_today ?? 0, color: DONUT_COLORS.late },
    { name: t("dashboard.hr.absent_today"), value: hrStats?.absent_today ?? 0, color: DONUT_COLORS.absent },
    { name: t("dashboard.hr.on_leave_today"), value: hrStats?.on_leave_today ?? 0, color: DONUT_COLORS.on_leave },
  ];
  const hasDonutData = donutData.some((d) => d.value > 0);

  return (
    <div className="space-y-12 animate-in fade-in duration-700">
      <header className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div className="space-y-1">
          <h1 data-testid="dashboard-title" className="text-5xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none">{t("dashboard.title")}</h1>
          <p className="text-slate-500 dark:text-slate-400 text-lg font-bold uppercase tracking-[0.25em] opacity-60">{t("dashboard.welcome")}</p>
        </div>

        <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
          <button onClick={() => setActiveTab("hr")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'hr' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><Users size={14} /> {t("dashboard.tabs.hr")}</button>
          <button onClick={() => setActiveTab("system")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'system' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><Cpu size={14} /> {t("dashboard.tabs.system")}</button>
        </div>
      </header>

      <div className="animate-in slide-in-from-bottom-4 duration-700">
        {activeTab === "hr" ? (
          <div className="space-y-12">
            {/* Row 1: 6 Stat Cards */}
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-6">
              {[
                { label: "total_employees", value: hrStats?.total_employees, icon: Users2, color: "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 border-primary-100 dark:border-primary-800" },
                { label: "present_today", value: hrStats?.present_today, icon: UserCheck, color: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" },
                { label: "late_today", value: hrStats?.late_today, icon: Clock, color: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800" },
                { label: "absent_today", value: hrStats?.absent_today, icon: UserX, color: "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800" },
                { label: "on_leave_today", value: hrStats?.on_leave_today, icon: CalendarClock, color: "bg-indigo-50 dark:bg-indigo-950 text-indigo-600 dark:text-indigo-300 border-indigo-100 dark:border-indigo-800" },
                { label: "pending_requests", value: hrStats?.pending_leave_requests, icon: AlertTriangle, color: "bg-orange-50 dark:bg-orange-950 text-orange-600 dark:text-orange-300 border-orange-100 dark:border-orange-800" },
              ].map((stat, i) => (
                <div key={i} className="bg-white dark:bg-slate-900 p-6 rounded-[2rem] border border-slate-100 dark:border-slate-800 shadow-sm hover:shadow-2xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden">
                  <div className={`${stat.color} w-12 h-12 rounded-[1rem] flex items-center justify-center mb-5 group-hover:scale-110 group-hover:rotate-3 transition-all shadow-inner border`}><stat.icon size={22} /></div>
                  <p className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mb-1">{t(`dashboard.hr.${stat.label}`)}</p>
                  <p className="text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter">
                    {isLoading ? "..." : stat.value ?? 0}
                  </p>
                  <stat.icon className="absolute -right-4 -bottom-4 w-24 h-24 text-slate-50 dark:text-slate-800 opacity-10 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
                </div>
              ))}
            </div>

            {/* Row 2: Attendance Trend */}
            <AttendanceTrendChart />

            {/* Row 2b: Department Attendance Comparison */}
            <DepartmentAttendanceChart />

            {/* Row 2c: Overtime Trend */}
            <OvertimeTrendChart />

            {/* Row 2d: Peak Check-in/Check-out Times */}
            <PeakTimesChart />

            {/* Row 2e: Leave Types + Lateness by Day */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-10">
              <LeaveTypesChart />
              <LatenessByDayChart />
            </div>

            {/* Row 3: Donut Chart + Quick Stats */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-10">
              {/* Donut Chart */}
              <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
                <h2 className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mb-8">{t("dashboard.hr.attendance_breakdown")}</h2>
                {hasDonutData ? (
                  <div className="flex flex-col items-center">
                    <ResponsiveContainer width="100%" height={260}>
                      <PieChart>
                        <Pie
                          data={donutData}
                          cx="50%"
                          cy="50%"
                          innerRadius={70}
                          outerRadius={110}
                          paddingAngle={3}
                          dataKey="value"
                          stroke="none"
                        >
                          {donutData.map((entry, index) => (
                            <Cell key={`cell-${index}`} fill={entry.color} />
                          ))}
                        </Pie>
                        <Tooltip
                          contentStyle={{ borderRadius: "1rem", border: "none", boxShadow: "0 8px 30px rgba(0,0,0,.1)", fontSize: "12px", fontWeight: 600 }}
                        />
                      </PieChart>
                    </ResponsiveContainer>
                    <div className="flex flex-wrap justify-center gap-5 mt-4">
                      {donutData.map((entry, i) => (
                        <div key={i} className="flex items-center gap-2">
                          <div className="w-3 h-3 rounded-full" style={{ backgroundColor: entry.color }} />
                          <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{entry.name}: {entry.value}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                ) : (
                  <div className="flex flex-col items-center justify-center h-[260px]">
                    <Users2 size={48} className="text-slate-200 dark:text-slate-700 mb-4" />
                    <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">{t("common.no_data")}</p>
                  </div>
                )}
              </div>

              {/* Quick Stats Panel */}
              <div className="space-y-6">
                <h2 className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] px-4">{t("dashboard.hr.quick_stats")}</h2>

                {/* Avg Work Hours */}
                <div className="bg-white dark:bg-slate-900 p-8 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm flex items-center gap-6 group hover:shadow-2xl hover:shadow-primary-100/10 transition-all">
                  <div className="bg-sky-50 dark:bg-sky-950 text-sky-600 dark:text-sky-400 w-14 h-14 rounded-[1.2rem] flex items-center justify-center shadow-inner border border-sky-100 dark:border-sky-800 group-hover:scale-110 transition-transform">
                    <Timer size={26} />
                  </div>
                  <div>
                    <p className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mb-1">{t("dashboard.hr.avg_work_hours")}</p>
                    <p className="text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter font-mono">
                      {isLoading ? "..." : `${(hrStats?.avg_work_hours_today ?? 0).toFixed(1)}h`}
                    </p>
                  </div>
                </div>

                {/* Early Departures */}
                <div className="bg-white dark:bg-slate-900 p-8 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm flex items-center gap-6 group hover:shadow-2xl hover:shadow-primary-100/10 transition-all">
                  <div className="bg-violet-50 dark:bg-violet-950 text-violet-600 dark:text-violet-400 w-14 h-14 rounded-[1.2rem] flex items-center justify-center shadow-inner border border-violet-100 dark:border-violet-800 group-hover:scale-110 transition-transform">
                    <LogOut size={26} />
                  </div>
                  <div>
                    <p className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mb-1">{t("dashboard.hr.early_departures")}</p>
                    <p className="text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter">
                      {isLoading ? "..." : hrStats?.early_departure_today ?? 0}
                    </p>
                  </div>
                </div>

                {/* Estimated Payroll */}
                <div className="bg-white dark:bg-slate-900 p-8 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm flex items-center gap-6 group hover:shadow-2xl hover:shadow-primary-100/10 transition-all">
                  <div className="bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-400 w-14 h-14 rounded-[1.2rem] flex items-center justify-center shadow-inner border border-rose-100 dark:border-rose-800 group-hover:scale-110 transition-transform">
                    <Banknote size={26} />
                  </div>
                  <div>
                    <p className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mb-1">{t("dashboard.hr.estimated_payroll")}</p>
                    <p className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter font-mono">
                      {isLoading ? "..." : formatCurrency(hrStats?.estimated_monthly_payroll || 0)}
                    </p>
                  </div>
                </div>
              </div>
            </div>

            {/* Row 3: Promotional Cards (preserved from original) */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-10">
              <div className="bg-slate-900 rounded-[3rem] p-14 text-white relative overflow-hidden group shadow-2xl shadow-slate-200 dark:shadow-black/40">
                <div className="relative z-10 h-full flex flex-col items-start">
                  <span className="text-primary-400 text-xs font-bold uppercase tracking-[0.3em] mb-6">{t("dashboard.hr.hr_management")}</span>
                  <h3 className="text-5xl font-bold mb-8 tracking-tighter leading-[1.1] uppercase">{t("dashboard.hr.hr_management_title")}</h3>
                  <p className="text-slate-500 dark:text-slate-400 mb-12 text-lg leading-relaxed max-w-sm font-bold uppercase tracking-tight opacity-80">{t("dashboard.hr.hr_management_desc")}</p>
                  <Link to="/employees" className="bg-white dark:bg-slate-900 text-slate-700 dark:text-slate-200 px-10 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest transition-all inline-flex items-center gap-4 hover:gap-6 shadow-2xl hover:shadow-primary-500/20 active:scale-95">
                    {t("dashboard.hr.hr_management_btn").toUpperCase()} <ArrowRight size={20} />
                  </Link>
                </div>
                <Users2 className="absolute -right-20 -bottom-20 w-80 h-80 text-primary-500 opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
              </div>

              <div className="bg-primary-600 rounded-[3rem] p-14 text-white relative overflow-hidden group shadow-2xl shadow-primary-900/20">
                <div className="relative z-10 h-full flex flex-col items-start">
                  <span className="text-primary-200 text-xs font-bold uppercase tracking-[0.3em] mb-6">{t("dashboard.hr.payroll")}</span>
                  <h3 className="text-5xl font-bold mb-8 tracking-tighter leading-[1.1] uppercase">{t("dashboard.hr.payroll_title")}</h3>
                  <p className="text-primary-100 mb-12 text-lg leading-relaxed max-w-sm font-bold uppercase tracking-tight opacity-80">{t("dashboard.hr.payroll_desc")}</p>
                  <Link to="/attendance-logs" className="bg-slate-900 text-white px-10 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest transition-all inline-flex items-center gap-4 hover:gap-6 shadow-2xl shadow-black/20 active:scale-95">
                    {t("dashboard.hr.payroll_btn").toUpperCase()} <ArrowRight size={20} />
                  </Link>
                </div>
                <Banknote className="absolute -right-20 -bottom-20 w-80 h-80 text-white opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
              </div>
            </div>
          </div>
        ) : (
          <div className="space-y-12 animate-in fade-in duration-500">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
              {[
                { label: "total_agents", value: agents.length, icon: Users, color: "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 border-primary-100 dark:border-primary-800" },
                { label: "online_agents", value: onlineAgents, icon: Activity, color: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" },
                { label: "total_devices", value: devices.length, icon: Cpu, color: "bg-indigo-50 dark:bg-indigo-950 text-indigo-600 border-indigo-100" },
                { label: "active_devices", value: activeDevices, icon: ShieldCheck, color: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800" }
              ].map((stat, i) => (
                <div key={i} className="bg-white dark:bg-slate-900 p-8 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm hover:shadow-2xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden">
                  <div className={`${stat.color} w-16 h-16 rounded-[1.5rem] flex items-center justify-center mb-8 group-hover:scale-110 group-hover:rotate-3 transition-all shadow-inner border`}><stat.icon size={28} /></div>
                  <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mb-2">{t(`dashboard.system.${stat.label}`)}</p>
                  <p className="text-5xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter">{isLoading ? "..." : stat.value}</p>
                  <stat.icon className="absolute -right-6 -bottom-6 w-32 h-32 text-slate-50 opacity-10 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
                </div>
              ))}
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-10">
              <section className="space-y-8">
                <div className="flex justify-between items-center px-4">
                  <h2 className="text-xs font-bold text-slate-500 dark:text-slate-400 tracking-[0.3em] uppercase">{t("dashboard.system.edge_agents")}</h2>
                  <span className="text-[9px] font-bold text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950 px-4 py-1.5 rounded-full uppercase tracking-widest border border-primary-100 dark:border-primary-800 shadow-sm">{t("dashboard.system.live_data")}</span>
                </div>
                <div className="space-y-4">
                  {agents.map((agent) => (
                    <div key={agent.agent_id} className="bg-white dark:bg-slate-900 p-6 px-8 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 flex items-center justify-between hover:shadow-2xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden">
                      <div className="flex items-center gap-6">
                        <div className="bg-slate-50 dark:bg-slate-800 p-5 rounded-[1.5rem] text-slate-500 dark:text-slate-400 group-hover:bg-primary-50 dark:group-hover:bg-primary-950 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-all border border-slate-100 dark:border-slate-800 shadow-inner">
                          <Monitor size={28} />
                        </div>
                        <div>
                          <h3 className="font-bold text-slate-700 dark:text-slate-200 text-xl uppercase tracking-tight leading-none">{agent.agent_id}</h3>
                          <p className="text-[10px] font-bold uppercase text-slate-500 dark:text-slate-400 tracking-widest mt-2 opacity-70">{t("dashboard.system.seen")}: {formatTime(agent.last_seen)}</p>
                        </div>
                      </div>
                      <span className={cn("px-5 py-2 rounded-full text-[10px] font-bold uppercase tracking-widest border shadow-sm", agent.is_online ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" : "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 opacity-60")}>
                        {agent.is_online ? t("dashboard.system.connected").toUpperCase() : t("dashboard.system.disconnected").toUpperCase()}
                      </span>
                    </div>
                  ))}
                  {agents.length === 0 && !isLoading && <div className="py-32 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center"><Monitor size={40} className="text-slate-200 dark:text-slate-700 mb-4" /><p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">{t("dashboard.system.no_agents")}</p></div>}
                </div>
              </section>

              <section className="space-y-8">
                <h2 className="text-xs font-bold text-slate-500 dark:text-slate-400 tracking-[0.3em] uppercase px-4">{t("dashboard.system.quick_actions")}</h2>
                <div className="bg-slate-900 rounded-[3rem] p-14 text-white relative overflow-hidden shadow-2xl shadow-slate-200 dark:shadow-black/40 group">
                  <div className="relative z-10">
                    <h3 className="text-4xl font-bold mb-6 tracking-tighter uppercase leading-[1.1]">{t("dashboard.system.device_management")}</h3>
                    <p className="text-slate-500 dark:text-slate-400 mb-12 text-lg leading-relaxed max-w-sm font-bold uppercase tracking-tight opacity-80">{t("dashboard.system.device_management_desc")}</p>
                    <Link to="/devices" className="bg-primary-600 hover:bg-primary-50 dark:bg-primary-950 text-white px-10 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest transition-all inline-block shadow-2xl shadow-primary-900/40 active:scale-95">
                      {t("dashboard.system.device_management_btn").toUpperCase()}
                    </Link>
                  </div>
                  <Cpu className="absolute -right-16 -bottom-16 w-80 h-80 text-primary-400 opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
                </div>
              </section>
            </div>
          </div>
        )}
      </div>

      {error && (
        <div className="bg-rose-50 dark:bg-rose-950 border-2 border-rose-100 dark:border-rose-800 p-10 rounded-[3rem] flex items-center gap-8 text-rose-700 dark:text-rose-300 animate-in slide-in-from-bottom-8 shadow-2xl shadow-rose-100/50">
          <div className="bg-white dark:bg-slate-900 p-5 rounded-[1.5rem] shadow-sm border border-rose-100 dark:border-rose-800"><ShieldAlert size={40} className="text-rose-500" /></div>
          <div>
            <p className="font-bold text-2xl uppercase tracking-tighter leading-none mb-2">{t("dashboard.connection_error")}</p>
            <p className="font-bold opacity-70 uppercase tracking-tight text-sm">{t("dashboard.connection_error_desc")}</p>
          </div>
        </div>
      )}
    </div>
  );
}
