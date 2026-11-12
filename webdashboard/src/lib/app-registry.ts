/* eslint-disable max-lines -- static app registry (one entry per app); splitting would obscure the master list */

import type React from "react";

import {
  LayoutDashboard,
  UserRound,
  Users,
  Building2,
  FolderTree,
  Scale,
  CalendarCheck,
  Plane,
  ClipboardCheck,
  ArrowLeftRight,
  BarChart3,
  CheckCircle,
  CalendarRange,
  Zap,
  DollarSign,
  Settings,
  UserCircle,
  History,
  Clock,
  Activity,
  Inbox,
  FileText,
  ShieldCheck,
  Palmtree,
} from "lucide-react";

import type { NavGroup } from "@/hooks/use-nav-menu-items";
import type { FeatureFlagName } from "@/lib/feature-flags";

import { ROUTES } from "@/lib/routes";

// ── Types ──────────────────────────────────────────────────────────

export interface AppDefinition {
  descriptionKey: string;
  featureFlag?: FeatureFlagName;
  group: NavGroup;
  icon: React.ElementType;
  id: string;
  labelKey: string;
  permissions?: string[];
  testId: string;
  to: string;
}

// ── Registry ───────────────────────────────────────────────────────

/**
 * Master app registry. Each app maps to a route and has permission guards
 * that mirror the sidebar's filtering logic exactly.
 *
 * Labels reuse existing i18n keys from common.json, employees.json, etc.
 * Descriptions reuse dashboard or settings descriptions where possible.
 */
export const APP_REGISTRY: AppDefinition[] = [
  // ── Dashboard ──────────────────────────────────────────────────
  {
    id: "dashboard",
    icon: LayoutDashboard,
    labelKey: "common.dashboard",
    descriptionKey: "dashboard.welcome",
    to: ROUTES.HOME,
    group: "dashboard",
    testId: "app-card-dashboard",
  },
  {
    id: "dashboard-analytics",
    icon: BarChart3,
    labelKey: "dashboard.catalog.title",
    descriptionKey: "appLauncher.descriptions.charts",
    to: ROUTES.DASHBOARD,
    group: "dashboard",
    testId: "app-card-dashboard-analytics",
  },

  // ── Organization ───────────────────────────────────────────────
  {
    id: "employees",
    icon: UserRound,
    labelKey: "common.employees",
    descriptionKey: "dashboard.hr.hr_management_desc",
    to: ROUTES.EMPLOYEES,
    group: "org",
    permissions: ["employees:read"],
    testId: "app-card-employees",
  },
  {
    id: "people",
    icon: Users,
    labelKey: "members.title",
    descriptionKey: "appLauncher.descriptions.directory",
    to: ROUTES.SETTINGS.MEMBERS,
    group: "settings",
    permissions: ["members:read"],
    testId: "app-card-people",
  },
  {
    id: "departments",
    icon: Building2,
    labelKey: "common.departments",
    descriptionKey: "settings.categories.organization_desc",
    to: ROUTES.DEPARTMENTS,
    group: "org",
    permissions: ["departments:read"],
    testId: "app-card-departments",
  },
  {
    id: "org-chart",
    icon: FolderTree,
    labelKey: "common.org_chart",
    descriptionKey: "appLauncher.descriptions.org_chart",
    to: ROUTES.ORG_CHART,
    group: "org",
    permissions: ["departments:read"],
    testId: "app-card-org-chart",
  },
  {
    id: "contracts",
    icon: Scale,
    labelKey: "common.contracts",
    descriptionKey: "appLauncher.descriptions.econtracts",
    to: ROUTES.CONTRACTS,
    group: "org",
    permissions: ["documents:contract:read"],
    testId: "app-card-contracts",
  },
  {
    id: "econtracts",
    icon: Scale,
    labelKey: "common.econtracts",
    descriptionKey: "appLauncher.descriptions.econtracts",
    to: ROUTES.ECONTRACTS,
    group: "org",
    permissions: ["documents:contract:read"],
    // Backend not wired yet; keep envelope workflow hidden.
    featureFlag: "ff_econtracts",
    testId: "app-card-econtracts",
  },

  // ── Time & Attendance ──────────────────────────────────────────
  {
    id: "attendance-logs",
    icon: CalendarCheck,
    labelKey: "attendance_logs.title",
    descriptionKey: "attendance_logs.subtitle",
    to: ROUTES.ATTENDANCE_LOGS,
    group: "time",
    permissions: ["attendance:logs:read"],
    testId: "app-card-attendance-logs",
  },
  {
    id: "leave-requests",
    icon: Plane,
    labelKey: "leave_requests.title",
    descriptionKey: "leave_requests.subtitle",
    to: ROUTES.LEAVE_REQUESTS,
    group: "time",
    permissions: ["leave:requests:read", "leave:requests:submit_on_behalf", "leave:requests:approve"],
    testId: "app-card-leave-requests",
  },
  {
    id: "leave-balances",
    icon: Scale,
    labelKey: "common.leave_balances",
    descriptionKey: "appLauncher.descriptions.leave_balances",
    to: ROUTES.LEAVE_BALANCES,
    group: "time",
    permissions: ["leave:balances:read"],
    testId: "app-card-leave-balances",
  },
  {
    id: "attendance-adjustments",
    icon: ClipboardCheck,
    labelKey: "common.attendance_confirm",
    descriptionKey: "appLauncher.descriptions.attendance_adjustments",
    to: ROUTES.ATTENDANCE_ADJUSTMENTS,
    group: "time",
    permissions: ["attendance:adjustments:approve", "attendance:adjustments:submit_on_behalf"],
    testId: "app-card-attendance-adjustments",
  },
  {
    id: "shift-swaps",
    icon: ArrowLeftRight,
    labelKey: "shift_swaps.title",
    descriptionKey: "appLauncher.descriptions.shift_swaps",
    to: ROUTES.SHIFT_SWAPS,
    group: "time",
    permissions: ["shift_swaps:manage"],
    testId: "app-card-shift-swaps",
  },
  {
    id: "reports",
    icon: BarChart3,
    labelKey: "common.reports",
    descriptionKey: "reports.subtitle",
    to: ROUTES.REPORTS,
    group: "time",
    permissions: ["reports:read"],
    testId: "app-card-reports",
  },
  {
    id: "approvals",
    icon: Inbox,
    labelKey: "approvals.inbox.title",
    descriptionKey: "approvals.inbox.subtitle",
    to: ROUTES.APPROVALS,
    group: "time",
    permissions: ["leave:requests:read", "overtime:approve", "attendance:adjustments:approve", "shift_swaps:manage"],
    testId: "app-card-approvals",
  },
  {
    id: "holidays",
    icon: Palmtree,
    labelKey: "settings.categories.holidays",
    descriptionKey: "settings.categories.holidays_desc",
    to: ROUTES.HOLIDAYS,
    group: "time",
    permissions: ["holidays:manage", "attendance:logs:read"],
    testId: "app-card-holidays",
  },

  // ── Shifts ─────────────────────────────────────────────────────
  {
    id: "shifts",
    icon: CalendarRange,
    labelKey: "common.shifts",
    descriptionKey: "appLauncher.descriptions.shifts",
    to: ROUTES.SHIFTS,
    group: "shifts",
    permissions: ["shifts:read"],
    testId: "app-card-shifts",
  },
  {
    id: "shifts-rules",
    icon: Zap,
    labelKey: "common.sections.shifts_rules",
    descriptionKey: "shifts.rules.subtitle",
    to: ROUTES.SHIFTS_RULES,
    group: "settings",
    permissions: ["shifts:read"],
    testId: "app-card-shifts-rules",
  },

  // ── Payroll ────────────────────────────────────────────────────
  {
    id: "payroll",
    icon: DollarSign,
    labelKey: "payroll.title",
    descriptionKey: "payroll.subtitle",
    to: ROUTES.PAYROLL,
    group: "payroll",
    permissions: ["payroll:read"],
    testId: "app-card-payroll",
  },
  {
    id: "documents",
    icon: FileText,
    labelKey: "common.my_documents",
    descriptionKey: "appLauncher.descriptions.documents",
    to: ROUTES.DOCUMENTS,
    group: "payroll",
    permissions: ["payroll:read"],
    testId: "app-card-documents",
  },

  // ── System / Admin ─────────────────────────────────────────────
  {
    id: "infra-monitor",
    icon: Activity,
    labelKey: "monitor.title",
    descriptionKey: "appLauncher.descriptions.infra_monitor",
    to: ROUTES.INFRA_MONITOR,
    group: "system",
    permissions: ["infra:monitor"],
    testId: "app-card-infra-monitor",
  },
  {
    id: "settings",
    icon: Settings,
    labelKey: "settings.title",
    descriptionKey: "organization_settings.description",
    to: ROUTES.SETTINGS.ROOT,
    group: "system",
    permissions: ["organization:settings:read", "members:roles:manage"],
    testId: "app-card-settings",
  },
  {
    id: "audit-logs",
    icon: ShieldCheck,
    labelKey: "audit_logs.title",
    descriptionKey: "audit_logs.subtitle",
    to: ROUTES.AUDIT_LOGS,
    group: "system",
    permissions: ["audit_logs:read"],
    testId: "app-card-audit-logs",
  },

  // ── Self-Service (ESS) ─────────────────────────────────────────
  {
    id: "ess-profile",
    icon: UserCircle,
    labelKey: "ess.profile.title",
    descriptionKey: "ess.profile.subtitle",
    to: ROUTES.ESS_PROFILE,
    group: "ess",
    permissions: ["ess:profile"],
    testId: "app-card-ess-profile",
  },
  {
    id: "ess-attendance",
    icon: History,
    labelKey: "ess.attendance.title",
    descriptionKey: "ess.attendance.subtitle",
    to: ROUTES.ESS_ATTENDANCE,
    group: "ess",
    permissions: ["ess:attendance"],
    testId: "app-card-ess-attendance",
  },
  {
    id: "ess-clock",
    icon: Clock,
    labelKey: "ess.clock.title",
    descriptionKey: "ess.clock.subtitle",
    to: ROUTES.ESS_CLOCK,
    group: "ess",
    permissions: ["ess:clock"],
    testId: "app-card-ess-clock",
  },
];

// ── Group display names (i18n keys) ────────────────────────────────

export const GROUP_LABEL_KEYS: Record<NavGroup, string> = {
  dashboard: "common.dashboard",
  org: "common.sections.organization",
  time: "common.sections.time_leave",
  shifts: "common.sections.shifts_management",
  payroll: "common.sections.payroll",
  settings: "common.organization_settings.title",
  system: "settings.title",
  ess: "common.sections.self_service",
};

