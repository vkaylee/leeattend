import React from "react";
import { useTranslation } from "react-i18next";
import { Link, useLocation } from "@tanstack/react-router";
import { useMyPermissions } from "@/hooks/use-permissions";
import {
  LayoutDashboard,
  Cpu,
  History,
  Settings,
  UserCheck,
  UserCircle,
  Users,
  Clock,
  CheckCircle,
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
  ArrowLeftRight,
  TimerIcon,
  DollarSign,
  UserCog,
  Activity,
  Palmtree
} from "lucide-react";
import { cn } from "@/lib/utils";
import { ROUTES } from "@/lib/routes";

import { useProfile } from "@/hooks/use-profile";

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
  const { hasAnyPermission } = useMyPermissions();
  const { user } = useProfile();

  const handleLogout = () => { localStorage.clear(); window.location.href = ROUTES.LOGIN; };

  const menuItems = [
    { to: ROUTES.HOME, icon: LayoutDashboard, label: t("common.dashboard"), exact: true, testId: "sidebar-link-dashboard" },
    {
      section: "org", sectionIcon: Folder, label: t("common.sections.organization"), testId: "sidebar-section-org", items: [
        { to: ROUTES.EMPLOYEES, icon: UserRound, label: t("common.employees"), testId: "sidebar-link-employees", permissions: ["employees:read"] },
        { to: ROUTES.DEPARTMENTS, icon: Building2, label: t("common.departments"), testId: "sidebar-link-departments", permissions: ["departments:read"] }
      ]
    },
    {
      section: "time", sectionIcon: Timer, label: t("common.sections.time_leave"), testId: "sidebar-section-time", items: [
        { to: ROUTES.ATTENDANCE_LOGS, icon: CalendarCheck, label: t("attendance_logs.title"), testId: "sidebar-link-attendance-logs", permissions: ["attendance:logs:read"] },
        { to: ROUTES.LEAVE_REQUESTS, icon: Plane, label: t("leave_requests.title"), testId: "sidebar-link-leave-requests", permissions: ["leave:requests:read"] },
        { to: ROUTES.LEAVE_BALANCES, icon: Scale, label: t("common.leave_balances"), testId: "sidebar-link-leave-balances", permissions: ["leave:balances:read"] },
        { to: ROUTES.ATTENDANCE_ADJUSTMENTS, icon: ClipboardCheck, label: t("common.attendance_confirm"), testId: "sidebar-link-attendance-adjustments", permissions: ["attendance:adjustments:approve"] },
        { to: ROUTES.SHIFT_SWAPS, icon: ArrowLeftRight, label: t("shift_swaps.title"), testId: "sidebar-link-shift-swaps", permissions: ["shift_swaps:manage"] },
        { to: ROUTES.REPORTS, icon: BarChart3, label: t("common.reports"), testId: "sidebar-link-reports", permissions: ["reports:read"] },
        { to: ROUTES.APPROVALS_OT, icon: CheckCircle, label: "OT Approvals", testId: "sidebar-link-approvals-ot", permissions: ["overtime:approve"] }
      ]
    },
    {
      section: "payroll", sectionIcon: Wallet, label: t("common.sections.payroll_shifts"), testId: "sidebar-section-payroll", items: [
        { to: ROUTES.SHIFTS, icon: CalendarRange, label: t("common.shifts"), testId: "sidebar-link-shifts", permissions: ["shifts:read"] },
        { to: ROUTES.PAYROLL, icon: DollarSign, label: t("payroll.title"), testId: "sidebar-link-payroll", permissions: ["payroll:read"] }
      ]
    },
    {
      section: "system", sectionIcon: Wrench, label: t("settings.title"), testId: "sidebar-section-system", items: [
        { to: ROUTES.INFRA_MONITOR, icon: Activity, label: t("monitor.title"), testId: "sidebar-link-monitor", systemRoles: ["platform_admin", "support"] },
        { to: ROUTES.SYSTEM_ROLES, icon: ShieldCheck, label: "Platform Admins", testId: "sidebar-link-system-roles", systemRoles: ["platform_admin"] },
        { to: ROUTES.SETTINGS.ROOT, icon: Settings, label: t("settings.title"), testId: "sidebar-link-settings-root", permissions: ["workspace:settings:read", "members:roles:manage"] }
      ]
    },
    {
      section: "ess", sectionIcon: User, label: t("common.sections.self_service"), testId: "sidebar-section-ess", items: [
        { to: ROUTES.ESS_PROFILE, icon: UserCircle, label: t("ess.profile.title"), testId: "sidebar-link-ess-profile", permissions: ["ess:profile"] },
        { to: ROUTES.ESS_ATTENDANCE, icon: History, label: t("ess.attendance.title"), testId: "sidebar-link-ess-attendance", permissions: ["ess:attendance"] },
        { to: ROUTES.ESS_CLOCK, icon: Clock, label: t("ess.clock.title"), testId: "sidebar-link-ess-clock", permissions: ["ess:clock"] }
      ]
    }
  ];

  const filteredMenuItems = menuItems
    .map((item) => {
      if (item.section && item.items) {
        const filteredItems = item.items.filter(
          (subItem: any) => {
            if (subItem.systemRoles && subItem.systemRoles.length > 0) {
              return subItem.systemRoles.some((role: string) => user?.system_roles?.includes(role));
            }
            return !subItem.permissions || subItem.permissions.length === 0 || hasAnyPermission(...subItem.permissions);
          }
        );
        return { ...item, items: filteredItems };
      }
      return item;
    })
    .filter((item) => {
      if (item.section) {
        return item.items && item.items.length > 0;
      }
      return true;
    });

  return (
    <div className={cn("fixed inset-y-0 left-0 z-50 flex flex-col bg-white dark:bg-slate-950 border-r border-slate-100 dark:border-slate-800 transition-all duration-300 shadow-xl shadow-slate-200/50 dark:shadow-black/30", isCollapsed ? "w-20" : "w-72", !isOpen && "-translate-x-full lg:translate-x-0")}>
      <div className="flex h-24 items-center justify-between px-6 border-b border-slate-50 dark:border-slate-800">
        {!isCollapsed && <div className="flex items-center gap-3"><div className="h-10 w-10 rounded-2xl bg-gradient-to-tr from-primary-600 to-indigo-500 shadow-lg shadow-primary-200 flex items-center justify-center text-white font-black text-xl tracking-tighter italic">L</div><span className="text-xl font-black tracking-tighter text-slate-800 dark:text-slate-200 uppercase italic">Lee<span className="text-primary-600 dark:text-primary-400">Attend</span></span></div>}
        <button onClick={onClose} className="lg:hidden p-3 min-w-[44px] min-h-[44px] flex items-center justify-center text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 bg-slate-50 dark:bg-slate-800 rounded-xl transition-colors"><X className="h-5 w-5" /></button>
      </div>
      <div className="flex-1 overflow-y-auto custom-scrollbar px-4 py-6 space-y-8">
        {filteredMenuItems.map((item) => (
          <div key={item.label}>
            {item.section ? (
              <div className="space-y-2">
                {!isCollapsed && <button onClick={() => toggleSection(item.section!)} data-testid={item.testId} className="flex w-full items-center justify-between px-4 py-3 min-h-[44px] text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.25em] hover:text-primary-600 dark:hover:text-primary-400 transition-colors group"><div className="flex items-center gap-2">{item.sectionIcon && <item.sectionIcon className="h-3 w-3 opacity-50 group-hover:opacity-100" />}<span>{item.label}</span></div>{openSections[item.section] ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}</button>}
                {(openSections[item.section] || isCollapsed) && (
                  <div className="space-y-1">
                    {item.items?.map((subItem) => (
                      <Link key={subItem.to} to={subItem.to} data-testid={subItem.testId} className={cn("flex items-center gap-3 px-4 py-3 rounded-2xl transition-all duration-300 group", location.pathname === subItem.to ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-300 shadow-sm shadow-primary-100/50" : "text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 hover:text-slate-700 dark:hover:text-slate-200")}>
                        <div className={cn("p-2 rounded-xl transition-colors", location.pathname === subItem.to ? "bg-white dark:bg-primary-900/30 shadow-sm" : "bg-slate-50 dark:bg-slate-800 group-hover:bg-white dark:group-hover:bg-slate-700")}>{subItem.icon && <subItem.icon className="h-4 w-4" />}</div>
                        {!isCollapsed && <span className="font-bold text-sm truncate">{subItem.label}</span>}
                      </Link>
                    ))}
                  </div>
                )}
              </div>
            ) : (
              <Link to={item.to!} data-testid={item.testId} className={cn("flex items-center gap-3 px-4 py-3 rounded-2xl transition-all duration-300 group", (item.exact ? location.pathname === item.to : location.pathname.startsWith(item.to!)) ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-300 shadow-sm shadow-primary-100/50" : "text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 hover:text-slate-700 dark:hover:text-slate-200")}>
                <div className={cn("p-2 rounded-xl transition-colors", (item.exact ? location.pathname === item.to : location.pathname.startsWith(item.to!)) ? "bg-white dark:bg-primary-900/30 shadow-sm" : "bg-slate-50 dark:bg-slate-800 group-hover:bg-white dark:group-hover:bg-slate-700")}>{item.icon && <item.icon className="h-4 w-4" />}</div>
                {!isCollapsed && <span className="font-bold text-sm truncate">{item.label}</span>}
              </Link>
            )}
          </div>
        ))}
      </div>
      <div className="p-4 border-t border-slate-50 dark:border-slate-800">
        <button onClick={handleLogout} className="flex w-full items-center gap-3 px-4 py-4 text-slate-400 dark:text-slate-500 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30 rounded-2xl transition-all duration-300 group"><div className="p-2 rounded-xl bg-slate-50 dark:bg-slate-800 group-hover:bg-white dark:group-hover:bg-slate-700 transition-colors"><LogOut className="h-4 w-4" /></div>{!isCollapsed && <span className="font-bold text-sm uppercase tracking-widest">{t("common.logout")}</span>}</button>
      </div>
    </div>
  );
}
