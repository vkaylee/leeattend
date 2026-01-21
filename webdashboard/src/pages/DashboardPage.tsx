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
import { cn } from "@/lib/utils";

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

  return (
    <div className="space-y-12 animate-in fade-in duration-700">
      <header className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div className="space-y-1">
          <h1 data-testid="dashboard-title" className="text-5xl font-black text-slate-900 tracking-tighter uppercase leading-none">{t("dashboard.title")}</h1>
          <p className="text-slate-400 text-lg font-black uppercase tracking-[0.25em] opacity-60">{t("dashboard.welcome")}</p>
        </div>

        <div className="flex bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50 shadow-inner">
          <button onClick={() => setActiveTab("hr")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'hr' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><Users size={14} /> {t("dashboard.tabs.hr")}</button>
          <button onClick={() => setActiveTab("system")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'system' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><Cpu size={14} /> {t("dashboard.tabs.system")}</button>
        </div>
      </header>

      <div className="animate-in slide-in-from-bottom-4 duration-700">
        {activeTab === "hr" ? (
          <div className="space-y-12">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
              {[
                { label: "total_employees", value: hrStats?.total_employees, icon: Users2, color: "bg-primary-50 text-primary-600 border-primary-100" },
                { label: "present_today", value: hrStats?.present_today, icon: UserCheck, color: "bg-emerald-50 text-emerald-600 border-emerald-100" },
                { label: "pending_requests", value: hrStats?.pending_leave_requests, icon: CalendarClock, color: "bg-amber-50 text-amber-600 border-amber-100" },
                { label: "estimated_payroll", value: hrStats?.estimated_monthly_payroll, icon: Banknote, color: "bg-rose-50 text-rose-600 border-rose-100", isCurrency: true }
              ].map((stat, i) => (
                <div key={i} className="bg-white p-8 rounded-[2.5rem] border border-slate-100 shadow-sm hover:shadow-2xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden">
                  <div className={`${stat.color} w-16 h-16 rounded-[1.5rem] flex items-center justify-center mb-8 group-hover:scale-110 group-hover:rotate-3 transition-all shadow-inner border`}><stat.icon size={28} /></div>
                  <p className="text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] mb-2">{t(`dashboard.hr.${stat.label}`)}</p>
                  <p className={cn("font-black text-slate-900 tracking-tighter truncate", stat.isCurrency ? "text-2xl font-mono" : "text-5xl")}>
                    {isLoading ? "..." : stat.isCurrency ? formatCurrency(stat.value || 0) : stat.value || 0}
                  </p>
                  <stat.icon className="absolute -right-6 -bottom-6 w-32 h-32 text-slate-50 opacity-10 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
                </div>
              ))}
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-10">
               <div className="bg-slate-900 rounded-[3rem] p-14 text-white relative overflow-hidden group shadow-2xl shadow-slate-200">
                  <div className="relative z-10 h-full flex flex-col items-start">
                    <span className="text-primary-400 text-xs font-black uppercase tracking-[0.3em] mb-6">{t("dashboard.hr.hr_management")}</span>
                    <h3 className="text-5xl font-black mb-8 tracking-tighter leading-[1.1] uppercase">{t("dashboard.hr.hr_management_title")}</h3>
                    <p className="text-slate-400 mb-12 text-lg leading-relaxed max-w-sm font-bold uppercase tracking-tight opacity-80">{t("dashboard.hr.hr_management_desc")}</p>
                    <Link to="/employees" className="bg-white text-slate-900 px-10 py-4 rounded-2xl font-black text-xs uppercase tracking-widest transition-all inline-flex items-center gap-4 hover:gap-6 shadow-2xl hover:shadow-primary-500/20 active:scale-95">
                      {t("dashboard.hr.hr_management_btn").toUpperCase()} <ArrowRight size={20} />
                    </Link>
                  </div>
                  <Users2 className="absolute -right-20 -bottom-20 w-80 h-80 text-primary-500 opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
               </div>

               <div className="bg-primary-600 rounded-[3rem] p-14 text-white relative overflow-hidden group shadow-2xl shadow-primary-900/20">
                  <div className="relative z-10 h-full flex flex-col items-start">
                    <span className="text-primary-200 text-xs font-black uppercase tracking-[0.3em] mb-6">{t("dashboard.hr.payroll")}</span>
                    <h3 className="text-5xl font-black mb-8 tracking-tighter leading-[1.1] uppercase">{t("dashboard.hr.payroll_title")}</h3>
                    <p className="text-primary-100 mb-12 text-lg leading-relaxed max-w-sm font-bold uppercase tracking-tight opacity-80">{t("dashboard.hr.payroll_desc")}</p>
                    <Link to="/time-tracking" className="bg-slate-900 text-white px-10 py-4 rounded-2xl font-black text-xs uppercase tracking-widest transition-all inline-flex items-center gap-4 hover:gap-6 shadow-2xl shadow-black/20 active:scale-95">
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
                { label: "total_agents", value: agents.length, icon: Users, color: "bg-primary-50 text-primary-600 border-primary-100" },
                { label: "online_agents", value: onlineAgents, icon: Activity, color: "bg-emerald-50 text-emerald-600 border-emerald-100" },
                { label: "total_devices", value: devices.length, icon: Cpu, color: "bg-indigo-50 text-indigo-600 border-indigo-100" },
                { label: "active_devices", value: activeDevices, icon: ShieldCheck, color: "bg-amber-50 text-amber-600 border-amber-100" }
              ].map((stat, i) => (
                <div key={i} className="bg-white p-8 rounded-[2.5rem] border border-slate-100 shadow-sm hover:shadow-2xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden">
                  <div className={`${stat.color} w-16 h-16 rounded-[1.5rem] flex items-center justify-center mb-8 group-hover:scale-110 group-hover:rotate-3 transition-all shadow-inner border`}><stat.icon size={28} /></div>
                  <p className="text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] mb-2">{t(`dashboard.system.${stat.label}`)}</p>
                  <p className="text-5xl font-black text-slate-900 tracking-tighter">{isLoading ? "..." : stat.value}</p>
                  <stat.icon className="absolute -right-6 -bottom-6 w-32 h-32 text-slate-50 opacity-10 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
                </div>
              ))}
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-10">
              <section className="space-y-8">
                <div className="flex justify-between items-center px-4">
                  <h2 className="text-xs font-black text-slate-400 tracking-[0.3em] uppercase">{t("dashboard.system.edge_agents")}</h2>
                  <span className="text-[9px] font-black text-primary-600 bg-primary-50 px-4 py-1.5 rounded-full uppercase tracking-widest border border-primary-100 shadow-sm">{t("dashboard.system.live_data")}</span>
                </div>
                <div className="space-y-4">
                  {agents.map((agent) => (
                    <div key={agent.agent_id} className="bg-white p-6 px-8 rounded-[2.5rem] border border-slate-100 flex items-center justify-between hover:shadow-2xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden">
                      <div className="flex items-center gap-6">
                        <div className="bg-slate-50 p-5 rounded-[1.5rem] text-slate-400 group-hover:bg-primary-50 group-hover:text-primary-600 transition-all border border-slate-100 shadow-inner">
                          <Monitor size={28} />
                        </div>
                        <div>
                          <h3 className="font-black text-slate-900 text-xl uppercase tracking-tight leading-none">{agent.agent_id}</h3>
                          <p className="text-[10px] font-black uppercase text-slate-400 tracking-widest mt-2 opacity-70">{t("dashboard.system.seen")}: {formatTime(agent.last_seen)}</p>
                        </div>
                      </div>
                      <span className={cn("px-5 py-2 rounded-full text-[10px] font-black uppercase tracking-widest border shadow-sm", agent.is_online ? "bg-emerald-50 text-emerald-600 border-emerald-100" : "bg-slate-50 text-slate-400 border-slate-100 opacity-60")}>
                        {agent.is_online ? t("dashboard.system.connected").toUpperCase() : t("dashboard.system.disconnected").toUpperCase()}
                      </span>
                    </div>
                  ))}
                  {agents.length === 0 && !isLoading && <div className="py-32 text-center border-2 border-dashed border-slate-100 rounded-[3rem] bg-slate-50/30 flex flex-col items-center"><Monitor size={40} className="text-slate-200 mb-4" /><p className="text-[10px] font-black text-slate-300 uppercase tracking-widest">{t("dashboard.system.no_agents")}</p></div>}
                </div>
              </section>

              <section className="space-y-8">
                <h2 className="text-xs font-black text-slate-400 tracking-[0.3em] uppercase px-4">{t("dashboard.system.quick_actions")}</h2>
                <div className="bg-slate-900 rounded-[3rem] p-14 text-white relative overflow-hidden shadow-2xl shadow-slate-200 group">
                  <div className="relative z-10">
                    <h3 className="text-4xl font-black mb-6 tracking-tighter uppercase leading-[1.1]">{t("dashboard.system.device_management")}</h3>
                    <p className="text-slate-400 mb-12 text-lg leading-relaxed max-w-sm font-bold uppercase tracking-tight opacity-80">{t("dashboard.system.device_management_desc")}</p>
                    <Link to="/devices" className="bg-primary-600 hover:bg-primary-500 text-white px-10 py-4 rounded-2xl font-black text-xs uppercase tracking-widest transition-all inline-block shadow-2xl shadow-primary-900/40 active:scale-95">
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
        <div className="bg-rose-50 border-2 border-rose-100 p-10 rounded-[3rem] flex items-center gap-8 text-rose-700 animate-in slide-in-from-bottom-8 shadow-2xl shadow-rose-100/50">
          <div className="bg-white p-5 rounded-[1.5rem] shadow-sm border border-rose-100"><ShieldAlert size={40} className="text-rose-500" /></div>
          <div>
            <p className="font-black text-2xl uppercase tracking-tighter leading-none mb-2">{t("dashboard.connection_error")}</p>
            <p className="font-bold opacity-70 uppercase tracking-tight text-sm">{t("dashboard.connection_error_desc")}</p>
          </div>
        </div>
      )}
    </div>
  );
}
