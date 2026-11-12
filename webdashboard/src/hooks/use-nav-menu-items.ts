import type React from "react";

import {
  LayoutDashboard,
  History,
  Settings,
  UserCircle,
  Clock,
  CheckCircle,
  UserRound,
  CalendarRange,
  Building2,
  Scale,
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
  Zap,
  Mail,
  Inbox,
  FileText,
  ShieldCheck,
  ShieldAlert,
  Palmtree,
  HeartHandshake,
  ClipboardList,
  Milestone,
} from "lucide-react";
import { useTranslation } from "react-i18next";

import { useMyPermissions } from "@/hooks/use-permissions";
import { isFeatureEnabled } from "@/lib/feature-flags";
import { ROUTES } from "@/lib/routes";

// ── Types ──────────────────────────────────────────────────────────

export interface MenuLinkType {
  exact?: boolean;
  icon?: React.ElementType;
  label: string;
  permissions?: string[];
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
    "common", "employees", "attendance", "leave", "shift_swaps", "monitor", "audit_logs",
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
    permissions: ["organization:settings:read", "members:roles:assign", "members:roles:manage"],
  };

  const groups: MenuGroupType[] = [
    {
      section: "org",
      sectionIcon: Folder,
      label: t("common.sections.organization"),
      testId: "sidebar-section-org",
      items: [
        { to: ROUTES.EMPLOYEES, icon: UserRound, label: t("common.employees"), testId: "sidebar-link-employees", permissions: ["employees:read"] },
        { to: ROUTES.ASSIGNMENTS, icon: ArrowLeftRight, label: t("common.assignments", { defaultValue: "Phái cử & Luân chuyển" }), testId: "sidebar-link-assignments", permissions: ["employees:read"] },
        { to: ROUTES.WORKING_CONDITIONS, icon: HeartHandshake, label: t("common.working_conditions", { defaultValue: "Điều kiện làm việc" }), testId: "sidebar-link-working-conditions", permissions: ["employees:read"] },
        { to: ROUTES.LIFECYCLE, icon: ClipboardList, label: t("common.lifecycle", { defaultValue: "Quy trình & Vòng đời" }), testId: "sidebar-link-lifecycle", permissions: ["employees:read"] },
        { to: ROUTES.EMPLOYMENT_PHASES, icon: Milestone, label: t("common.employment_phases", { defaultValue: "Employment Phases" }), testId: "sidebar-link-employment-phases", permissions: ["employment_phases:read"] },
        { to: ROUTES.ATTENDANCE_POLICY_ASSIGNMENTS, icon: ShieldCheck, label: t("common.attendance_policy_assignments", { defaultValue: "Attendance Policies" }), testId: "sidebar-link-attendance-policy-assignments", permissions: ["attendance_policy_assignments:read"] },
        { to: ROUTES.DEPARTMENTS, icon: Building2, label: t("common.departments"), testId: "sidebar-link-departments", permissions: ["departments:read"] },
        { to: ROUTES.ORG_CHART, icon: FolderTree, label: t("common.org_chart"), testId: "sidebar-link-org-chart", permissions: ["departments:read"] },
        ...(isFeatureEnabled("ff_econtracts") ? [{ to: ROUTES.ECONTRACTS, icon: Scale, label: t("common.econtracts", { defaultValue: "e-Contracts Hub" }), testId: "sidebar-link-econtracts", permissions: ["employees:update"] }] : []),
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
        { to: ROUTES.HOLIDAYS, icon: Palmtree, label: t("settings.categories.holidays", { defaultValue: "Ngày nghỉ lễ" }), testId: "sidebar-link-holidays", permissions: ["holidays:manage", "attendance:logs:read"] },
        { to: ROUTES.APPROVALS, icon: Inbox, label: t("approvals.inbox.title", "Hộp thư Phê duyệt"), testId: "sidebar-link-approvals", permissions: ["leave:requests:read", "overtime:approve", "attendance:adjustments:approve", "shift_swaps:manage"] },
      ],
    },
    {
      section: "shifts",
      sectionIcon: CalendarRange,
      label: t("common.sections.shifts_management"),
      testId: "sidebar-section-shifts",
      items: [
        { to: ROUTES.SHIFTS, icon: CalendarRange, label: t("common.shifts"), testId: "sidebar-link-shifts", permissions: ["shifts:read"] },
      ],
    },
    {
      section: "payroll",
      sectionIcon: Wallet,
      label: t("common.sections.payroll"),
      testId: "sidebar-section-payroll",
      items: [
        { to: ROUTES.PAYROLL, icon: DollarSign, label: t("payroll.title"), testId: "sidebar-link-payroll", permissions: ["payroll:read"] },
        { to: ROUTES.DOCUMENTS, icon: FileText, label: t("common.my_documents", { defaultValue: "Documents" }), testId: "sidebar-link-documents", permissions: ["payroll:read"] },
      ],
    },
    {
      section: "system",
      sectionIcon: Wrench,
      label: t("common.sections.system"),
      testId: "sidebar-section-system",
      items: [
        { to: ROUTES.INFRA_MONITOR, icon: Activity, label: t("monitor.title", { defaultValue: "Worker Monitor" }), testId: "sidebar-link-infra-monitor", permissions: ["infra:monitor"] },
        { to: ROUTES.AUDIT_LOGS, icon: ShieldCheck, label: t("audit_logs.title", { defaultValue: "Audit Logs" }), testId: "sidebar-link-audit-logs", permissions: ["audit_logs:read"] },
        { to: ROUTES.SETTINGS.ROLES, icon: ShieldAlert, label: t("settings.categories.roles", { defaultValue: "Vai trò & Phân quyền" }), testId: "sidebar-link-roles", permissions: ["roles:manage", "members:roles:manage"] },
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
        { to: ROUTES.ESS_OT, icon: Timer, label: t("ess.ot.title", "My OT"), testId: "sidebar-link-ess-ot", permissions: ["ess:ot"] },
        { to: ROUTES.ESS_DOCUMENTS, icon: Folder, label: t("ess.documents.title", "My Documents"), testId: "sidebar-link-ess-documents", permissions: ["ess:documents"] },
      ],
    },
  ];

  return { dashboard, settings, groups };
}

// ── Filtering logic (identical to sidebar.tsx lines 228-249) ───────
// ── Filtering logic ────────────────────────────────────────────────

/** Filter menu groups by standalone organization permissions. */
export function useFilteredMenuItems(): NavMenuStructure {
  const { dashboard, settings, groups } = useNavMenuItems();
  const { hasAnyPermission } = useMyPermissions();

  const filteredGroups = groups
    .map((group) => {
      const filteredItems = group.items.filter((item) => {
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
  ROUTES.EMPLOYEES, ROUTES.DEPARTMENTS,
  ROUTES.ORG_CHART, ROUTES.ECONTRACTS, ROUTES.ASSIGNMENTS,
  ROUTES.WORKING_CONDITIONS, ROUTES.LIFECYCLE, ROUTES.EMPLOYMENT_PHASES,
  ROUTES.ATTENDANCE_POLICY_ASSIGNMENTS,
];
const TIME_PATHS: string[] = [
  ROUTES.ATTENDANCE_LOGS, ROUTES.LEAVE_REQUESTS, ROUTES.ATTENDANCE_ADJUSTMENTS,
  ROUTES.REPORTS, ROUTES.LEAVE_BALANCES, ROUTES.SHIFT_SWAPS, ROUTES.APPROVALS, ROUTES.APPROVALS_OT,
  ROUTES.HOLIDAYS,
];
const SHIFTS_PATHS: string[] = [ROUTES.SHIFTS];
const PAYROLL_PATHS: string[] = [ROUTES.PAYROLL, ROUTES.DOCUMENTS];

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
  if (pathname === ROUTES.AUDIT_LOGS || pathname.startsWith("/system/") || pathname.startsWith("/infra/")) return "system";
  if (pathname.startsWith("/ess/")) return "ess";
  return null;
}
