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

  const handleLogout = () => {
    localStorage.clear();
    window.location.href = "/login";
  };

  const menuItems = [
    {
      to: "/",
      icon: LayoutDashboard,
      label: t("common.dashboard"),
      exact: true
    },
    {
      section: "org",
      label: t("common.sections.organization"),
      items: [
        { to: "/employees", icon: UserRound, label: t("common.employees") },
        { to: "/departments", icon: Building2, label: t("common.departments") },
        { to: "/members", icon: Users, label: t("members.title") }
      ]
    },
    {
      section: "time",
      label: t("common.sections.time_leave"),
      items: [
        { to: "/time-tracking", icon: Clock, label: t("common.time_tracking") },
        { to: "/attendance-logs", icon: ListOrdered, label: t("common.logs") },
        { to: "/leave-requests", icon: CalendarDays, label: t("leave_requests.title") },
        { to: "/leave-balances", icon: Banknote, label: t("common.leave_balances") },
        { to: "/attendance-adjustments", icon: History, label: t("common.attendance_confirm") },
        { to: "/reports", icon: FileBarChart, label: t("common.reports") }
      ]
    },
    {
      section: "payroll",
      label: t("common.sections.payroll_shifts"),
      items: [
        { to: "/shifts", icon: CalendarRange, label: t("common.shifts") },
        { to: "/working-patterns", icon: ListOrdered, label: t("patterns.title") },
        { to: "/assignment-rules", icon: Zap, label: t("rules.title") },
        { to: "/payroll", icon: Banknote, label: t("payroll.title") }
      ]
    },
    {
      section: "system",
      label: t("settings.title"),
      items: [
        { to: "/workspace/settings/general", icon: Globe, label: t("settings.categories.general") },
        { to: "/workspace/settings/people", icon: Users, label: t("settings.categories.people") },
        { to: "/workspace/settings/time", icon: Clock, label: t("settings.categories.time") },
        { to: "/workspace/settings/payroll", icon: Banknote, label: t("settings.categories.payroll") },
        { to: "/workspace/settings/connectivity", icon: Share2, label: t("settings.categories.connectivity") }
      ]
    },
    {
      section: "ess",
      label: t("common.sections.self_service"),
      items: [
        { to: "/ess/profile", icon: Briefcase, label: t("common.my_profile") },
        { to: "/ess/attendance", icon: CalendarDays, label: t("ess.attendance.title") }
      ]
    }
  ];

  return (
    <aside className={cn(
      "fixed inset-y-0 left-0 w-72 bg-white border-r border-slate-100 flex flex-col z-50 transition-transform duration-300 ease-in-out lg:translate-x-0 lg:static lg:h-screen shadow-premium-sm",
      isOpen ? "translate-x-0" : "-translate-x-full"
    )}>
      <div className="p-8 pb-4">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-3 px-2">
            <div className="bg-primary-600 p-2.5 rounded-xl shadow-xl shadow-primary-200 rotate-3 transform hover:rotate-0 transition-transform duration-500">
              <Cpu className="text-white w-5 h-5" />
            </div>
            <div className="flex flex-col">
              <span className="font-black text-xl tracking-tighter text-slate-900 leading-none">LeeAttend</span>
              <span className="text-[9px] font-black text-slate-400 uppercase tracking-[0.25em] mt-1.5 opacity-70">HRM Intelligence</span>
            </div>
          </div>
          <button className="lg:hidden p-2 text-slate-400 hover:text-slate-600 active:scale-90 transition-all" onClick={onClose}>
            <X className="w-5 h-5" />
          </button>
        </div>
      </div>

      <nav className="flex-1 px-6 space-y-1 overflow-y-auto scrollbar-hide pb-10 scroll-smooth mt-4">
        {menuItems.map((group, idx) => {
          if (!group.section) {
            const isActive = location.pathname === group.to;
            return (
              <Link
                key={idx}
                to={group.to!}
                className={cn(
                  "flex items-center gap-3 px-4 py-3.5 rounded-2xl transition-all duration-300 group relative mb-1",
                  isActive 
                    ? "bg-primary-600 text-white shadow-lg shadow-primary-100" 
                    : "text-slate-500 hover:bg-slate-50 hover:text-primary-600"
                )}
              >
                <group.icon className={cn("w-5 h-5 transition-transform group-hover:scale-110", isActive ? "text-white" : "text-slate-400 group-hover:text-primary-500")} />
                <span className="font-bold text-sm tracking-tight">{group.label}</span>
                {isActive && <div className="absolute right-3 w-1.5 h-1.5 bg-white rounded-full shadow-sm animate-pulse" />}
              </Link>
            );
          }

          const isSectionOpen = openSections[group.section];
          return (
            <div key={idx} className="pt-4">
              <button 
                onClick={() => toggleSection(group.section!)}
                className="w-full flex items-center justify-between px-4 py-2 text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] hover:text-primary-600 transition-colors group/sec"
              >
                <span className="group-hover/sec:translate-x-1 transition-transform">{group.label}</span>
                {isSectionOpen ? <ChevronDown className="w-3 h-3" /> : <ChevronRight className="w-3 h-3" />}
              </button>
              <div className={cn(
                "space-y-1 mt-2 overflow-hidden transition-all duration-500 ease-in-out",
                isSectionOpen ? "max-h-[500px] opacity-100" : "max-h-0 opacity-0 invisible"
              )}>
                {group.items!.map((item, itemIdx) => {
                  const isActive = location.pathname === item.to;
                  return (
                    <Link
                      key={itemIdx}
                      to={item.to}
                      className={cn(
                        "flex items-center gap-3 px-4 py-3 rounded-xl transition-all group/item relative",
                        isActive ? "bg-primary-50 text-primary-600 shadow-sm" : "text-slate-500 hover:bg-slate-50 hover:text-primary-600"
                      )}
                    >
                      <item.icon className={cn("w-4 h-4 transition-all group-hover/item:scale-110", isActive ? "text-primary-600" : "text-slate-300 group-hover/item:text-primary-500")} />
                      <span className="font-bold text-xs tracking-tight">{item.label}</span>
                      {isActive && <div className="absolute left-0 w-1 h-4 bg-primary-600 rounded-r-full" />}
                    </Link>
                  );
                })}
              </div>
            </div>
          );
        })}
      </nav>

      <div className="p-6 border-t border-slate-50 space-y-1 bg-slate-50/30">
        <Link
          to="/profile"
          className={cn(
            "flex items-center gap-3 px-4 py-3 rounded-2xl transition-all group",
            location.pathname === "/profile" ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:bg-white hover:text-primary-600 hover:shadow-sm"
          )}
        >
          <UserCircle className="w-5 h-5 group-hover:scale-110 transition-transform" />
          <span className="font-bold text-sm tracking-tight">{t("common.profile")}</span>
        </Link>
        <button
          onClick={onShowSwitcher}
          className="w-full flex items-center gap-3 px-4 py-3 text-slate-500 hover:bg-white hover:text-primary-600 hover:shadow-sm rounded-2xl transition-all group font-bold text-sm border border-transparent hover:border-slate-100"
        >
          <Building2 className="w-5 h-5 group-hover:scale-110 transition-transform" />
          <span className="tracking-tight">{t("common.switch_workspace")}</span>
        </button>
        <button
          onClick={handleLogout}
          className="w-full flex items-center gap-3 px-4 py-3 text-rose-500 hover:bg-rose-50 rounded-2xl transition-all group font-bold text-sm"
        >
          <LogOut className="w-5 h-5 group-hover:scale-110 transition-transform" />
          <span className="tracking-tight">{t("common.logout")}</span>
        </button>
      </div>
    </aside>
  );
}
