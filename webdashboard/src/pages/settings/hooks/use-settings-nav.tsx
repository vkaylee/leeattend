import {
  Building2,
  ShieldAlert,
  KeyRound,
  CreditCard,
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
];

const getOperationsGroup = (t: TFunction): SettingItem[] => [
  {
    description: t("settings.categories.organization_desc", { defaultValue: "Quản lý cơ cấu phòng ban và giai đoạn công tác" }),
    icon: <Building2 className="w-4 h-4" />,
    label: t("settings.categories.organization", { defaultValue: "Cơ cấu tổ chức" }),
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
    description: t("settings.categories.holidays_desc", { defaultValue: "Lịch nghỉ lễ và quy tắc chấm công ngày lễ" }),
    icon: <Palmtree className="w-4 h-4" />,
    label: t("settings.categories.holidays", { defaultValue: "Ngày nghỉ lễ" }),
    path: ROUTES.SETTINGS.HOLIDAYS,
    permissions: ["holidays:manage"],
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
    description: t("settings.categories.econtracts_desc", { defaultValue: "Cấu hình tích hợp và chữ ký số hợp đồng" }),
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
    items.filter(item => !item.permissions || item.permissions.length === 0 || hasAnyPermission(...item.permissions));

  return {
    accessGroup: filterItems(accessGroup),
    operationsGroup: filterItems(operationsGroup),
    systemGroup: filterItems(systemGroup),
    workspaceGroup: filterItems(workspaceGroup),
  };
}
