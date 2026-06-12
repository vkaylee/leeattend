import { Link, useLocation } from "@tanstack/react-router";
import {
  LayoutDashboard,
  History,
  Settings,
  UserCircle,
  Users,
  Clock,
  Database,
  CheckCircle,
  UserRound,
  CalendarRange,
  Building2,
  X,
  ChevronDown,
  ChevronRight,
  ShieldCheck,
  Scale,
  Server,
  BarChart3,
  Folder,
  FolderTree,
  Timer,
  Wallet,
  Wrench,
  User,
  Plane,
  CalendarCheck,
  ClipboardCheck,
  ArrowLeftRight,
  DollarSign,
  Activity,
  LayoutGrid,
  Zap
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useMyPermissions } from "@/hooks/use-permissions";
import { useProfile } from "@/hooks/use-profile";
import { useWorkspace } from "@/hooks/use-workspace";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";


interface SidebarProps {
  isCollapsed: boolean;
  isOpen: boolean;
  onClose: () => void;
  onShowSwitcher: () => void;
  openSections: Partial<Record<string, boolean>>;
  toggleSection: (section: string) => void;
}

interface MenuItemBase {
  icon?: React.ElementType;
  label: string;
  testId: string;
  to?: string;
}

interface MenuSectionType extends MenuItemBase {
  items: MenuLinkType[];
  section: string;
  sectionIcon: React.ElementType;
}

interface MenuLinkType extends MenuItemBase {
  exact?: boolean;
  permissions?: string[];
  systemRoles?: string[];
  to: string;
}

type MenuItem = MenuSectionType | MenuLinkType;

function SidebarLink({
  to,
  icon: Icon,
  label,
  testId,
  isCollapsed,
  isActive
}: {
  isActive: boolean;
  isCollapsed: boolean;
  label: string;
  testId: string;
  to: string;
  icon?: React.ElementType;
}) {
  return (
    <Link
      className={cn(
        "flex items-center gap-3 px-4 py-3 rounded-2xl transition-all duration-300 group relative",
        isCollapsed ? "justify-center px-0" : "px-4",
        isActive
          ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-300 shadow-sm shadow-primary-100/50"
          : "text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 hover:text-slate-700 dark:hover:text-slate-200"
      )}
      data-testid={testId}
      to={to as "/"}
    >
      <div className={cn(
        "p-2 rounded-xl transition-colors",
        isActive
          ? "bg-white dark:bg-primary-900/30 shadow-sm"
          : "bg-slate-50 dark:bg-slate-800 group-hover:bg-white dark:group-hover:bg-slate-700"
      )}>
        {Icon ? <Icon className="h-4 w-4" /> : null}
      </div>
      {!isCollapsed && <span className="font-bold text-sm truncate">{label}</span>}
    </Link>
  );
}

function SidebarSection({
  item,
  isCollapsed,
  openSections,
  toggleSection,
  location,
}: {
  isCollapsed: boolean;
  item: MenuSectionType;
  location: { pathname: string };
  openSections: Partial<Record<string, boolean>>;
  toggleSection: (section: string) => void;
}) {
  return (
    <div className="space-y-2">
      {!isCollapsed && (
        <Button
          className="flex w-full items-center justify-between px-4 py-3 min-h-[44px] text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] hover:text-primary-600 dark:hover:text-primary-400 transition-colors group"
          data-testid={item.testId}
          onClick={() => { toggleSection(item.section); }}
          variant="ghost"
        >
          <div className="flex items-center gap-2">
            <item.sectionIcon className="h-3 w-3 opacity-50 group-hover:opacity-100" />
            <span>{item.label}</span>
          </div>
          {openSections[item.section] ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}
        </Button>
      )}
      {(openSections[item.section] ?? isCollapsed) ? (
        <div className="space-y-1">
          {item.items.map((subItem) => (
            <SidebarLink
              icon={subItem.icon}
              isActive={location.pathname === subItem.to}
              isCollapsed={isCollapsed}
              key={subItem.to}
              label={subItem.label}
              testId={subItem.testId}
              to={subItem.to}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}

function useSidebarMenuItems(): MenuItem[] {
  const { t } = useTranslation(['common', 'employees', 'attendance', 'leave', 'shift_swaps', 'payroll', 'settings', 'system_roles']);
  return [
    { to: ROUTES.HOME, icon: LayoutDashboard, label: t("common.dashboard"), exact: true, testId: "sidebar-link-dashboard" },
    {
      section: "org", sectionIcon: Folder, label: t("common.sections.organization"), testId: "sidebar-section-org", items: [
        { to: ROUTES.EMPLOYEES, icon: UserRound, label: t("common.employees"), testId: "sidebar-link-employees", permissions: ["employees:read"] },
        { to: ROUTES.PEOPLE, icon: Users, label: t("members.title", { defaultValue: "Directory" }), testId: "sidebar-link-people", permissions: ["members:read"] },
        { to: ROUTES.DEPARTMENTS, icon: Building2, label: t("common.departments"), testId: "sidebar-link-departments", permissions: ["departments:read"] },
        { to: ROUTES.ORG_CHART, icon: FolderTree, label: t("common.org_chart"), testId: "sidebar-link-org-chart", permissions: ["departments:read"] }
      ]
    },
    {
      section: "time", sectionIcon: Timer, label: t("common.sections.time_leave"), testId: "sidebar-section-time", items: [
        { to: ROUTES.ATTENDANCE_LOGS, icon: CalendarCheck, label: t("attendance_logs.title"), testId: "sidebar-link-attendance-logs", permissions: ["attendance:logs:read"] },
        { to: ROUTES.LEAVE_REQUESTS, icon: Plane, label: t("leave_requests.title"), testId: "sidebar-link-leave-requests", permissions: ["leave:requests:read", "leave:requests:submit_on_behalf", "leave:requests:approve"] },
        { to: ROUTES.LEAVE_BALANCES, icon: Scale, label: t("common.leave_balances"), testId: "sidebar-link-leave-balances", permissions: ["leave:balances:read"] },
        { to: ROUTES.ATTENDANCE_ADJUSTMENTS, icon: ClipboardCheck, label: t("common.attendance_confirm"), testId: "sidebar-link-attendance-adjustments", permissions: ["attendance:adjustments:approve", "attendance:adjustments:submit_on_behalf"] },
        { to: ROUTES.SHIFT_SWAPS, icon: ArrowLeftRight, label: t("shift_swaps.title"), testId: "sidebar-link-shift-swaps", permissions: ["shift_swaps:manage"] },
        { to: ROUTES.REPORTS, icon: BarChart3, label: t("common.reports"), testId: "sidebar-link-reports", permissions: ["reports:read"] },
        { to: ROUTES.APPROVALS_OT, icon: CheckCircle, label: "OT Approvals", testId: "sidebar-link-approvals-ot", permissions: ["overtime:approve", "overtime:claim_on_behalf"] }
      ]
    },
    {
      section: "payroll", sectionIcon: Wallet, label: t("common.sections.payroll_shifts"), testId: "sidebar-section-payroll", items: [
        { to: ROUTES.SHIFTS, icon: CalendarRange, label: t("common.shifts"), testId: "sidebar-link-shifts", permissions: ["shifts:read"] },
        { to: ROUTES.WORKING_PATTERNS, icon: LayoutGrid, label: t("patterns.title", "Working Patterns"), testId: "sidebar-link-working-patterns", permissions: ["shifts:read"] },
        { to: ROUTES.ASSIGNMENT_RULES, icon: Zap, label: t("rules.title", "Assignment Rules"), testId: "sidebar-link-assignment-rules", permissions: ["shifts:read"] },
        { to: ROUTES.PAYROLL, icon: DollarSign, label: t("payroll.title"), testId: "sidebar-link-payroll", permissions: ["payroll:read"] }
      ]
    },
    {
      section: "system", sectionIcon: Wrench, label: t("settings.title"), testId: "sidebar-section-system", items: [
        { to: ROUTES.INFRA_MONITOR, icon: Activity, label: t("monitor.title"), testId: "sidebar-link-monitor", systemRoles: ["platform_admin", "support"] },
        { to: ROUTES.SYSTEM_ROLES, icon: ShieldCheck, label: "Platform Admins", testId: "sidebar-link-system-roles", systemRoles: ["platform_admin"] },
        { to: ROUTES.SYSTEM_DB_CLUSTERS, icon: Server, label: "DB Clusters", testId: "sidebar-link-system-db-clusters", systemRoles: ["platform_admin"] },
        { to: ROUTES.SYSTEM_TENANTS, icon: Database, label: t("system_admin.tenants.title"), testId: "sidebar-link-system-tenants", systemRoles: ["platform_admin"] },
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
}

// eslint-disable-next-line complexity
export default function Sidebar({ isOpen, onClose, openSections, toggleSection, isCollapsed }: SidebarProps) {
  const location = useLocation();
  const { hasAnyPermission } = useMyPermissions();
  const { user } = useProfile();
  const { workspace } = useWorkspace();

  const menuItems = useSidebarMenuItems();

  const filteredMenuItems = menuItems
    .map((item: MenuItem) => {
      if ("section" in item) {
        const filteredItems = item.items.filter(
          (subItem: MenuLinkType) => {
            if (subItem.systemRoles && subItem.systemRoles.length > 0) {
              return subItem.systemRoles.some((role: string) => user?.system_roles?.includes(role));
            }
            const permissions = subItem.permissions;
            return !permissions || permissions.length === 0 || hasAnyPermission(...permissions);
          }
        );
        return { ...item, items: filteredItems };
      }
      return item;
    })
    .filter((item: MenuItem) => {
      if ("section" in item) {
        return item.items.length > 0;
      }
      return true;
    });

  return (
    <div className={cn("fixed inset-y-0 left-0 z-50 flex flex-col bg-white dark:bg-slate-950 border-r border-slate-100 dark:border-slate-800 transition-all duration-300 shadow-xl shadow-slate-200/50 dark:shadow-black/30", isCollapsed ? "w-20" : "w-72", !isOpen && "-translate-x-full lg:translate-x-0")}>
      <div className={cn(
        "flex items-center px-6 border-b border-slate-50 dark:border-slate-800 shrink-0 relative",
        workspace?.logo_url && !isCollapsed ? "py-4 min-h-[6rem] items-start" : "h-24 items-center justify-between"
      )}>
        {!isCollapsed && (
          <div className="flex flex-col gap-1.5 w-full">
            {workspace?.logo_url ? (
              <div className="flex flex-col gap-1">
                <div 
                  aria-label={workspace.name || "Workspace Logo"}
                  className="h-12 w-full bg-contain bg-no-repeat bg-left drop-shadow-sm"
                  data-testid="sidebar-workspace-logo"
                  role="img"
                  style={{ backgroundImage: `url(${workspace.logo_url})` }}
                />
                {/* eslint-disable-next-line @typescript-eslint/no-unnecessary-condition */}
                {workspace?.tagline ? (
                  <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] line-clamp-1 mt-1" data-testid="sidebar-workspace-tagline">
                    {workspace.tagline}
                  </span>
                ) : null}
              </div>
            ) : (
              <div className="flex items-center gap-3">
                <div className="h-10 w-10 shrink-0 rounded-2xl bg-gradient-to-tr from-primary-600 to-primary-400 shadow-lg shadow-primary-200 dark:shadow-primary-900/20 flex items-center justify-center text-white font-black text-xl tracking-tighter italic pl-1">
                  {workspace?.name.charAt(0).toUpperCase() ?? "L"}
                </div>
                <div className="flex flex-col overflow-visible pr-4">
                  <span className="text-xl font-black tracking-tighter text-slate-800 dark:text-slate-200 uppercase italic truncate pl-1 pr-4">
                    {workspace?.name ?? <React.Fragment>Lee<span className="text-primary-600 dark:text-primary-400">Attend</span></React.Fragment>}
                  </span>
                  {workspace?.tagline ? (
                    <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] truncate" data-testid="sidebar-workspace-tagline">
                      {workspace.tagline}
                    </span>
                  ) : null}
                </div>
              </div>
            )}
          </div>
        )}

        {isCollapsed && workspace?.logo_url ? (
          <div className="w-full flex justify-center">
            <div 
              aria-label={workspace.name || "Logo"}
              className="w-10 h-10 bg-contain bg-no-repeat bg-center drop-shadow-sm"
              role="img"
              style={{ backgroundImage: `url(${workspace.logo_url})` }}
            />
          </div>
        ) : null}

        {isCollapsed && !workspace?.logo_url ? (
          <div className="w-full flex justify-center">
            <div className="h-10 w-10 shrink-0 rounded-2xl bg-gradient-to-tr from-primary-600 to-primary-400 shadow-lg shadow-primary-200 dark:shadow-primary-900/20 flex items-center justify-center text-white font-black text-xl tracking-tighter italic">
              {workspace?.name.charAt(0).toUpperCase() ?? "L"}
            </div>
          </div>
        ) : null}

        <Button 
          className={cn(
            "lg:hidden p-3 min-w-[44px] min-h-[44px] flex items-center justify-center text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 bg-slate-50 dark:bg-slate-800 rounded-xl transition-colors",
            workspace?.logo_url && !isCollapsed ? "absolute right-4 top-4" : ""
          )} 
          onClick={onClose}
          variant="ghost"
        >
          <X className="h-5 w-5" />
        </Button>
      </div>
      <div className="flex-1 overflow-y-auto custom-scrollbar px-4 py-6 space-y-8">
        {filteredMenuItems.map((item: MenuItem) => (
          <div key={item.label}>
            {"section" in item ? (
              <SidebarSection
                isCollapsed={isCollapsed}
                item={item}
                location={location}
                openSections={openSections}
                toggleSection={toggleSection}
              />
            ) : (
              <SidebarLink
                icon={item.icon}
                isActive={item.exact ? location.pathname === item.to : location.pathname.startsWith(item.to)}
                isCollapsed={isCollapsed}
                label={item.label}
                testId={item.testId}
                to={item.to}
              />
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

