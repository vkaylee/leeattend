import { Cpu, ShieldCheck } from "lucide-react";

import { type TFunction } from "i18next";

import { Typography } from "@/components/ui/typography";
import { type Device } from "@/lib/api";

interface DashboardSystemTabProps {
  activeDevices: number;
  devices: Device[];
  isLoading: boolean;
  t: TFunction;
}

export function DashboardSystemTab({ devices, activeDevices, isLoading, t }: DashboardSystemTabProps) {
  return (
    <div className="space-y-12 animate-in fade-in duration-500">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 sm:gap-8">
        {[
          { label: "total_devices", value: devices.length, icon: Cpu, color: "bg-blue-50 dark:bg-blue-950 text-blue-600 border-blue-100" },
          { label: "active_devices", value: activeDevices, icon: ShieldCheck, color: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800" },
        ].map((stat, index) => (
          <div className="bg-white dark:bg-slate-900 p-5 sm:p-8 rounded-2xl sm:rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm" key={index}>
            <div className={`${stat.color} w-14 h-14 rounded-[1.5rem] flex items-center justify-center mb-6 border`}><stat.icon size={28} /></div>
            <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mb-2" variant="p">{t(`dashboard.system.${stat.label}`)}</Typography>
            <Typography className="text-4xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter" variant="p">{isLoading ? "..." : stat.value}</Typography>
          </div>
        ))}
      </div>
      <section className="space-y-8">
        <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 tracking-[0.3em] uppercase px-4" variant="h2">{t("dashboard.system.quick_actions")}</Typography>
        <div className="bg-slate-900 rounded-3xl p-6 sm:p-10 text-white relative overflow-hidden shadow-2xl">
          <Typography className="text-3xl font-bold mb-6 tracking-tighter uppercase leading-[1.1]" variant="h3">{t("dashboard.system.device_management")}</Typography>
          <span className="text-slate-300 text-sm">{t("dashboard.system.device_management_desc")}</span>
          <Cpu className="absolute -right-16 -bottom-16 w-60 h-60 text-primary-400 opacity-5 -rotate-12" />
        </div>
      </section>
    </div>
  );
}
