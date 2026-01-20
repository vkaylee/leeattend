import React from "react";
import { useTranslation } from "react-i18next";
import { Link, useLocation } from "react-router-dom";
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
  X,
  LogOut,
  ListOrdered,
  ChevronDown,
  ChevronRight,
  Briefcase,
  Zap,
  Globe,
  Palette,
  ShieldCheck,
  Share2,
  HardDrive
} from "lucide-react";
import { cn } from "@/lib/utils";

interface SidebarProps {
  isOpen: boolean;
  onClose: () => void;
  openSections: Record<string, boolean>;
  toggleSection: (section: string) => void;
  onShowSwitcher: () => void;
}

export default function Sidebar({ isOpen, onClose, openSections, toggleSection, onShowSwitcher }: SidebarProps) {
  const { t } = useTranslation();
  const location = useLocation();

  const handleLogout = () => { localStorage.clear(); window.location.href = "/login"; };

  const menuItems = [
    { to: "/", icon: LayoutDashboard, label: t("common.dashboard"), exact: true },
    { section: "org", label: t("common.sections.organization"), items: [
      { to: "/employees", icon: UserRound, label: t("common.employees") },
      { to: "/departments", icon: Building2, label: t("common.departments") },
      { to: "/members", icon: Users, label: t("members.title") }
    ]},
    { section: "time", label: t("common.sections.time_leave"), items: [
      { to: "/time-tracking", icon: Clock, label: t("common.time_tracking") },
      { to: "/attendance-logs", icon: ListOrdered, label: t("common.logs") },
      { to: "/leave-requests", icon: CalendarDays, label: t("leave_requests.title") },
      { to: "/leave-balances", icon: Banknote, label: t("common.leave_balances") },
      { to: "/attendance-adjustments", icon: History, label: t("common.attendance_confirm") },
      { to: "/reports", icon: FileBarChart, label: t("common.reports") }
    ]},
    { section: "payroll", label: t("common.sections.payroll_shifts"), items: [
      { to: "/shifts", icon: CalendarRange, label: t("common.shifts") },
      { to: "/working-patterns", icon: ListOrdered, label: t("patterns.title") },
      { to: "/assignment-rules", icon: Zap, label: t("rules.title") },
      { to: "/payroll", icon: Banknote, label: t("payroll.title") }
    ]},
    { section: "system", label: t("settings.title"), items: [
      { to: "/workspace/settings/general", icon: Globe, label: t("settings.categories.general") },
      { to: "/workspace/settings/people", icon: Users, label: t("settings.categories.people") },
      { to: "/workspace/settings/time", icon: Clock, label: t("settings.categories.time") },
      { to: "/workspace/settings/payroll", icon: Banknote, label: t("settings.categories.payroll") },
      { to: "/workspace/settings/connectivity", icon: Share2, label: t("settings.categories.connectivity") }
    ]},
    { section: "ess", label: t("common.sections.self_service"), items: [
      { to: "/ess/profile", icon: Briefcase, label: t("common.my_profile") },
      { to: "/ess/attendance", icon: CalendarDays, label: t("ess.attendance.title") }
    ]}
  ];

  return (
    <aside className={cn(
      "fixed inset-y-0 left-0 w-72 bg-white border-r border-slate-100 flex flex-col z-50 transition-all duration-500 ease-in-out lg:translate-x-0 lg:static lg:h-screen shadow-2xl shadow-slate-900/5",
      isOpen ? "translate-x-0" : "-translate-x-full"
    )}>
      <div className="p-10 pb-6">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-4 px-1">
            <div className="bg-slate-900 p-2.5 rounded-xl shadow-2xl shadow-slate-400 rotate-3 transform hover:rotate-0 transition-all duration-500 group/logo">
              <Cpu className="text-white w-6 h-6 group-hover/logo:scale-110 transition-transform" />
            </div>
            <div className="flex flex-col">
              <span className="font-black text-2xl tracking-tighter text-slate-900 leading-none uppercase">Attend</span>
              <span className="text-[10px] font-black text-primary-600 uppercase tracking-[0.25em] mt-1.5 opacity-80">Premium Hub</span>
            </div>
          </div>
          <button className="lg:hidden p-3 bg-slate-50 text-slate-400 hover:text-rose-600 rounded-xl active:scale-90 transition-all shadow-sm border border-slate-100" onClick={onClose}><X size={20} /></button>
        </div>
      </div>

      <nav className="flex-1 px-6 space-y-1 overflow-y-auto scrollbar-hide pb-12 scroll-smooth mt-6">
        {menuItems.map((group, idx) => {
          if (!group.section) {
            const isActive = location.pathname === group.to;
            return (
              <Link key={idx} to={group.to!} className={cn("flex items-center gap-4 px-5 py-4 rounded-2xl transition-all duration-500 group relative mb-2", 
                isActive ? "bg-primary-600 text-white shadow-xl shadow-primary-100 scale-[1.02]" : "text-slate-500 hover:bg-slate-50 hover:text-primary-600"
              )}>
                <group.icon className={cn("w-5 h-5 transition-all group-hover:scale-110", isActive ? "text-white" : "text-slate-400 group-hover:text-primary-500")} />
                <span className="font-black text-sm uppercase tracking-tight">{group.label}</span>
                {isActive && <div className="absolute right-4 w-2 h-2 bg-white rounded-full shadow-2xl animate-pulse" />}
              </Link>
            );
          }

          const isSectionOpen = openSections[group.section];
          return (
            <div key={idx} className="pt-6">
              <button onClick={() => toggleSection(group.section!)} className="w-full flex items-center justify-between px-5 py-2.5 text-[9px] font-black text-slate-400 uppercase tracking-[0.3em] hover:text-primary-600 transition-all group/sec border-b border-slate-50/50 mb-3">
                <span className="group-hover/sec:translate-x-1 transition-all">{group.label}</span>
                <div className={cn("w-5 h-5 rounded-lg bg-slate-50 flex items-center justify-center text-slate-300 transition-all", isSectionOpen && "rotate-180 bg-primary-50 text-primary-500")}><ChevronDown size={12} /></div>
              </button>
              <div className={cn("space-y-1.5 overflow-hidden transition-all duration-700 ease-in-out px-1", isSectionOpen ? "max-h-[600px] opacity-100" : "max-h-0 opacity-0 invisible")}>
                {group.items!.map((item, itemIdx) => {
                  const isActive = location.pathname === item.to;
                  return (
                    <Link key={itemIdx} to={item.to} className={cn("flex items-center gap-4 px-5 py-3.5 rounded-xl transition-all group/item relative", 
                      isActive ? "bg-primary-50 text-primary-600 shadow-inner border border-primary-100/50" : "text-slate-500 hover:bg-slate-50 hover:text-primary-600"
                    )}>
                      <div className={cn("w-8 h-8 rounded-lg flex items-center justify-center transition-all", isActive ? "bg-white text-primary-600 shadow-sm" : "bg-transparent text-slate-300 group-hover/item:text-primary-500")}><item.icon size={16} /></div>
                      <span className="font-black text-xs uppercase tracking-tight">{item.label}</span>
                      {isActive && <div className="absolute left-0 w-1.5 h-6 bg-primary-600 rounded-r-full shadow-lg shadow-primary-500/50" />}
                    </Link>
                  );
                })}
              </div>
            </div>
          );
        })}
      </nav>

      <div className="p-8 border-t border-slate-50 space-y-2 bg-slate-50/20">
        <Link to="/profile" className={cn("flex items-center gap-4 px-5 py-3.5 rounded-2xl transition-all group", 
          location.pathname === "/profile" ? "bg-white text-primary-600 shadow-xl border border-slate-100 scale-[1.02]" : "text-slate-500 hover:bg-white hover:text-primary-600 hover:shadow-xl hover:scale-[1.02] border border-transparent hover:border-slate-100"
        )}>
          <UserCircle size={20} className="group-hover:scale-110 transition-transform" />
          <span className="font-black text-xs uppercase tracking-widest">{t("common.profile")}</span>
        </Link>
        <button onClick={onShowSwitcher} className="w-full flex items-center gap-4 px-5 py-3.5 text-slate-500 hover:bg-white hover:text-primary-600 hover:shadow-xl hover:scale-[1.02] rounded-2xl transition-all group font-black text-xs uppercase tracking-widest border border-transparent hover:border-slate-100 active:scale-95">
          <Building2 size={20} className="group-hover:scale-110 transition-transform" />
          <span className="tracking-widest">{t("common.switch_workspace")}</span>
        </button>
        <button onClick={handleLogout} className="w-full flex items-center gap-4 px-5 py-3.5 text-rose-500 hover:bg-rose-50 rounded-2xl transition-all group font-black text-xs uppercase tracking-widest active:scale-95">
          <LogOut size={20} className="group-hover:translate-x-1 transition-transform" />
          <span className="tracking-widest">{t("common.logout")}</span>
        </button>
      </div>
    </aside>
  );
}
