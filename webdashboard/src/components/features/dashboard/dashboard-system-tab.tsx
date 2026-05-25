import { Link } from "@tanstack/react-router";
import { Monitor, ShieldCheck, Activity, Cpu, Users } from "lucide-react";

import { type TFunction } from "i18next";

import { Typography } from "@/components/ui/typography";
import { type AgentStatus, type Device } from "@/lib/api";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

interface DashboardSystemTabProps {
  activeDevices: number;
  agents: AgentStatus[];
  devices: Device[];
  formatTime: (time: string | undefined) => string;
  isLoading: boolean;
  onlineAgents: number;
  t: TFunction;
}

export function DashboardSystemTab({ agents, devices, onlineAgents, activeDevices, isLoading, formatTime, t }: DashboardSystemTabProps) {
  return (
    <div className="space-y-12 animate-in fade-in duration-500">
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 sm:gap-8">
        {[
          { label: "total_agents", value: agents.length, icon: Users, color: "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 border-primary-100 dark:border-primary-800" },
          { label: "online_agents", value: onlineAgents, icon: Activity, color: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" },
          { label: "total_devices", value: devices.length, icon: Cpu, color: "bg-blue-50 dark:bg-blue-950 text-blue-600 border-blue-100" },
          { label: "active_devices", value: activeDevices, icon: ShieldCheck, color: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800" }
        ].map((stat, index) => (
          <div className="bg-white dark:bg-slate-900 p-6 sm:p-8 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm hover:shadow-2xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden" key={index}>
            <div className={`${stat.color} w-14 sm:w-16 h-14 sm:h-16 rounded-[1.5rem] flex items-center justify-center mb-6 sm:mb-8 group-hover:scale-110 group-hover:rotate-3 transition-all shadow-inner border`}><stat.icon size={28} /></div>
            <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mb-2" variant="p">{t(`dashboard.system.${stat.label}`)}</Typography>
            <Typography className="text-4xl sm:text-5xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter" variant="p">{isLoading ? "..." : stat.value}</Typography>
            <stat.icon className="absolute -right-6 -bottom-6 w-28 sm:w-32 h-28 sm:h-32 text-slate-50 opacity-10 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
          </div>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-10">
        <section className="space-y-8">
          <div className="flex justify-between items-center px-4">
            <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 tracking-[0.3em] uppercase" variant="h2">{t("dashboard.system.edge_agents")}</Typography>
            <span className="text-[9px] font-bold text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950 px-4 py-1.5 rounded-full uppercase tracking-widest border border-primary-100 dark:border-primary-800 shadow-sm">{t("dashboard.system.live_data")}</span>
          </div>
          <div className="space-y-4">
            {agents.map((agent) => (
              <div className="bg-white dark:bg-slate-900 p-6 px-8 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 flex items-center justify-between hover:shadow-2xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden" key={agent.agent_id}>
                <div className="flex items-center gap-6">
                  <div className="bg-slate-50 dark:bg-slate-800 p-5 rounded-[1.5rem] text-slate-500 dark:text-slate-400 group-hover:bg-primary-50 dark:group-hover:bg-primary-950 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-all border border-slate-100 dark:border-slate-800 shadow-inner">
                    <Monitor size={28} />
                  </div>
                  <div>
                    <Typography className="font-bold text-slate-700 dark:text-slate-200 text-xl uppercase tracking-tight leading-none" variant="h3">{agent.agent_id}</Typography>
                    <Typography className="text-[10px] font-bold uppercase text-slate-500 dark:text-slate-400 tracking-widest mt-2 opacity-70" variant="p">{t("dashboard.system.seen")}: {formatTime(agent.last_seen)}</Typography>
                  </div>
                </div>
                <span className={cn("px-5 py-2 rounded-full text-[10px] font-bold uppercase tracking-widest border shadow-sm", agent.is_online ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" : "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 opacity-60")}>
                  {agent.is_online ? t("dashboard.system.connected").toUpperCase() : t("dashboard.system.disconnected").toUpperCase()}
                </span>
              </div>
            ))}
            {agents.length === 0 && !isLoading && <div className="py-32 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center"><Monitor className="text-slate-200 dark:text-slate-700 mb-4" size={40} /><Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">{t("dashboard.system.no_agents")}</Typography></div>}
          </div>
        </section>

        <section className="space-y-8">
          <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 tracking-[0.3em] uppercase px-4" variant="h2">{t("dashboard.system.quick_actions")}</Typography>
          <div className="bg-slate-900 rounded-[3rem] p-10 sm:p-14 text-white relative overflow-hidden shadow-2xl shadow-slate-200 dark:shadow-black/40 group">
            <div className="relative z-10">
              <Typography className="text-3xl sm:text-4xl font-bold mb-6 tracking-tighter uppercase leading-[1.1]" variant="h3">{t("dashboard.system.device_management")}</Typography>
              <Typography className="text-slate-500 dark:text-slate-400 mb-8 sm:mb-12 text-base sm:text-lg leading-relaxed max-w-sm font-bold uppercase tracking-tight opacity-80" variant="p">{t("dashboard.system.device_management_desc")}</Typography>
              <Link className="bg-primary-600 hover:bg-primary-50 dark:bg-primary-950 text-white px-10 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest transition-all inline-block shadow-2xl shadow-primary-900/40 active:scale-95" to={ROUTES.SETTINGS.CONNECTIVITY}>
                {t("dashboard.system.device_management_btn").toUpperCase()}
              </Link>
            </div>
            <Cpu className="absolute -right-16 -bottom-16 w-60 sm:w-80 h-60 sm:h-80 text-primary-400 opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
          </div>
        </section>
      </div>
    </div>
  );
}
