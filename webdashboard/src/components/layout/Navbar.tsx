import React from "react";
import { useTranslation } from "react-i18next";
import { useLocation } from "react-router-dom";
import { Menu, ChevronRight as BreadcrumbSeparator, Bell } from "lucide-react";
import { cn } from "@/lib/utils";

interface NavbarProps {
  currentWorkspaceName: string;
  onOpenSidebar: () => void;
}

export default function Navbar({ currentWorkspaceName, onOpenSidebar }: NavbarProps) {
  const { t } = useTranslation();
  const location = useLocation();

  const getPageName = (path: string) => {
    const map: Record<string, string> = {
      "/": t("common.dashboard"), "/members": t("members.title"), "/employees": t("common.employees"), "/departments": t("common.departments"),
      "/attendance-logs": t("common.logs"), "/leave-requests": t("leave_requests.title"),
      "/leave-balances": t("common.leave_balances"), "/attendance-adjustments": t("common.attendance_confirm"), "/reports": t("common.reports"),
      "/workspace/leave/balances": t("common.leave_balances"), "/workspace/reports": t("common.reports"),
      "/shifts": t("common.shifts"), "/working-patterns": t("patterns.title"), "/assignment-rules": t("rules.title"), "/payroll": t("payroll.title"),
      "/agents": t("agents.title"), "/devices": t("common.devices"), "/workspace/settings": t("settings.title"), "/workspace/settings/general": t("settings.categories.general"),
      "/workspace/settings/people": t("settings.categories.people"), "/workspace/settings/time": t("settings.categories.time"),
      "/workspace/settings/payroll": t("settings.categories.payroll"), "/workspace/settings/connectivity": t("settings.categories.connectivity"),
      "/profile": t("common.profile"), "/ess/profile": t("common.my_profile"), "/ess/attendance": t("ess.attendance.title"),
      "/ess/clock": t("ess.clock.title"),
    };
    return map[path] || path.split('/').pop()?.replace(/-/g, ' ') || "";
  };

  return (
    <header className="h-20 border-b border-slate-100 bg-white/80 backdrop-blur-xl flex items-center justify-between px-6 lg:px-10 gap-6 flex-shrink-0 z-30 sticky top-0 shadow-2xl shadow-slate-900/5">
      <div className="flex items-center gap-6">
        <button className="lg:hidden p-3 bg-slate-50 text-slate-500 hover:bg-slate-100 rounded-xl active:scale-90 transition-all border border-slate-100 shadow-sm" onClick={onOpenSidebar}><Menu size={20} /></button>
        
        {/* Premium Breadcrumbs */}
        <nav className="flex items-center gap-3">
          <span className="hidden sm:inline text-slate-400 font-black uppercase text-[10px] tracking-[0.25em] opacity-60">{currentWorkspaceName || "Workspace"}</span>
          <BreadcrumbSeparator className="w-3 h-3 text-slate-300" />
          <span className="text-slate-900 font-black uppercase text-[10px] tracking-[0.25em] bg-slate-100 px-3 py-1 rounded-lg shadow-inner">
            {getPageName(location.pathname).toUpperCase()}
          </span>
        </nav>
      </div>
      
      <div className="flex items-center gap-6 ml-auto">
        <button className="p-3 text-slate-400 hover:text-primary-600 hover:bg-white rounded-[1.25rem] transition-all relative group border border-transparent hover:border-slate-100 hover:shadow-xl hover:shadow-primary-900/5">
          <div className="absolute top-3 right-3 w-2.5 h-2.5 bg-rose-500 rounded-full border-2 border-white shadow-sm group-hover:scale-125 transition-transform animate-pulse"></div>
          <Bell size={20} strokeWidth={2.5} />
        </button>

        <div className="h-10 w-px bg-slate-100 mx-1 hidden sm:block opacity-50"></div>

        <div className="flex items-center gap-4 pl-2 group cursor-pointer active:scale-95 transition-all">
          <div className="hidden sm:block text-right">
            <p className="text-xs font-black text-slate-900 leading-none group-hover:text-primary-600 transition-colors uppercase tracking-tight">{t("common.admin")}</p>
            <p className="text-[9px] font-black text-slate-400 mt-1.5 uppercase tracking-[0.2em] opacity-60 leading-none">{t("common.workspace_owner")}</p>
          </div>
          <div className="w-12 h-12 rounded-[1.25rem] bg-slate-900 text-white flex items-center justify-center font-black text-sm shadow-2xl shadow-slate-400 border border-slate-800 group-hover:bg-primary-600 group-hover:border-primary-500 transition-all duration-500 group-hover:scale-110 group-hover:rotate-3">
            AD
          </div>
        </div>
      </div>
    </header>
  );
}
