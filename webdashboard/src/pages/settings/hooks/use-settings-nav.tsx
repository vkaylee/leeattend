import {
  Building2,
  ShieldAlert,
  KeyRound,
  CreditCard,
  ShieldCheck,
  Clock,
  Banknote,
  CalendarDays,
  Palmtree,
  Cpu,
  Settings2,
  MapPin,
  Workflow,
  Braces,
  ClipboardList,
  Database,
  Scale,
  Tags,
} from "lucide-react";
import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { TFunction } from "i18next";

import { useMyPermissions } from "@/hooks/use-permissions";
import { ROUTES } from "@/lib/routes";

export interface SettingItem {
  description: string;
  icon: React.ReactNode;
  label: string;
  path: string;
  permissions?: string[];
}

const getWorkspaceGroup = (t: TFunction): SettingItem[] => [
  {
    description: t("settings.categories.general_desc"),
    icon: <Settings2 className="w-4 h-4" />,
    label: t("settings.categories.general"),
    path: ROUTES.SETTINGS.GENERAL,
    permissions: ["workspace:settings:read"],
  },
  {
    description: t("settings.categories.billing_desc"),
    icon: <CreditCard className="w-4 h-4" />,
    label: t("common.billing"),
    path: ROUTES.SETTINGS.BILLING,
    permissions: ["workspace:settings:read"],
  },
  {
    description: t("settings.categories.locations_desc", { defaultValue: "Manage work locations and timezones" }),
    icon: <MapPin className="w-4 h-4" />,
    label: t("settings.categories.locations", { defaultValue: "Locations" }),
    path: ROUTES.SETTINGS.LOCATIONS,
    permissions: ["workspace:settings:read"],
  },
  {
    description: t("description", { ns: "custom_fields", defaultValue: "Manage custom fields for employees, employments, and departments." }),
    icon: <Braces className="w-4 h-4" />,
    label: t("title", { ns: "custom_fields", defaultValue: "Custom Fields" }),
    path: ROUTES.SETTINGS.CUSTOM_FIELDS,
    permissions: ["workspace:settings:read"],
  },
];

const getAccessGroup = (t: TFunction): SettingItem[] => [
  {
    description: t("common.members.roles.subtitle"),
    icon: <ShieldAlert className="w-4 h-4" />,
    label: t("rolesPage.title", { defaultValue: "Roles" }),
    path: ROUTES.SETTINGS.ROLES,
    permissions: ["members:roles:manage"],
  },
  {
    description: t("common.members.permissions.modal_subtitle"),
    icon: <KeyRound className="w-4 h-4" />,
    label: t("common.permissions"),
    path: ROUTES.SETTINGS.PERMISSIONS,
    permissions: ["members:roles:manage"],
  },
];

const getOperationsGroup = (t: TFunction): SettingItem[] => [
  {
    description: t("settings.categories.organization_desc", { defaultValue: "Manage departments and employment phases" }),
    icon: <Building2 className="w-4 h-4" />,
    label: t("settings.categories.organization", { defaultValue: "Organization" }),
    path: ROUTES.SETTINGS.ORGANIZATION,
    permissions: ["employees:read", "departments:read"],
  },
  {
    description: t("settings.categories.time_desc"),
    icon: <Clock className="w-4 h-4" />,
    label: t("settings.categories.time"),
    path: ROUTES.SETTINGS.TIME,
    permissions: ["attendance:logs:read"],
  },
  {
    description: t("settings.categories.payroll_desc"),
    icon: <Banknote className="w-4 h-4" />,
    label: t("settings.categories.payroll"),
    path: ROUTES.SETTINGS.PAYROLL,
    permissions: ["payroll:read", "payroll:settings:manage"],
  },
  {
    description: t("settings.categories.leave_desc"),
    icon: <CalendarDays className="w-4 h-4" />,
    label: t("settings.categories.leave"),
    path: ROUTES.SETTINGS.LEAVE,
    permissions: ["leave:policies:manage"],
  },
  {
    description: t("common.public_holidays"),
    icon: <Palmtree className="w-4 h-4" />,
    label: t("common.holidays"),
    path: ROUTES.SETTINGS.HOLIDAYS,
    permissions: ["holidays:manage"],
  },
  {
    description: t("settings.categories.overtime_desc", { defaultValue: "Manage overtime quotas and approvals" }),
    icon: <Clock className="w-4 h-4" />,
    label: t("settings.categories.overtime", { defaultValue: "Overtime" }),
    path: ROUTES.SETTINGS.OVERTIME,
    permissions: ["workspace:settings:read", "approval_workflows:read"],
  },
  {
    description: t("settings.categories.workflows_desc", { defaultValue: "Manage approval workflows for requests" }),
    icon: <Workflow className="w-4 h-4" />,
    label: t("settings.categories.workflows", { defaultValue: "Approval Workflows" }),
    path: ROUTES.SETTINGS.WORKFLOWS,
    permissions: ["workspace:settings:read", "approval_workflows:read"],
  },
  {
    description: t("settings.categories.lifecycle_desc", { defaultValue: "Manage onboarding and offboarding checklist templates" }),
    icon: <ClipboardList className="w-4 h-4" />,
    label: t("settings.categories.lifecycle", { defaultValue: "Lifecycle Checklists" }),
    path: ROUTES.SETTINGS.LIFECYCLE_TEMPLATES,
    permissions: ["workspace:settings:read"],
  },
  {
    description: t("settings.categories.condition_types_desc", { defaultValue: "Configure working condition types available for employees" }),
    icon: <Tags className="w-4 h-4" />,
    label: t("settings.categories.condition_types", { defaultValue: "Condition Types" }),
    path: ROUTES.SETTINGS.CONDITION_TYPES,
    permissions: ["workspace:settings:read"],
  },
];

const getSystemGroup = (t: TFunction): SettingItem[] => [
  {
    description: t("common.security_events"),
    icon: <ShieldCheck className="w-4 h-4" />,
    label: t("common.logs"),
    path: ROUTES.SETTINGS.AUDIT_LOGS,
    permissions: ["workspace:settings:read", "infra:monitor"],
  },
  {
    description: t("settings.categories.storage_desc", { defaultValue: "Manage storage providers and file configuration" }),
    icon: <Database className="w-4 h-4" />,
    label: t("settings.categories.storage", { defaultValue: "Storage" }),
    path: ROUTES.SETTINGS.STORAGE,
    permissions: ["workspace:settings:read"],
  },
  {
    description: t("settings.categories.econtracts_desc", { defaultValue: "Manage e-signing integrations and credentials" }),
    icon: <Scale className="w-4 h-4" />,
    label: t("settings.categories.econtracts", { defaultValue: "e-Contracts Integration" }),
    path: ROUTES.SETTINGS.ECONTRACTS,
    permissions: ["workspace:settings:read"],
  },
];


export function useSettingsNav() {
  const { t } = useTranslation();
  const { hasAnyPermission } = useMyPermissions();

  const workspaceGroup = useMemo(() => getWorkspaceGroup(t), [t]);
  const accessGroup = useMemo(() => getAccessGroup(t), [t]);
  const operationsGroup = useMemo(() => getOperationsGroup(t), [t]);
  const systemGroup = useMemo(() => getSystemGroup(t), [t]);
  const filterItems = (items: SettingItem[]) =>
    items.filter(item => !item.permissions || item.permissions.length === 0 || hasAnyPermission(...item.permissions));

  return {
    accessGroup: filterItems(accessGroup),
    operationsGroup: filterItems(operationsGroup),
    systemGroup: filterItems(systemGroup),
    workspaceGroup: filterItems(workspaceGroup),
  };
}
