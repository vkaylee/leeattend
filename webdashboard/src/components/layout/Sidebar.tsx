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
  HardDrive,
  Scale,
  BarChart3,
  Folder,
  Timer,
  Wallet,
  Wrench,
  User,
  FileText,
  Plane,
  CalendarCheck,
  ClipboardCheck,
  TimerIcon,
  DollarSign,
  UserCog
} from "lucide-react";
import { cn } from "@/lib/utils";

interface SidebarProps {
  isOpen: boolean;
  onClose: () => void;
  openSections: Record<string, boolean>;
  toggleSection: (section: string) => void;
  onShowSwitcher: () => void;
  isCollapsed: boolean;
}

export default function Sidebar({ isOpen, onClose, openSections, toggleSection, onShowSwitcher, isCollapsed }: SidebarProps) {
  const { t } = useTranslation();
  const location = useLocation();

  const handleLogout = () => { localStorage.clear(); window.location.href = "/login"; };

  const menuItems = [
    { to: "/", icon: LayoutDashboard, label: t("common.dashboard"), exact: true, testId: "sidebar-link-dashboard" },
    { section: "org", sectionIcon: Folder, label: t("common.sections.organization"), testId: "sidebar-section-org", items: [
      { to: "/employees", icon: UserRound, label: t("common.employees"), testId: "sidebar-link-employees" },
      { to: "/departments", icon: Building2, label: t("common.departments"), testId: "sidebar-link-departments" },
      { to: "/members", icon: Users, label: t("members.title"), testId: "sidebar-link-members" }
    ]},
    { section: "time", sectionIcon: Timer, label: t("common.sections.time_leave"), testId: "sidebar-section-time", items: [
      { to: "/attendance-logs", icon: CalendarCheck, label: t("attendance_logs.title"), testId: "sidebar-link-attendance-logs" },
      { to: "/leave-requests", icon: Plane, label: t("leave_requests.title"), testId: "sidebar-link-leave-requests" },
      { to: "/workspace/leave/balances", icon: Scale, label: t("common.leave_balances"), testId: "sidebar-link-leave-balances" },
      { to: "/attendance-adjustments", icon: ClipboardCheck, label: t("common.attendance_confirm"), testId: "sidebar-link-attendance-adjustments" },
      { to: "/workspace/reports", icon: BarChart3, label: t("common.reports"), testId: "sidebar-link-reports" }
    ]},
    { section: "payroll", sectionIcon: Wallet, label: t("common.sections.payroll_shifts"), testId: "sidebar-section-payroll", items: [
      { to: "/shifts", icon: CalendarRange, label: t("common.shifts"), testId: "sidebar-link-shifts" },
      { to: "/working-patterns", icon: ListOrdered, label: t("patterns.title"), testId: "sidebar-link-working-patterns" },
      { to: "/assignment-rules", icon: Zap, label: t("rules.title"), testId: "sidebar-link-assignment-rules" },
      { to: "/payroll", icon: DollarSign, label: t("payroll.title"), testId: "sidebar-link-payroll" }
    ]},
    { section: "system", sectionIcon: Wrench, label: t("settings.title"), testId: "sidebar-section-system", items: [
      { to: "/workspace/settings/general", icon: Globe, label: t("settings.categories.general"), testId: "sidebar-link-settings-general" },
      { to: "/workspace/settings/people", icon: UserCog, label: t("settings.categories.people"), testId: "sidebar-link-settings-people" },
      { to: "/workspace/settings/time", icon: TimerIcon, label: t("settings.categories.time"), testId: "sidebar-link-settings-time" },
      { to: "/workspace/settings/payroll", icon: Banknote, label: t("settings.categories.payroll"), testId: "sidebar-link-settings-payroll" },
      { to: "/workspace/settings/connectivity", icon: Share2, label: t("settings.categories.connectivity"), testId: "sidebar-link-settings-connectivity" }
    ]},
    { section: "ess", sectionIcon: User, label: t("common.sections.self_service"), testId: "sidebar-section-ess", items: [
      { to: "/ess/clock", icon: Clock, label: t("ess.clock.title"), testId: "sidebar-link-ess-clock" },
      { to: "/ess/profile", icon: Briefcase, label: t("common.my_profile"), testId: "sidebar-link-ess-profile" },
      { to: "/ess/attendance", icon: CalendarDays, label: t("ess.attendance.title"), testId: "sidebar-link-ess-attendance" }
    ]}
  ];

  return (
    <aside className={cn(
      "fixed inset-y-0 left-0 bg-white border-r border-slate-100 flex flex-col z-50 transition-all duration-500 ease-in-out lg:translate-x-0 lg:static lg:h-screen shadow-2xl shadow-slate-900/5",
      isOpen ? "translate-x-0" : "-translate-x-full",
      isCollapsed ? "lg:w-20 w-72" : "w-72"
    )}>
      <div className={cn("pb-6 transition-all duration-500", isCollapsed ? "lg:p-4 p-10" : "p-10")}>
        <div className="flex items-center justify-between mb-4">
          <div className={cn("flex items-center gap-4 px-1 transition-all duration-500", isCollapsed && "lg:justify-center lg:px-0")}>
            <div className="bg-slate-900 p-2.5 rounded-xl shadow-2xl shadow-slate-400 rotate-3 transform hover:rotate-0 transition-all duration-500 group/logo">
              <Cpu className="text-white w-6 h-6 group-hover/logo:scale-110 transition-transform" />
            </div>
            {!isCollapsed && (
              <div className="flex flex-col">
                <span className="font-black text-2xl tracking-tighter text-slate-900 leading-none uppercase">Attend</span>
                <span className="text-[10px] font-black text-primary-600 uppercase tracking-[0.25em] mt-1.5 opacity-80">Premium Hub</span>
              </div>
            )}
          </div>
          <button className="lg:hidden p-3 bg-slate-50 text-slate-400 hover:text-rose-600 rounded-xl active:scale-90 transition-all shadow-sm border border-slate-100" onClick={onClose}><X size={20} /></button>
        </div>
      </div>

      <nav className={cn("flex-1 space-y-1 overflow-y-auto scrollbar-hide pb-12 scroll-smooth mt-6 transition-all duration-500", isCollapsed ? "lg:px-2 px-6" : "px-6")}>
        {menuItems.map((group, idx) => {
          if (!group.section) {
            const isActive = location.pathname === group.to;
            const Icon = group.icon as any;
            return (
              <Link
                key={idx}
                to={group.to!}
                data-testid={group.testId}
                title={isCollapsed ? group.label : undefined}
                className={cn(
                  "flex items-center gap-4 rounded-2xl transition-all duration-500 group relative mb-2",
                  isActive ? "bg-primary-600 text-white shadow-xl shadow-primary-100 scale-[1.02]" : "text-slate-500 hover:bg-slate-50 hover:text-primary-600",
                  isCollapsed ? "lg:justify-center lg:px-3 lg:py-3 px-5 py-4" : "px-5 py-4"
                )}
              >
                <Icon className={cn("transition-all group-hover:scale-110", isActive ? "text-white" : "text-slate-400 group-hover:text-primary-500", isCollapsed ? "lg:w-6 lg:h-6 w-5 h-5" : "w-5 h-5")} />
                {!isCollapsed && <span className="font-black text-sm uppercase tracking-tight">{group.label}</span>}
                {isActive && !isCollapsed && <div className="absolute right-4 w-2 h-2 bg-white rounded-full shadow-2xl animate-pulse" />}
              </Link>
            );
          }

          const isSectionOpen = openSections[group.section];
          const SectionIcon = group.sectionIcon as any;
          return (
            <div key={idx} className="pt-6">
              <button
                onClick={() => toggleSection(group.section!)}
                data-testid={group.testId}
                title={isCollapsed ? group.label : undefined}
                className={cn(
                  "w-full flex items-center justify-between font-black text-slate-400 uppercase hover:text-primary-600 transition-all group/sec border-b border-slate-50/50 mb-3 tracking-[0.3em]",
                  isCollapsed ? "lg:justify-center lg:px-2 lg:py-3 px-5 py-2.5 text-[9px]" : "px-5 py-2.5 text-[9px]"
                )}
              >
                {isCollapsed ? (
                  <SectionIcon className="hidden lg:block w-5 h-5 transition-all" />
                ) : (
                  <>
                    <span className="group-hover/sec:translate-x-1 transition-all">{group.label}</span>
                    <div className={cn("w-5 h-5 rounded-lg bg-slate-50 flex items-center justify-center text-slate-300 transition-all", isSectionOpen && "rotate-180 bg-primary-50 text-primary-500")}><ChevronDown size={12} /></div>
                  </>
                )}
              </button>
              <div className={cn("space-y-1.5 overflow-hidden transition-all duration-700 ease-in-out", isCollapsed ? "lg:px-0 px-1" : "px-1", isSectionOpen || isCollapsed ? "max-h-[600px] opacity-100" : "max-h-0 opacity-0 invisible")}>
                {group.items!.map((item, itemIdx) => {
                  const isActive = location.pathname === item.to;
                  return (
                    <Link
                      key={itemIdx}
                      to={item.to}
                      data-testid={item.testId}
                      title={isCollapsed ? item.label : undefined}
                      className={cn(
                        "flex items-center gap-4 rounded-xl transition-all group/item relative",
                        isActive ? "bg-primary-50 text-primary-600 shadow-inner border border-primary-100/50" : "text-slate-500 hover:bg-slate-50 hover:text-primary-600",
                        isCollapsed ? "lg:justify-center lg:px-3 lg:py-3 px-5 py-3.5" : "px-5 py-3.5"
                      )}
                    >
                      <div className={cn("rounded-lg flex items-center justify-center transition-all", isActive ? "bg-white text-primary-600 shadow-sm" : "bg-transparent text-slate-300 group-hover/item:text-primary-500", isCollapsed ? "lg:w-auto lg:h-auto w-8 h-8" : "w-8 h-8")}>
                        <item.icon className={cn("transition-all", isCollapsed ? "lg:w-6 lg:h-6 w-4 h-4" : "w-4 h-4")} />
                      </div>
                      {!isCollapsed && <span className="font-black text-xs uppercase tracking-tight">{item.label}</span>}
                      {isActive && !isCollapsed && <div className="absolute left-0 w-1.5 h-6 bg-primary-600 rounded-r-full shadow-lg shadow-primary-500/50" />}
                    </Link>
                  );
                })}
              </div>
            </div>
          );
        })}
      </nav>

      <div className={cn("border-t border-slate-50 space-y-2 bg-slate-50/20 transition-all duration-500", isCollapsed ? "lg:p-2 p-8" : "p-8")}>
        <Link
          to="/profile"
          data-testid="sidebar-link-profile"
          title={isCollapsed ? t("common.profile") : undefined}
          className={cn(
            "flex items-center gap-4 rounded-2xl transition-all group",
            location.pathname === "/profile" ? "bg-white text-primary-600 shadow-xl border border-slate-100 scale-[1.02]" : "text-slate-500 hover:bg-white hover:text-primary-600 hover:shadow-xl hover:scale-[1.02] border border-transparent hover:border-slate-100",
            isCollapsed ? "lg:justify-center lg:px-3 lg:py-3 px-5 py-3.5" : "px-5 py-3.5"
          )}
        >
          <UserCircle size={isCollapsed ? 24 : 20} className="group-hover:scale-110 transition-transform" />
          {!isCollapsed && <span className="font-black text-xs uppercase tracking-widest">{t("common.profile")}</span>}
        </Link>
        <button
          onClick={onShowSwitcher}
          data-testid="sidebar-btn-switcher"
          title={isCollapsed ? t("common.switch_workspace") : undefined}
          className={cn(
            "w-full flex items-center gap-4 text-slate-500 hover:bg-white hover:text-primary-600 hover:shadow-xl hover:scale-[1.02] rounded-2xl transition-all group font-black text-xs uppercase tracking-widest border border-transparent hover:border-slate-100 active:scale-95",
            isCollapsed ? "lg:justify-center lg:px-3 lg:py-3 px-5 py-3.5" : "px-5 py-3.5"
          )}
        >
          <Building2 size={isCollapsed ? 24 : 20} className="group-hover:scale-110 transition-transform" />
          {!isCollapsed && <span className="tracking-widest">{t("common.switch_workspace")}</span>}
        </button>
        <button
          onClick={handleLogout}
          data-testid="sidebar-btn-logout"
          title={isCollapsed ? t("common.logout") : undefined}
          className={cn(
            "w-full flex items-center gap-4 text-rose-500 hover:bg-rose-50 rounded-2xl transition-all group font-black text-xs uppercase tracking-widest active:scale-95",
            isCollapsed ? "lg:justify-center lg:px-3 lg:py-3 px-5 py-3.5" : "px-5 py-3.5"
          )}
        >
          <LogOut size={isCollapsed ? 24 : 20} className="group-hover:translate-x-1 transition-transform" />
          {!isCollapsed && <span className="tracking-widest">{t("common.logout")}</span>}
        </button>
      </div>
    </aside>
  );
}