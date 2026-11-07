import {
  Building2,
  ShieldAlert,
  KeyRound,
  CreditCard,
  Clock,
  Banknote,
  CalendarDays,
  Cpu,
  Settings2,
  MapPin,
  Workflow,
  Braces,
  ClipboardList,
  Database,
  Scale,
  Tags,
  Mail,
  Users,
  Zap,
} from "lucide-react";
import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { TFunction } from "i18next";

import { useMyPermissions } from "@/hooks/use-permissions";
import { isFeatureEnabled, type FeatureFlagName } from "@/lib/feature-flags";
import { ROUTES } from "@/lib/routes";

export interface SettingItem {
  description: string;
  featureFlag?: FeatureFlagName;
  icon: React.ReactNode;
  label: string;
  path: string;
  permissions?: string[];
}

const getWorkspaceGroup = (t: TFunction): SettingItem[] => [
  {
    description: t("settings.categories.general_desc", { defaultValue: "Cấu hình chung và thông tin tổ chức" }),
    icon: <Settings2 className="w-4 h-4" />,
    label: t("settings.categories.general", { defaultValue: "Cài đặt chung" }),
    path: ROUTES.SETTINGS.GENERAL,
    permissions: ["workspace:settings:read"],
  },
  {
    description: t("settings.categories.locations_desc", { defaultValue: "Quản lý các chi nhánh và địa điểm làm việc" }),
    icon: <MapPin className="w-4 h-4" />,
    label: t("settings.categories.locations", { defaultValue: "Địa điểm làm việc" }),
    path: ROUTES.SETTINGS.LOCATIONS,
    permissions: ["workspace:settings:read"],
  },
  {
    description: t("settings.categories.custom_fields_desc", { defaultValue: "Quản lý trường dữ liệu tùy chỉnh cho nhân sự" }),
    icon: <Braces className="w-4 h-4" />,
    label: t("settings.categories.custom_fields", { defaultValue: "Trường tùy chỉnh" }),
    path: ROUTES.SETTINGS.CUSTOM_FIELDS,
    permissions: ["workspace:settings:read"],
  },
];

const getAccessGroup = (t: TFunction): SettingItem[] => [
  {
    description: t("settings.categories.roles_desc", { defaultValue: "Quản lý vai trò và phân quyền chức vụ" }),
    icon: <ShieldAlert className="w-4 h-4" />,
    label: t("settings.categories.roles", { defaultValue: "Vai trò & Chức danh" }),
    path: ROUTES.SETTINGS.ROLES,
    permissions: ["members:roles:manage"],
  },
  {
    description: t("settings.categories.permissions_desc", { defaultValue: "Cấp phát hoặc thu hồi quyền hạn cá nhân" }),
    icon: <KeyRound className="w-4 h-4" />,
    label: t("settings.categories.permissions", { defaultValue: "Phân quyền người dùng" }),
    path: ROUTES.SETTINGS.PERMISSIONS,
    permissions: ["members:roles:manage"],
  },
  {
    description: t("settings.categories.members_desc", { defaultValue: "Quản lý tài khoản người dùng và lời mời" }),
    icon: <Users className="w-4 h-4" />,
    label: t("members.title", { defaultValue: "Tài khoản người dùng" }),
    path: ROUTES.PEOPLE,
    permissions: ["members:read"],
  },
];

const getOperationsGroup = (t: TFunction): SettingItem[] => [
  {
    description: t("settings.categories.organization_desc", { defaultValue: "Quản lý pháp nhân và lộ trình công tác" }),
    icon: <Building2 className="w-4 h-4" />,
    label: t("settings.categories.organization", { defaultValue: "Pháp nhân & Mốc công tác" }),
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
    description: t("settings.categories.overtime_desc", { defaultValue: "Quản lý hạn mức tăng ca và luồng phê duyệt OT" }),
    icon: <Clock className="w-4 h-4" />,
    label: t("settings.categories.overtime", { defaultValue: "Quy định Tăng ca" }),
    path: ROUTES.SETTINGS.OVERTIME,
    permissions: ["workspace:settings:read", "approval_workflows:read"],
  },
  {
    description: t("settings.categories.workflows_desc", { defaultValue: "Thiết lập quy trình phê duyệt đa cấp" }),
    icon: <Workflow className="w-4 h-4" />,
    label: t("settings.categories.workflows", { defaultValue: "Quy trình phê duyệt" }),
    path: ROUTES.SETTINGS.WORKFLOWS,
    permissions: ["workspace:settings:read", "approval_workflows:read"],
  },
  {
    description: t("settings.categories.lifecycle_desc", { defaultValue: "Mẫu danh mục thủ tục tiếp nhận và bàn giao" }),
    icon: <ClipboardList className="w-4 h-4" />,
    label: t("settings.categories.lifecycle", { defaultValue: "Quy trình Onboarding" }),
    path: ROUTES.SETTINGS.LIFECYCLE_TEMPLATES,
    permissions: ["workspace:settings:read"],
  },
  {
    description: t("settings.categories.condition_types_desc", { defaultValue: "Cấu hình điều kiện làm việc đặc thù" }),
    icon: <Tags className="w-4 h-4" />,
    label: t("settings.categories.condition_types", { defaultValue: "Điều kiện làm việc" }),
    path: ROUTES.SETTINGS.CONDITION_TYPES,
    permissions: ["workspace:settings:read"],
  },
  {
    description: t("shifts.rules.subtitle", { defaultValue: "Mẫu lịch xoay ca và quy tắc gán ca tự động" }),
    icon: <Zap className="w-4 h-4" />,
    label: t("common.sections.shifts_rules", { defaultValue: "Mẫu ca & Quy tắc phân ca" }),
    path: ROUTES.SHIFTS_RULES,
    permissions: ["shifts:read"],
  },
];

const getSystemGroup = (t: TFunction): SettingItem[] => [
  {
    description: t("settings.categories.storage_desc", { defaultValue: "Cấu hình lưu trữ MinIO S3 và tài liệu" }),
    icon: <Database className="w-4 h-4" />,
    label: t("settings.categories.storage", { defaultValue: "Lưu trữ tài liệu" }),
    path: ROUTES.SETTINGS.STORAGE,
    permissions: ["workspace:settings:read"],
  },
  {
    description: t("smtp.subtitle", { defaultValue: "Cấu hình kết nối máy chủ gửi email thông báo" }),
    icon: <Mail className="w-4 h-4" />,
    label: t("smtp.title", { defaultValue: "Máy chủ Email (SMTP)" }),
    path: ROUTES.SYSTEM_SMTP,
    permissions: ["security:settings:manage"],
  },
  {
    description: t("settings.categories.econtracts_desc", { defaultValue: "Cấu hình tích hợp và chữ ký số hợp đồng" }),
    featureFlag: "ff_econtracts",
    icon: <Scale className="w-4 h-4" />,
    label: t("settings.categories.econtracts", { defaultValue: "Hợp đồng điện tử" }),
    path: ROUTES.SETTINGS.ECONTRACTS,
    permissions: ["workspace:settings:read"],
  },
];

export interface SettingsNavState {
  accessGroup: SettingItem[];
  operationsGroup: SettingItem[];
  systemGroup: SettingItem[];
  workspaceGroup: SettingItem[];
}

export function useSettingsNav(): SettingsNavState {
  const { t } = useTranslation();
  const { hasAnyPermission } = useMyPermissions();

  const workspaceGroup = useMemo(() => getWorkspaceGroup(t), [t]);
  const accessGroup = useMemo(() => getAccessGroup(t), [t]);
  const operationsGroup = useMemo(() => getOperationsGroup(t), [t]);
  const systemGroup = useMemo(() => getSystemGroup(t), [t]);
  const filterItems = (items: SettingItem[]) =>
    items.filter(
      (item) =>
        (!item.permissions || item.permissions.length === 0 || hasAnyPermission(...item.permissions)) &&
        (!item.featureFlag || isFeatureEnabled(item.featureFlag))
    );
  return {
    accessGroup: filterItems(accessGroup),
    operationsGroup: filterItems(operationsGroup),
    systemGroup: filterItems(systemGroup),
    workspaceGroup: filterItems(workspaceGroup),
  };
}
