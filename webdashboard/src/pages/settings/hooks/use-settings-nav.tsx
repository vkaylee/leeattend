import {
  Building2,
  ShieldAlert,
  KeyRound,
  CreditCard,
  AlertTriangle,
  ShieldCheck,
  Clock,
  Banknote,
  CalendarDays,
  Palmtree,
  Cpu,
  Settings2,
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
    icon: <Building2 className="w-4 h-4" />,
    label: t("common.overview"),
    path: ROUTES.SETTINGS.OVERVIEW,
    permissions: ["workspace:settings:read"],
  },
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
];

const getSystemGroup = (t: TFunction): SettingItem[] => [
  {
    description: t("settings.categories.connectivity_desc"),
    icon: <Cpu className="w-4 h-4" />,
    label: t("settings.categories.connectivity"),
    path: ROUTES.SETTINGS.CONNECTIVITY,
    permissions: ["infra:monitor"],
  },
  {
    description: t("common.security_events"),
    icon: <ShieldCheck className="w-4 h-4" />,
    label: t("common.logs"),
    path: ROUTES.SETTINGS.AUDIT_LOGS,
    permissions: ["workspace:settings:read", "infra:monitor"],
  },
];

const getDangerItem = (t: TFunction): SettingItem => ({
  description: t("common.destructive_operations"),
  icon: <AlertTriangle className="w-4 h-4" />,
  label: t("common.danger_zone"),
  path: ROUTES.SETTINGS.DANGER,
  permissions: ["workspace:delete"],
});

export function useSettingsNav() {
  const { t } = useTranslation();
  const { hasAnyPermission } = useMyPermissions();

  const workspaceGroup = useMemo(() => getWorkspaceGroup(t), [t]);
  const accessGroup = useMemo(() => getAccessGroup(t), [t]);
  const operationsGroup = useMemo(() => getOperationsGroup(t), [t]);
  const systemGroup = useMemo(() => getSystemGroup(t), [t]);
  const dangerItem = useMemo(() => getDangerItem(t), [t]);

  const filterItems = (items: SettingItem[]) =>
    items.filter(item => !item.permissions || item.permissions.length === 0 || hasAnyPermission(...item.permissions));

  return {
    accessGroup: filterItems(accessGroup),
    dangerItem: hasAnyPermission(...(dangerItem.permissions ?? [])) ? dangerItem : null,
    operationsGroup: filterItems(operationsGroup),
    systemGroup: filterItems(systemGroup),
    workspaceGroup: filterItems(workspaceGroup),
  };
}
