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
} from "lucide-react";
import { Link } from "react-router-dom";
import { useAgentsStatus } from "@/hooks/use-agents";
import { useDevices } from "@/hooks/use-devices";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useDashboardStats } from "@/hooks/use-dashboard";

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
    const currency = i18n.language === "vi" ? "VND" : i18n.language === "zh" ? "CNY" : "USD";
    return new Intl.NumberFormat(locale, { style: "currency", currency: currency }).format(Number(value));
  };

  return (
    <div className="space-y-10">
      <header className="flex flex-col md:flex-row md:items-end justify-between gap-6 px-2">
        <div>
          <h1 data-testid="dashboard-title" className="text-4xl font-black text-slate-900 tracking-tight">{t("dashboard.title")}</h1>
          <p className="text-slate-500 mt-2 text-lg font-medium">{t("dashboard.welcome")}</p>
        </div>

        {/* Tab Switcher */}
        <div className="flex bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50 shadow-inner">
          <button
            onClick={() => setActiveTab("hr")}
            className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${
              activeTab === "hr" ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700"
            }`}
          >
            <Users size={14} /> {t("dashboard.tabs.hr")}
          </button>
          <button
            onClick={() => setActiveTab("system")}
            className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${
              activeTab === "system" ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700"
            }`}
          >
            <Cpu size={14} /> {t("dashboard.tabs.system")}
          </button>
        </div>
      </header>

      <div className="animate-in fade-in duration-700">
        {activeTab === "hr" ? (
          <div className="space-y-12">
            {/* HR Stats Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              {[
                { label: "total_employees", value: hrStats?.total_employees, icon: Users2, color: "bg-primary-50 text-primary-600" },
                { label: "present_today", value: hrStats?.present_today, icon: UserCheck, color: "bg-emerald-50 text-emerald-600" },
                { label: "pending_requests", value: hrStats?.pending_leave_requests, icon: CalendarClock, color: "bg-amber-50 text-amber-600" },
                { label: "estimated_payroll", value: hrStats?.estimated_monthly_payroll, icon: Banknote, color: "bg-rose-50 text-rose-600", isCurrency: true }
              ].map((stat, i) => (
                <div key={i} className="bg-white p-8 rounded-[32px] border border-slate-100 shadow-sm hover:shadow-xl hover:shadow-primary-100/20 transition-all group relative overflow-hidden">
                  <div className={`${stat.color} w-14 h-14 rounded-2xl flex items-center justify-center mb-6 group-hover:scale-110 transition-transform shadow-inner`}>
                    <stat.icon size={24} />
                  </div>
                  <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t(`dashboard.hr.${stat.label}`)}</p>
                  <p className={`font-black text-slate-900 mt-2 truncate ${stat.isCurrency ? "text-2xl" : "text-4xl"}`}>
                    {isLoading ? "..." : stat.isCurrency ? formatCurrency(stat.value || 0) : stat.value || 0}
                  </p>
                  <stat.icon className="absolute -right-4 -bottom-4 w-32 h-32 text-slate-50 opacity-[0.03] -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
                </div>
              ))}
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-10">
               <div className="bg-slate-900 rounded-[3rem] p-12 text-white relative overflow-hidden group shadow-2xl shadow-slate-200">
                  <div className="relative z-10 h-full flex flex-col">
                    <span className="text-primary-400 text-xs font-black uppercase tracking-widest mb-4">{t("dashboard.hr.hr_management")}</span>
                    <h3 className="text-4xl font-black mb-6 tracking-tight leading-tight">{t("dashboard.hr.hr_management_title")}</h3>
                    <p className="text-slate-400 mb-10 text-lg leading-relaxed max-w-sm font-medium">{t("dashboard.hr.hr_management_desc")}</p>
                    <Link to="/employees" className="bg-white text-slate-900 px-8 py-4 rounded-2xl font-black text-xs uppercase tracking-widest transition-all inline-flex items-center gap-3 w-fit hover:gap-5 shadow-xl">
                      {t("dashboard.hr.hr_management_btn")} <ArrowRight className="w-5 h-5" />
                    </Link>
                  </div>
                  <Users2 className="absolute -right-16 -bottom-16 w-64 h-64 text-primary-500 opacity-10 -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
               </div>

               <div className="bg-primary-600 rounded-[3rem] p-12 text-white relative overflow-hidden group shadow-2xl shadow-primary-100">
                  <div className="relative z-10 h-full flex flex-col">
                    <span className="text-primary-200 text-xs font-black uppercase tracking-widest mb-4">{t("dashboard.hr.payroll")}</span>
                    <h3 className="text-4xl font-black mb-6 tracking-tight leading-tight">{t("dashboard.hr.payroll_title")}</h3>
                    <p className="text-primary-100 mb-10 text-lg leading-relaxed max-w-sm font-medium">{t("dashboard.hr.payroll_desc")}</p>
                    <Link to="/time-tracking" className="bg-slate-900 text-white px-8 py-4 rounded-2xl font-black text-xs uppercase tracking-widest transition-all inline-flex items-center gap-3 w-fit hover:gap-5 shadow-xl shadow-black/20">
                      {t("dashboard.hr.payroll_btn")} <ArrowRight className="w-5 h-5" />
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
              {[
                { label: "total_agents", value: agents.length, icon: Users, color: "bg-primary-50 text-primary-600" },
                { label: "online_agents", value: onlineAgents, icon: Activity, color: "bg-emerald-50 text-emerald-600" },
                { label: "total_devices", value: devices.length, icon: Cpu, color: "bg-indigo-50 text-indigo-600" },
                { label: "active_devices", value: activeDevices, icon: ShieldCheck, color: "bg-amber-50 text-amber-600" }
              ].map((stat, i) => (
                <div key={i} className="bg-white p-8 rounded-[32px] border border-slate-100 shadow-sm hover:shadow-xl hover:shadow-primary-100/20 transition-all group relative overflow-hidden">
                  <div className={`${stat.color} w-14 h-14 rounded-2xl flex items-center justify-center mb-6 group-hover:scale-110 transition-transform shadow-inner`}>
                    <stat.icon size={24} />
                  </div>
                  <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t(`dashboard.system.${stat.label}`)}</p>
                  <p className="text-4xl font-black text-slate-900 mt-2">{isLoading ? "..." : stat.value}</p>
                  <stat.icon className="absolute -right-4 -bottom-4 w-32 h-32 text-slate-50 opacity-[0.03] -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
                </div>
              ))}
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-10">
              <section className="space-y-6">
                <div className="flex justify-between items-center px-4">
                  <h2 className="text-2xl font-black text-slate-800 tracking-tight uppercase tracking-widest text-sm">{t("dashboard.system.edge_agents")}</h2>
                  <span className="text-[9px] font-black text-primary-600 bg-primary-50 px-3 py-1.5 rounded-full uppercase tracking-[0.2em] border border-primary-100">{t("dashboard.system.live_data")}</span>
                </div>
                <div className="space-y-4">
                  {agents.map((agent) => (
                    <div key={agent.agent_id} className="bg-white p-6 rounded-[2.5rem] border border-slate-100 flex items-center justify-between hover:shadow-xl hover:shadow-primary-100/10 transition-all group">
                      <div className="flex items-center gap-5">
                        <div className="bg-slate-50 p-4 rounded-2xl text-slate-400 group-hover:bg-primary-50 group-hover:text-primary-600 transition-all border border-slate-100 shadow-inner">
                          <Monitor size={24} />
                        </div>
                        <div>
                          <h3 className="font-black text-slate-900 text-lg uppercase tracking-tight">{agent.agent_id}</h3>
                          <p className="text-[10px] font-black uppercase text-slate-400 tracking-widest">{t("dashboard.system.seen")}: {formatTime(agent.last_seen)}</p>
                        </div>
                      </div>
                      <span className={cn("px-4 py-1.5 rounded-full text-[9px] font-black uppercase tracking-widest border", agent.is_online ? "bg-emerald-50 text-emerald-600 border-emerald-100" : "bg-slate-100 text-slate-400 border-slate-200")}>
                        {agent.is_online ? t("dashboard.system.connected") : t("dashboard.system.disconnected")}
                      </span>
                    </div>
                  ))}
                  {agents.length === 0 && !isLoading && <div className="bg-slate-50/50 p-16 text-center rounded-[3rem] border-2 border-dashed border-slate-100 text-slate-400 font-bold uppercase tracking-widest text-xs italic">{t("dashboard.system.no_agents")}</div>}
                </div>
              </section>

              <section className="space-y-6">
                <h2 className="text-2xl font-black text-slate-800 tracking-tight px-4 uppercase tracking-widest text-sm">{t("dashboard.system.quick_actions")}</h2>
                <div className="bg-slate-900 rounded-[3rem] p-12 text-white relative overflow-hidden shadow-2xl shadow-slate-200 group">
                  <div className="relative z-10">
                    <h3 className="text-3xl font-black mb-4 tracking-tight uppercase">{t("dashboard.system.device_management")}</h3>
                    <p className="text-slate-400 mb-10 text-lg leading-relaxed max-w-md font-medium">{t("dashboard.system.device_management_desc")}</p>
                    <Link to="/devices" className="bg-primary-600 hover:bg-primary-500 text-white px-8 py-4 rounded-2xl font-black text-xs uppercase tracking-widest transition-all inline-block shadow-xl shadow-primary-900/40">
                      {t("dashboard.system.device_management_btn")}
                    </Link>
                  </div>
                  <Cpu className="absolute -right-12 -bottom-12 w-64 h-64 text-primary-400 opacity-10 -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
                </div>
              </section>
            </div>
          </div>
        )}
      </div>

      {error && (
        <div className="bg-rose-50 border-2 border-rose-100 p-8 rounded-[3rem] flex items-center gap-6 text-rose-700 animate-in slide-in-from-bottom-4 shadow-xl shadow-rose-100/50">
          <div className="bg-rose-100 p-4 rounded-2xl"><ShieldAlert className="w-8 h-8" /></div>
          <div>
            <p className="font-black text-xl uppercase tracking-tight">{t("dashboard.connection_error")}</p>
            <p className="font-bold opacity-80 mt-1">{t("dashboard.connection_error_desc")}</p>
          </div>
        </div>
      )}
    </div>
  );
}

      {error && (
        <div className="bg-rose-50 border border-rose-100 p-6 rounded-3xl flex items-center gap-4 text-rose-700 animate-in slide-in-from-top-4 duration-500">
          <ShieldAlert className="w-7 h-7 flex-shrink-0" />
          <div>
            <p className="font-bold text-lg">{t("dashboard.connection_error")}</p>
            <p className="opacity-90">{t("dashboard.connection_error_desc")}</p>
          </div>
        </div>
      )}
    </div>
  );
}
