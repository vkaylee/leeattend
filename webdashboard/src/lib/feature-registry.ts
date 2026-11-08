/* eslint-disable max-lines -- comprehensive DSM 7 feature catalog (same as app-registry.ts) */

import type React from "react";
import {
  ArrowLeftRight,
  Calendar,
  CalendarCheck,
  CalendarPlus,
  CheckCircle,
  ClipboardList,
  Clock,
  DollarSign,
  FileSpreadsheet,
  FileText,
  FolderTree,
  History,
  PlusCircle,
  Scale,
  UserPlus,
  Users,
  Wifi,
  Zap,
} from "lucide-react";
import type { NavGroup } from "@/hooks/use-nav-menu-items";
import type { FeatureFlagName } from "@/lib/feature-flags";

import { APP_REGISTRY, type AppDefinition } from "@/lib/app-registry";
import { isFeatureEnabled } from "@/lib/feature-flags";
import { ROUTES } from "@/lib/routes";
import { removeDiacritics } from "@/lib/utils";

export type FeatureCategory = "app" | "action";

export interface FeatureItem {
  category: FeatureCategory;
  descriptionFallback: string;
  descriptionKey?: string;
  featureFlag?: FeatureFlagName;
  gradient: string;
  group?: NavGroup;
  icon: React.ElementType;
  id: string;
  keywords?: string[];
  permissions?: string[];
  testId: string;
  titleFallback: string;
  titleKey?: string;
  to: string;
}

// ── DSM 7 App Gradients ──────────────────────────────────────────────

export const APP_GRADIENTS: Record<string, string> = {
  dashboard: "from-indigo-500 to-purple-600 shadow-indigo-500/25",
  "dashboard-analytics": "from-pink-500 to-rose-600 shadow-pink-500/25",
  employees: "from-blue-500 to-cyan-600 shadow-blue-500/25",
  people: "from-sky-500 to-blue-600 shadow-sky-500/25",
  departments: "from-teal-500 to-emerald-600 shadow-teal-500/25",
  "org-chart": "from-emerald-500 to-green-600 shadow-emerald-500/25",
  econtracts: "from-amber-600 to-stone-600 shadow-amber-600/25",
  "attendance-logs": "from-amber-500 to-orange-600 shadow-amber-500/25",
  "leave-requests": "from-sky-400 to-indigo-600 shadow-sky-500/25",
  "leave-balances": "from-orange-500 to-amber-600 shadow-orange-500/25",
  "attendance-adjustments": "from-amber-400 to-yellow-600 shadow-amber-500/25",
  "shift-swaps": "from-violet-500 to-fuchsia-600 shadow-violet-500/25",
  reports: "from-blue-600 to-indigo-700 shadow-blue-600/25",
  approvals: "from-emerald-500 to-teal-700 shadow-emerald-500/25",
  "approvals-ot": "from-emerald-500 to-teal-600 shadow-emerald-500/25",
  shifts: "from-purple-500 to-indigo-600 shadow-purple-500/25",
  "shifts-rules": "from-fuchsia-500 to-pink-600 shadow-fuchsia-500/25",
  payroll: "from-emerald-600 to-teal-700 shadow-emerald-600/25",
  documents: "from-blue-500 to-slate-600 shadow-blue-500/25",
  "infra-monitor": "from-cyan-500 to-blue-700 shadow-cyan-500/25",
  settings: "from-slate-600 to-slate-800 shadow-slate-600/25",
  "ess-profile": "from-violet-500 to-purple-700 shadow-violet-500/25",
  "ess-attendance": "from-orange-500 to-rose-600 shadow-orange-500/25",
  "ess-clock": "from-cyan-500 to-emerald-600 shadow-cyan-500/25",
};

// ── App to FeatureItem Conversion ───────────────────────────────────

export function appToFeature(app: AppDefinition): FeatureItem {
  return {
    id: `app-${app.id}`,
    category: "app",
    titleKey: app.labelKey,
    titleFallback: app.id,
    descriptionKey: app.descriptionKey,
    descriptionFallback: "",
    to: app.to,
    icon: app.icon,
    gradient: APP_GRADIENTS[app.id] ?? "from-primary-500 to-primary-700 shadow-primary-500/25",
    group: app.group,
    permissions: app.permissions,
    featureFlag: app.featureFlag,
    keywords: [app.id, app.group, "app", "ung dung"],
    testId: `dsm-app-${app.id}`,
  };
}

// ── Quick Actions & Sub-Features ────────────────────────────────────

export const ACTION_FEATURES: FeatureItem[] = [
  {
    id: "act-clock-in",
    category: "action",
    titleKey: "ess.clock.title",
    titleFallback: "Bấm giờ điểm danh (Clock In/Out)",
    descriptionKey: "ess.clock.subtitle",
    descriptionFallback: "Chấm công trực tuyến qua Wi-Fi / GPS",
    to: ROUTES.ESS_CLOCK,
    icon: Clock,
    gradient: "from-cyan-500 to-emerald-600 shadow-cyan-500/25",
    permissions: ["ess:clock"],
    keywords: ["clock in", "check in", "diem danh", "bam gio", "cham cong", "punch in", "online clock"],
    testId: "dsm-action-clock-in",
  },
  {
    id: "act-claim-ot",
    category: "action",
    titleFallback: "Đăng ký làm thêm giờ (Claim OT)",
    descriptionFallback: "Gửi yêu cầu xác nhận làm ngoài giờ",
    to: ROUTES.ESS_OT,
    icon: PlusCircle,
    gradient: "from-amber-500 to-orange-600 shadow-amber-500/25",
    permissions: ["ess:attendance"],
    keywords: ["ot", "tang ca", "lam them", "overtime", "dang ky ot", "claim ot", "ngoai gio"],
    testId: "dsm-action-claim-ot",
  },
  {
    id: "act-create-leave",
    category: "action",
    titleFallback: "Tạo đơn xin nghỉ phép",
    descriptionFallback: "Nộp đơn nghỉ phép năm, nghỉ ốm, việc riêng",
    to: ROUTES.LEAVE_REQUESTS,
    icon: CalendarPlus,
    gradient: "from-sky-500 to-blue-600 shadow-sky-500/25",
    permissions: ["leave:requests:read"],
    keywords: ["xin nghi", "nghi phep", "don phep", "annual leave", "tao don", "phep nam", "nghi om"],
    testId: "dsm-action-create-leave",
  },
  {
    id: "act-swap-shift",
    category: "action",
    titleFallback: "Yêu cầu đổi ca làm việc",
    descriptionFallback: "Đổi ca trực tiếp với đồng nghiệp",
    to: ROUTES.SHIFT_SWAPS,
    icon: ArrowLeftRight,
    gradient: "from-violet-500 to-fuchsia-600 shadow-violet-500/25",
    permissions: ["shift_swaps:manage"],
    keywords: ["doi ca", "swap ca", "chuyen ca", "shift swap", "doi lich"],
    testId: "dsm-action-swap-shift",
  },
  {
    id: "act-new-employee",
    category: "action",
    titleFallback: "Thêm nhân viên mới",
    descriptionFallback: "Tạo hồ sơ và tài khoản nhân sự mới",
    to: ROUTES.EMPLOYEES,
    icon: UserPlus,
    gradient: "from-blue-500 to-cyan-600 shadow-blue-500/25",
    permissions: ["employees:read"],
    keywords: ["them nhan vien", "tao ho so", "new employee", "them moi", "onboard"],
    testId: "dsm-action-new-employee",
  },
  {
    id: "act-new-shift",
    category: "action",
    titleFallback: "Thiết lập ca làm mới",
    descriptionFallback: "Tạo khung giờ và quy tắc phân ca",
    to: ROUTES.SHIFTS_NEW,
    icon: Calendar,
    gradient: "from-purple-500 to-indigo-600 shadow-purple-500/25",
    permissions: ["shifts:read"],
    keywords: ["tao ca", "ca moi", "them ca", "new shift", "khung gio"],
    testId: "dsm-action-new-shift",
  },
  {
    id: "act-attendance-adjustment",
    category: "action",
    titleFallback: "Giải trình & Điều chỉnh công",
    descriptionFallback: "Bổ sung lượt chấm công bị quên hoặc sai lệch",
    to: ROUTES.ATTENDANCE_ADJUSTMENTS,
    icon: CheckCircle,
    gradient: "from-amber-400 to-yellow-600 shadow-amber-500/25",
    permissions: ["attendance:adjustments:approve", "attendance:adjustments:submit_on_behalf"],
    keywords: ["sua cong", "giai trinh", "quen cham cong", "missing punch", "dieu chinh"],
    testId: "dsm-action-attendance-adjustment",
  },
  {
    id: "act-export-reports",
    category: "action",
    titleFallback: "Xuất báo cáo & Dữ liệu chuyên cần",
    descriptionFallback: "Tải file Excel tổng hợp công và đi muộn",
    to: ROUTES.REPORTS,
    icon: FileSpreadsheet,
    gradient: "from-blue-600 to-indigo-700 shadow-blue-600/25",
    permissions: ["reports:read"],
    keywords: ["xuat excel", "xuat bao cao", "export report", "tai bao cao", "chuyen can"],
    testId: "dsm-action-export-reports",
  },
  {
    id: "act-payroll-process",
    category: "action",
    titleFallback: "Bảng tính & Chốt lương tháng",
    descriptionFallback: "Xem chi tiết bảng lương và xuất phiếu lương",
    to: ROUTES.PAYROLL,
    icon: DollarSign,
    gradient: "from-emerald-600 to-teal-700 shadow-emerald-600/25",
    permissions: ["payroll:read"],
    keywords: ["tinh luong", "chot luong", "bang luong", "payroll", "phieu luong", "payslip"],
    testId: "dsm-action-payroll-process",
  },
  {
    id: "act-org-chart",
    category: "action",
    titleFallback: "Sơ đồ tổ chức công ty",
    descriptionFallback: "Trực quan hóa cấu trúc phân cấp các phòng ban",
    to: ROUTES.ORG_CHART,
    icon: FolderTree,
    gradient: "from-teal-500 to-emerald-700 shadow-teal-500/25",
    permissions: ["departments:read"],
    keywords: ["so do to chuc", "org chart", "cay phong ban", "co cau to chuc", "hierarchy"],
    testId: "dsm-action-org-chart",
  },
  {
    id: "act-members-directory",
    category: "action",
    titleFallback: "Danh bạ & Thành viên",
    descriptionFallback: "Tra cứu thông tin liên lạc và chức danh nhân sự",
    to: ROUTES.SETTINGS.MEMBERS,
    icon: Users,
    gradient: "from-indigo-500 to-blue-700 shadow-indigo-500/25",
    permissions: ["members:read"],
    keywords: ["danh ba", "thanh vien", "directory", "people", "dong nghiep", "tra cuu"],
    testId: "dsm-action-members-directory",
  },
  {
    id: "act-shift-rules",
    category: "action",
    titleFallback: "Quy tắc tự động hóa ca làm",
    descriptionFallback: "Cấu hình phân ca tự động và mẫu ca làm việc",
    to: ROUTES.SHIFTS_RULES,
    icon: Zap,
    gradient: "from-amber-500 to-yellow-600 shadow-amber-500/25",
    permissions: ["shifts:read"],
    keywords: ["tu dong hoa ca", "shift rules", "quy tac ca", "working patterns", "mau ca"],
    testId: "dsm-action-shift-rules",
  },
  {
    id: "act-leave-balances",
    category: "action",
    titleFallback: "Bảng theo dõi quỹ phép",
    descriptionFallback: "Tổng hợp số ngày phép đã dùng và còn lại của nhân sự",
    to: ROUTES.LEAVE_BALANCES,
    icon: Scale,
    gradient: "from-cyan-500 to-teal-700 shadow-cyan-500/25",
    permissions: ["leave:balances:read"],
    keywords: ["quy phep", "han muc phep", "phep ton", "leave balance", "so du phep"],
    testId: "dsm-action-leave-balances",
  },
  {
    id: "act-attendance-logs",
    category: "action",
    titleFallback: "Nhật ký chấm công toàn công ty",
    descriptionFallback: "Chi tiết giờ vào ra, đi muộn về sớm của nhân viên",
    to: ROUTES.ATTENDANCE_LOGS,
    icon: CalendarCheck,
    gradient: "from-emerald-500 to-green-700 shadow-emerald-500/25",
    permissions: ["attendance:logs:read"],
    keywords: ["bang cham cong", "lich su cham cong", "attendance logs", "di muon", "ve som"],
    testId: "dsm-action-attendance-logs",
  },
  {
    id: "act-my-attendance-history",
    category: "action",
    titleFallback: "Lịch sử chấm công của tôi",
    descriptionFallback: "Xem chi tiết dữ liệu bấm giờ cá nhân",
    to: ROUTES.ESS_ATTENDANCE,
    icon: History,
    gradient: "from-blue-500 to-indigo-600 shadow-blue-500/25",
    permissions: ["ess:attendance"],
    keywords: ["lich su cham cong cua toi", "my attendance", "gio vao ra cua toi", "cong cua toi"],
    testId: "dsm-action-my-attendance-history",
  },
  {
    id: "act-my-documents",
    category: "action",
    titleFallback: "Tài liệu & Hồ sơ của tôi",
    descriptionFallback: "Hợp đồng, giấy tờ cá nhân và chứng chỉ lưu trữ",
    to: ROUTES.ESS_DOCUMENTS,
    icon: FileText,
    gradient: "from-slate-600 to-slate-800 shadow-slate-600/25",
    permissions: ["ess:documents"],
    keywords: ["tai lieu cua toi", "my documents", "ho so ca nhan", "hop dong cua toi", "giay to"],
    testId: "dsm-action-my-documents",
  },
  {
    id: "act-setting-time-wifi",
    category: "action",
    titleKey: "settings.categories.time",
    titleFallback: "Cài đặt Wi-Fi & Chấm công",
    descriptionFallback: "Cấu hình IP mạng và danh sách địa chỉ Wi-Fi BSSID hợp lệ",
    to: ROUTES.SETTINGS.TIME,
    icon: Wifi,
    gradient: "from-blue-600 to-cyan-600 shadow-blue-500/25",
    permissions: ["settings:time", "settings:read"],
    keywords: ["wifi", "bssid", "ip", "cham cong wifi", "diem danh wifi", "cai dat wifi", "time settings"],
    testId: "dsm-action-setting-time-wifi",
  },
];


// ── Master Feature Registry ─────────────────────────────────────────

export function getAllFeatures(): FeatureItem[] {
  const appFeatures = APP_REGISTRY.map(appToFeature);
  return [...appFeatures, ...ACTION_FEATURES];
}

// ── Permission Filtering ────────────────────────────────────────────

export function filterFeatures(
  features: FeatureItem[],
  hasAnyPermission: (...perms: string[]) => boolean,
): FeatureItem[] {
  return features.filter((item) => {
    if (item.featureFlag && !isFeatureEnabled(item.featureFlag)) {
      return false;
    }
    if (item.permissions && item.permissions.length > 0) {
      return hasAnyPermission(...item.permissions);
    }
    return true;
  });
}

// ── Search Index Builder ────────────────────────────────────────────

/**
 * Builds a search index string containing with-diacritics, without-diacritics,
 * keywords, and route paths for instant cmdk filtering.
 */
export function buildFeatureSearchIndex(
  item: FeatureItem,
  t?: (key: string, options?: { defaultValue?: string }) => string,
): string {
  const rawTitle = (item.titleKey && t ? t(item.titleKey, { defaultValue: item.titleFallback }) : item.titleFallback) || "";
  const rawDesc = (item.descriptionKey && t ? t(item.descriptionKey, { defaultValue: item.descriptionFallback }) : item.descriptionFallback) || "";

  const titleNoDiacritics = removeDiacritics(rawTitle);
  const descNoDiacritics = removeDiacritics(rawDesc);
  const kw = (item.keywords ?? []).join(" ");
  const kwNoDiacritics = removeDiacritics(kw);

  return `${rawTitle} ${titleNoDiacritics} ${rawDesc} ${descNoDiacritics} ${kw} ${kwNoDiacritics} ${item.to} ${item.category}`.toLowerCase();
}
