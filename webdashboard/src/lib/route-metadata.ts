import { ROUTES } from "./routes";

export interface RouteMetadata {
  /** Translation key for breadcrumb item, or fallback string */
  breadcrumbKey: string;
  defaultBreadcrumb: string;
  /** Translation key for page title, or fallback string */
  titleKey: string;
  defaultTitle: string;
}

/**
 * Centralized Route Metadata Dictionary.
 * Maps exact route paths to their localized title & breadcrumb definitions.
 * Follows DRY: Single source of truth for Topbar, Navbars, and Document titles.
 */
export const ROUTE_METADATA_MAP: Record<string, RouteMetadata> = {
  // Main
  [ROUTES.HOME]: {
    breadcrumbKey: "common.dashboard",
    defaultBreadcrumb: "Bảng điều khiển",
    titleKey: "common.dashboard",
    defaultTitle: "Bảng điều khiển",
  },
  [ROUTES.DASHBOARD]: {
    breadcrumbKey: "dashboard.catalog.title",
    defaultBreadcrumb: "Tổng quan",
    titleKey: "dashboard.catalog.title",
    defaultTitle: "Tổng quan",
  },
  [ROUTES.EMPLOYEES]: {
    breadcrumbKey: "common.employees",
    defaultBreadcrumb: "Nhân viên",
    titleKey: "common.employees",
    defaultTitle: "Nhân viên",
  },
  [ROUTES.DEPARTMENTS]: {
    breadcrumbKey: "common.departments",
    defaultBreadcrumb: "Phòng ban",
    titleKey: "common.departments",
    defaultTitle: "Phòng ban",
  },
  [ROUTES.ORG_CHART]: {
    breadcrumbKey: "common.org_chart",
    defaultBreadcrumb: "Sơ đồ tổ chức",
    titleKey: "common.org_chart",
    defaultTitle: "Sơ đồ tổ chức",
  },
  [ROUTES.NOTIFICATIONS]: {
    breadcrumbKey: "common.notifications",
    defaultBreadcrumb: "Thông báo",
    titleKey: "common.notifications",
    defaultTitle: "Thông báo",
  },

  // Attendance & Leave
  [ROUTES.ATTENDANCE_LOGS]: {
    breadcrumbKey: "common.attendance_logs",
    defaultBreadcrumb: "Nhật ký quẹt thẻ",
    titleKey: "common.attendance_logs",
    defaultTitle: "Nhật ký quẹt thẻ",
  },
  [ROUTES.LEAVE_REQUESTS]: {
    breadcrumbKey: "leave_requests.title",
    defaultBreadcrumb: "Yêu cầu nghỉ phép",
    titleKey: "leave_requests.title",
    defaultTitle: "Yêu cầu nghỉ phép",
  },
  [ROUTES.LEAVE_BALANCES]: {
    breadcrumbKey: "common.leave_balances",
    defaultBreadcrumb: "Số dư nghỉ phép",
    titleKey: "common.leave_balances",
    defaultTitle: "Số dư nghỉ phép",
  },
  [ROUTES.ATTENDANCE_ADJUSTMENTS]: {
    breadcrumbKey: "common.attendance_adjustments",
    defaultBreadcrumb: "Xác nhận công",
    titleKey: "common.attendance_adjustments",
    defaultTitle: "Xác nhận công",
  },
  [ROUTES.APPROVALS]: {
    breadcrumbKey: "approvals.inbox.title",
    defaultBreadcrumb: "Hộp thư Phê duyệt",
    titleKey: "approvals.inbox.title",
    defaultTitle: "Hộp thư Phê duyệt",
  },
  [ROUTES.APPROVALS_OT]: {
    breadcrumbKey: "ot_approvals.ot_title",
    defaultBreadcrumb: "Phê duyệt Tăng ca",
    titleKey: "ot_approvals.ot_title",
    defaultTitle: "Phê duyệt Tăng ca",
  },
  [ROUTES.HOLIDAYS]: {
    breadcrumbKey: "settings.categories.holidays",
    defaultBreadcrumb: "Ngày nghỉ lễ",
    titleKey: "settings.categories.holidays",
    defaultTitle: "Ngày nghỉ lễ",
  },

  // Operations
  [ROUTES.REPORTS]: {
    breadcrumbKey: "common.reports",
    defaultBreadcrumb: "Báo cáo",
    titleKey: "common.reports",
    defaultTitle: "Báo cáo",
  },
  [ROUTES.SHIFTS]: {
    breadcrumbKey: "common.shifts_and_roster",
    defaultBreadcrumb: "Ca làm & Lịch trực",
    titleKey: "common.shifts_and_roster",
    defaultTitle: "Ca làm & Lịch trực",
  },
  [ROUTES.SHIFTS_RULES]: {
    breadcrumbKey: "common.sections.shifts_rules",
    defaultBreadcrumb: "Cấu hình tự động",
    titleKey: "common.sections.shifts_rules",
    defaultTitle: "Cấu hình tự động",
  },
  [ROUTES.SHIFT_SWAPS]: {
    breadcrumbKey: "shift_swaps.title",
    defaultBreadcrumb: "Đổi ca",
    titleKey: "shift_swaps.title",
    defaultTitle: "Đổi ca",
  },
  [ROUTES.PAYROLL]: {
    breadcrumbKey: "payroll.title",
    defaultBreadcrumb: "Tính lương",
    titleKey: "payroll.title",
    defaultTitle: "Tính lương",
  },
  [ROUTES.DOCUMENTS]: {
    breadcrumbKey: "common.my_documents",
    defaultBreadcrumb: "Tài liệu & Biểu mẫu",
    titleKey: "common.my_documents",
    defaultTitle: "Tài liệu & Biểu mẫu",
  },
  [ROUTES.PROFILE]: {
    breadcrumbKey: "common.profile",
    defaultBreadcrumb: "Hồ sơ",
    titleKey: "common.profile",
    defaultTitle: "Hồ sơ",
  },

  // Infra & System
  [ROUTES.INFRA_MONITOR]: {
    breadcrumbKey: "infra.monitor_title",
    defaultBreadcrumb: "Giám sát hệ thống",
    titleKey: "infra.monitor_title",
    defaultTitle: "Giám sát Worker & Hạ tầng",
  },
  [ROUTES.SYSTEM_SMTP]: {
    breadcrumbKey: "system.smtp_title",
    defaultBreadcrumb: "Cài đặt Email SMTP",
    titleKey: "system.smtp_title",
    defaultTitle: "Cài đặt Email SMTP",
  },
  [ROUTES.AUDIT_LOGS]: {
    breadcrumbKey: "audit_logs.title",
    defaultBreadcrumb: "Nhật ký hệ thống",
    titleKey: "audit_logs.title",
    defaultTitle: "Nhật ký hệ thống",
  },

  // Employee Self-Service (ESS)
  [ROUTES.ESS_PROFILE]: {
    breadcrumbKey: "common.my_profile",
    defaultBreadcrumb: "Hồ sơ của tôi",
    titleKey: "common.my_profile",
    defaultTitle: "Hồ sơ của tôi",
  },
  [ROUTES.ESS_ATTENDANCE]: {
    breadcrumbKey: "common.my_attendance",
    defaultBreadcrumb: "Bảng công của tôi",
    titleKey: "common.my_attendance",
    defaultTitle: "Bảng công của tôi",
  },
  [ROUTES.ESS_CLOCK]: {
    breadcrumbKey: "common.clock_in_out",
    defaultBreadcrumb: "Máy chấm công",
    titleKey: "common.clock_in_out",
    defaultTitle: "Máy chấm công",
  },
  [ROUTES.ESS_OT]: {
    breadcrumbKey: "common.my_overtime",
    defaultBreadcrumb: "Tăng ca (OT)",
    titleKey: "common.my_overtime",
    defaultTitle: "Tăng ca (OT)",
  },
  [ROUTES.ESS_DOCUMENTS]: {
    breadcrumbKey: "common.my_documents",
    defaultBreadcrumb: "Tài liệu của tôi",
    titleKey: "common.my_documents",
    defaultTitle: "Tài liệu của tôi",
  },

  // Workspace Settings
  [ROUTES.SETTINGS.ROOT]: {
    breadcrumbKey: "settings.title",
    defaultBreadcrumb: "Trung tâm Cài đặt",
    titleKey: "settings.title",
    defaultTitle: "Trung tâm Cài đặt",
  },
  [ROUTES.SETTINGS.GENERAL]: {
    breadcrumbKey: "settings.categories.general",
    defaultBreadcrumb: "Cài đặt chung",
    titleKey: "settings.categories.general",
    defaultTitle: "Cài đặt chung",
  },
  [ROUTES.SETTINGS.ORGANIZATION]: {
    breadcrumbKey: "settings.categories.organization",
    defaultBreadcrumb: "Cơ cấu tổ chức",
    titleKey: "settings.categories.organization",
    defaultTitle: "Cơ cấu tổ chức",
  },
  [ROUTES.SETTINGS.TIME]: {
    breadcrumbKey: "settings.categories.time",
    defaultBreadcrumb: "Thời gian & Chấm công",
    titleKey: "settings.categories.time",
    defaultTitle: "Thời gian & Chấm công",
  },
  [ROUTES.SETTINGS.PAYROLL]: {
    breadcrumbKey: "settings.categories.payroll",
    defaultBreadcrumb: "Bảng lương & Công thức",
    titleKey: "settings.categories.payroll",
    defaultTitle: "Bảng lương & Công thức",
  },
  [ROUTES.SETTINGS.LEAVE]: {
    breadcrumbKey: "settings.categories.leave",
    defaultBreadcrumb: "Chính sách nghỉ phép",
    titleKey: "settings.categories.leave",
    defaultTitle: "Chính sách nghỉ phép",
  },
  [ROUTES.SETTINGS.OVERTIME]: {
    breadcrumbKey: "settings.categories.overtime",
    defaultBreadcrumb: "Quy định tăng ca",
    titleKey: "settings.categories.overtime",
    defaultTitle: "Quy định tăng ca",
  },
  [ROUTES.SETTINGS.LOCATIONS]: {
    breadcrumbKey: "settings.categories.locations",
    defaultBreadcrumb: "Địa điểm làm việc",
    titleKey: "settings.categories.locations",
    defaultTitle: "Địa điểm làm việc",
  },
  [ROUTES.SETTINGS.WORKFLOWS]: {
    breadcrumbKey: "settings.categories.workflows",
    defaultBreadcrumb: "Quy trình phê duyệt",
    titleKey: "settings.categories.workflows",
    defaultTitle: "Quy trình phê duyệt",
  },
  [ROUTES.SETTINGS.CUSTOM_FIELDS]: {
    breadcrumbKey: "settings.categories.custom_fields",
    defaultBreadcrumb: "Trường tùy chỉnh",
    titleKey: "settings.categories.custom_fields",
    defaultTitle: "Trường tùy chỉnh",
  },
  [ROUTES.SETTINGS.LIFECYCLE_TEMPLATES]: {
    breadcrumbKey: "settings.categories.lifecycle",
    defaultBreadcrumb: "Quy trình Onboarding",
    titleKey: "settings.categories.lifecycle",
    defaultTitle: "Quy trình Onboarding",
  },
  [ROUTES.SETTINGS.CONDITION_TYPES]: {
    breadcrumbKey: "settings.categories.condition_types",
    defaultBreadcrumb: "Điều kiện làm việc",
    titleKey: "settings.categories.condition_types",
    defaultTitle: "Điều kiện làm việc",
  },
  [ROUTES.SETTINGS.STORAGE]: {
    breadcrumbKey: "settings.categories.storage",
    defaultBreadcrumb: "Lưu trữ tài liệu",
    titleKey: "settings.categories.storage",
    defaultTitle: "Lưu trữ tài liệu",
  },
  [ROUTES.SETTINGS.ECONTRACTS]: {
    breadcrumbKey: "settings.categories.econtracts",
    defaultBreadcrumb: "Hợp đồng điện tử",
    titleKey: "settings.categories.econtracts",
    defaultTitle: "Hợp đồng điện tử",
  },
  [ROUTES.SETTINGS.SMTP]: {
    breadcrumbKey: "smtp.title",
    defaultBreadcrumb: "Máy chủ Email (SMTP)",
    titleKey: "smtp.title",
    defaultTitle: "Máy chủ Email (SMTP)",
  },
  [ROUTES.SETTINGS.ROLES]: {
    breadcrumbKey: "settings.categories.roles",
    defaultBreadcrumb: "Vai trò & Chức danh",
    titleKey: "settings.categories.roles",
    defaultTitle: "Vai trò & Chức danh",
  },
  [ROUTES.SETTINGS.PERMISSIONS]: {
    breadcrumbKey: "settings.categories.permissions",
    defaultBreadcrumb: "Phân quyền người dùng",
    titleKey: "settings.categories.permissions",
    defaultTitle: "Phân quyền người dùng",
  },
  [ROUTES.SETTINGS.MEMBERS]: {
    breadcrumbKey: "members.title",
    defaultBreadcrumb: "Tài khoản người dùng",
    titleKey: "members.title",
    defaultTitle: "Tài khoản người dùng",
  },
};

/**
 * Resolver function to look up metadata by current pathname.
 * Handles exact matches and nested prefixes.
 */
export function getRouteMetadata(pathname: string): RouteMetadata {
  if (ROUTE_METADATA_MAP[pathname]) {
    return ROUTE_METADATA_MAP[pathname];
  }

  // Check prefix match
  for (const [routePath, meta] of Object.entries(ROUTE_METADATA_MAP)) {
    if (routePath !== "/" && pathname.startsWith(`${routePath}/`)) {
      return meta;
    }
  }

  // Dynamic fallback: clean segment name
  const segments = pathname.split("/").filter(Boolean);
  const last = segments[segments.length - 1];
  const formatted = last
    ? last.replace(/-/g, " ").replace(/\b\w/g, (c) => c.toUpperCase())
    : "Bảng điều khiển";

  return {
    breadcrumbKey: "",
    defaultBreadcrumb: formatted,
    titleKey: "",
    defaultTitle: formatted,
  };
}
