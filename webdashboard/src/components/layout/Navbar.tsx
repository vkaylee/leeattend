import React from "react";
import { useTranslation } from "react-i18next";
import { useLocation } from "react-router-dom";
import { Menu, ChevronRight as BreadcrumbSeparator } from "lucide-react";
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
      "/": t("common.dashboard"),
      "/members": t("members.title"),
      "/employees": t("common.employees"),
      "/departments": t("common.departments"),
      "/time-tracking": t("common.time_tracking"),
      "/attendance-logs": t("common.logs"),
      "/leave-requests": t("leave_requests.title"),
      "/leave-balances": t("common.leave_balances"),
      "/attendance-adjustments": t("common.attendance_confirm"),
      "/reports": t("common.reports"),
      "/shifts": t("common.shifts"),
      "/working-patterns": t("patterns.title"),
      "/assignment-rules": t("rules.title"),
      "/payroll": t("payroll.title"),
      "/agents": t("agents.title"),
      "/devices": t("common.devices"),
      "/workspace/settings": t("settings.title"),
      "/workspace/settings/general": t("settings.categories.general"),
      "/workspace/settings/people": t("settings.categories.people"),
      "/workspace/settings/time": t("settings.categories.time"),
      "/workspace/settings/payroll": t("settings.categories.payroll"),
      "/workspace/settings/connectivity": t("settings.categories.connectivity"),
      "/profile": t("common.profile"),
      "/ess/profile": t("common.my_profile"),
      "/ess/attendance": t("ess.attendance.title"),
    };
    return map[path] || path.split('/').pop()?.replace(/-/g, ' ') || "";
  };

  return (
    <header className="h-16 border-b border-slate-100 bg-white/80 backdrop-blur-md flex items-center justify-between px-4 lg:px-8 gap-4 flex-shrink-0 z-30 sticky top-0">
      <div className="flex items-center gap-4">
        <button 
          className="lg:hidden p-2 text-slate-500 hover:bg-slate-50 rounded-xl active:scale-90 transition-all"
          onClick={onOpenSidebar}
        >
          <Menu className="w-6 h-6" />
        </button>
        
        {/* Breadcrumbs */}
        <nav className="flex items-center gap-2 text-sm font-bold">
          <span className="hidden sm:inline text-slate-400 font-black uppercase text-[10px] tracking-widest">{currentWorkspaceName || "Workspace"}</span>
          <BreadcrumbSeparator className="w-3.5 h-3.5 text-slate-300" />
          <span className="text-slate-900 font-black uppercase text-[10px] tracking-widest">
            {getPageName(location.pathname)}
          </span>
        </nav>
      </div>
      
      <div className="flex items-center gap-4 ml-auto">
        <button className="p-2.5 text-slate-400 hover:text-primary-600 hover:bg-primary-50 rounded-xl transition-all relative group">
          <div className="absolute top-2.5 right-2.5 w-2 h-2 bg-rose-500 rounded-full border-2 border-white shadow-sm group-hover:scale-110 transition-transform"></div>
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2.5" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
          </svg>
        </button>

        <div className="h-8 w-[1px] bg-slate-100 mx-1 hidden sm:block"></div>

        <div className="flex items-center gap-3 pl-1 group cursor-pointer">
          <div className="hidden sm:block text-right">
            <p className="text-xs font-black text-slate-900 leading-none group-hover:text-primary-600 transition-colors uppercase tracking-tight">{t("common.admin")}</p>
            <p className="text-[9px] font-black text-slate-400 mt-1.5 uppercase tracking-[0.15em] opacity-70">{t("common.workspace_owner")}</p>
          </div>
          <div className="w-10 h-10 rounded-2xl bg-primary-50 text-primary-600 flex items-center justify-center font-black text-sm shadow-inner border border-primary-100/50 group-hover:bg-primary-600 group-hover:text-white transition-all duration-500 group-hover:scale-105 group-hover:rotate-3 group-hover:shadow-lg group-hover:shadow-primary-100">
            AD
          </div>
        </div>
      </div>
    </header>
  );
}
