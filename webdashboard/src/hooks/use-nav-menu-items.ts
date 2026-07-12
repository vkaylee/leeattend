import type React from "react";

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
  Zap
} from "lucide-react";
import { useTranslation } from "react-i18next";

import { useMyPermissions } from "@/hooks/use-permissions";
import { useProfile } from "@/hooks/use-profile";
import { ROUTES } from "@/lib/routes";

// ── Types ──────────────────────────────────────────────────────────

export interface MenuLinkType {
  exact?: boolean;
  icon?: React.ElementType;
  label: string;
  permissions?: string[];
  systemRoles?: string[];
  testId: string;
  to: string;
}

export interface MenuGroupType {
  items: MenuLinkType[];
  label: string;
  section: string;
  sectionIcon: React.ElementType;
  testId: string;
}

export type NavGroup = "dashboard" | "settings" | "org" | "time" | "shifts" | "payroll" | "system" | "ess";

export interface NavMenuStructure {
  dashboard: MenuLinkType;
  groups: MenuGroupType[];
  settings: MenuLinkType;
}

// ── Menu definitions (extracted from sidebar.tsx) ──────────────────

/**
 * Returns the raw un-filtered menu structure.
 * i18n keys and test IDs are identical to the original sidebar.tsx.
 */
export function useNavMenuItems(): NavMenuStructure {
  const { t } = useTranslation([
    "common", "employees", "attendance", "leave", "shift_swaps",
    "payroll", "settings", "system_roles",
  ]);

  const dashboard: MenuLinkType = {
    to: ROUTES.HOME,
    icon: LayoutDashboard,
    label: t("common.dashboard"),
    exact: true,
    testId: "sidebar-link-dashboard",
  };

  const settings: MenuLinkType = {
    to: ROUTES.SETTINGS.ROOT,
    icon: Settings,
    label: t("settings.title"),
    testId: "sidebar-link-settings",
    permissions: ["workspace:settings:read", "members:roles:manage"],
  };

  const groups: MenuGroupType[] = [
    {
      section: "org",
      sectionIcon: Folder,
      label: t("common.sections.organization"),
      testId: "sidebar-section-org",
      items: [
        { to: ROUTES.EMPLOYEES, icon: UserRound, label: t("common.employees"), testId: "sidebar-link-employees", permissions: ["employees:read"] },
        { to: ROUTES.PEOPLE, icon: Users, label: t("members.title", { defaultValue: "Directory" }), testId: "sidebar-link-people", permissions: ["members:read"] },
        { to: ROUTES.DEPARTMENTS, icon: Building2, label: t("common.departments"), testId: "sidebar-link-departments", permissions: ["departments:read"] },
        { to: ROUTES.ORG_CHART, icon: FolderTree, label: t("common.org_chart"), testId: "sidebar-link-org-chart", permissions: ["departments:read"] },
        { to: ROUTES.ECONTRACTS, icon: Scale, label: t("common.econtracts", { defaultValue: "e-Contracts Hub" }), testId: "sidebar-link-econtracts", permissions: ["employees:update"] },
      ],
    },
    {
      section: "time",
      sectionIcon: Timer,
      label: t("common.sections.time_leave"),
      testId: "sidebar-section-time",
      items: [
        { to: ROUTES.ATTENDANCE_LOGS, icon: CalendarCheck, label: t("attendance_logs.title"), testId: "sidebar-link-attendance-logs", permissions: ["attendance:logs:read"] },
        { to: ROUTES.LEAVE_REQUESTS, icon: Plane, label: t("leave_requests.title"), testId: "sidebar-link-leave-requests", permissions: ["leave:requests:read", "leave:requests:submit_on_behalf", "leave:requests:approve"] },
        { to: ROUTES.LEAVE_BALANCES, icon: Scale, label: t("common.leave_balances"), testId: "sidebar-link-leave-balances", permissions: ["leave:balances:read"] },
        { to: ROUTES.ATTENDANCE_ADJUSTMENTS, icon: ClipboardCheck, label: t("common.attendance_confirm"), testId: "sidebar-link-attendance-adjustments", permissions: ["attendance:adjustments:approve", "attendance:adjustments:submit_on_behalf"] },
        { to: ROUTES.SHIFT_SWAPS, icon: ArrowLeftRight, label: t("shift_swaps.title"), testId: "sidebar-link-shift-swaps", permissions: ["shift_swaps:manage"] },
        { to: ROUTES.REPORTS, icon: BarChart3, label: t("common.reports"), testId: "sidebar-link-reports", permissions: ["reports:read"] },
        { to: ROUTES.APPROVALS_OT, icon: CheckCircle, label: t("ot_approvals.title", "OT Approvals"), testId: "sidebar-link-approvals-ot", permissions: ["overtime:approve", "overtime:claim_on_behalf"] },
      ],
    },
    {
      section: "shifts",
      sectionIcon: CalendarRange,
      label: t("common.sections.shifts_management"),
      testId: "sidebar-section-shifts",
      items: [
        { to: ROUTES.SHIFTS, icon: CalendarRange, label: t("common.shifts"), testId: "sidebar-link-shifts", permissions: ["shifts:read"] },
        { to: ROUTES.SHIFTS_RULES, icon: Zap, label: t("common.sections.shifts_rules", "Automation Policy"), testId: "sidebar-link-shifts-rules", permissions: ["shifts:read"] },
      ],
    },
    {
      section: "payroll",
      sectionIcon: Wallet,
      label: t("common.sections.payroll"),
      testId: "sidebar-section-payroll",
      items: [
        { to: ROUTES.PAYROLL, icon: DollarSign, label: t("payroll.title"), testId: "sidebar-link-payroll", permissions: ["payroll:read"] },
      ],
    },
    {
      section: "system",
      sectionIcon: Wrench,
      label: t("common.sections.system"),
      testId: "sidebar-section-system",
      items: [
        { to: ROUTES.INFRA_MONITOR, icon: Activity, label: t("monitor.title"), testId: "sidebar-link-monitor", systemRoles: ["platform_admin", "support"] },
        { to: ROUTES.SYSTEM_ROLES, icon: ShieldCheck, label: t("system_roles.title", "Platform Admins"), testId: "sidebar-link-system-roles", systemRoles: ["platform_admin"] },
        { to: ROUTES.SYSTEM_DB_CLUSTERS, icon: Server, label: t("db_clusters.title", "DB Clusters"), testId: "sidebar-link-system-db-clusters", systemRoles: ["platform_admin"] },
        { to: ROUTES.SYSTEM_TENANTS, icon: Database, label: t("system_admin.tenants.title"), testId: "sidebar-link-system-tenants", systemRoles: ["platform_admin"] },
      ],
    },
    {
      section: "ess",
      sectionIcon: User,
      label: t("common.sections.self_service"),
      testId: "sidebar-section-ess",
      items: [
        { to: ROUTES.ESS_PROFILE, icon: UserCircle, label: t("ess.profile.title"), testId: "sidebar-link-ess-profile", permissions: ["ess:profile"] },
        { to: ROUTES.ESS_ATTENDANCE, icon: History, label: t("ess.attendance.title"), testId: "sidebar-link-ess-attendance", permissions: ["ess:attendance"] },
        { to: ROUTES.ESS_CLOCK, icon: Clock, label: t("ess.clock.title"), testId: "sidebar-link-ess-clock", permissions: ["ess:clock"] },
      ],
    },
  ];

  return { dashboard, settings, groups };
}

// ── Filtering logic (identical to sidebar.tsx lines 228-249) ───────

/**
 * Filters menu groups by the current user's permissions and system roles.
 * Mirrors the exact logic from sidebar.tsx.
 */
export function useFilteredMenuItems(): NavMenuStructure {
  const { dashboard, settings, groups } = useNavMenuItems();
  const { hasAnyPermission } = useMyPermissions();
  const { user } = useProfile();

  const filteredGroups = groups
    .map((group) => {
      const filteredItems = group.items.filter((item) => {
        // System role check first (mirrors sidebar.tsx)
        if (item.systemRoles && item.systemRoles.length > 0) {
          return item.systemRoles.some((role) => user?.system_roles?.includes(role));
        }
        // Permission check
        const permissions = item.permissions;
        return !permissions || permissions.length === 0 || hasAnyPermission(...permissions);
      });
      return { ...group, items: filteredItems };
    })
    .filter((group) => group.items.length > 0);

  return { dashboard, settings, groups: filteredGroups };
}

// ── Path-based active group detection ──────────────────────────────

const ORG_PATHS: string[] = [
  ROUTES.PEOPLE, ROUTES.EMPLOYEES, ROUTES.DEPARTMENTS,
  ROUTES.ORG_CHART, ROUTES.ECONTRACTS,
];
const TIME_PATHS: string[] = [
  ROUTES.ATTENDANCE_LOGS, ROUTES.LEAVE_REQUESTS, ROUTES.ATTENDANCE_ADJUSTMENTS,
  ROUTES.REPORTS, ROUTES.LEAVE_BALANCES, ROUTES.SHIFT_SWAPS, ROUTES.APPROVALS_OT,
];
const SHIFTS_PATHS: string[] = [ROUTES.SHIFTS, ROUTES.SHIFTS_RULES];
const PAYROLL_PATHS: string[] = [ROUTES.PAYROLL];

/**
 * Determines which nav group is active based on the current route path.
 * Used by IconRail to highlight the active nav group.
 */
// eslint-disable-next-line complexity
export function getActiveNavGroup(pathname: string): NavGroup | null {
  if (pathname === ROUTES.HOME || pathname === ROUTES.DASHBOARD) return "dashboard";
  if (ORG_PATHS.some((p) => pathname === p || pathname.startsWith(p + "/"))) return "org";
  if (TIME_PATHS.some((p) => pathname === p || pathname.startsWith(p + "/"))) return "time";
  if (SHIFTS_PATHS.some((p) => pathname === p || pathname.startsWith(p + "/"))) return "shifts";
  if (PAYROLL_PATHS.some((p) => pathname === p || pathname.startsWith(p + "/"))) return "payroll";
  if (pathname.startsWith(ROUTES.SETTINGS.ROOT)) return "settings";
  if (pathname.startsWith("/system/") || pathname.startsWith("/infra/")) return "system";
  if (pathname.startsWith("/ess/")) return "ess";
  return null;
}
