/* eslint-disable max-lines -- comprehensive DSM 7 feature catalog (same as app-registry.ts) */

import type React from "react";
import {
  Activity,
  ArrowLeftRight,
  Banknote,
  Braces,
  Building2,
  Calendar,
  CalendarCheck,
  CalendarPlus,
  CheckCircle,
  ClipboardList,
  Clock,
  CreditCard,
  Database,
  DollarSign,
  FileSpreadsheet,
  FileText,
  FolderTree,
  GitMerge,
  History,
  KeyRound,
  ListTodo,
  Lock,
  Mail,
  MapPin,
  PlusCircle,
  Scale,
  Settings,
  ShieldCheck,
  Sliders,
  Tags,
  UserPlus,
  Users,
  Workflow,
  Zap,
} from "lucide-react";

import type { NavGroup } from "@/hooks/use-nav-menu-items";
import type { FeatureFlagName } from "@/lib/feature-flags";

import { APP_REGISTRY, type AppDefinition } from "@/lib/app-registry";
import { isFeatureEnabled } from "@/lib/feature-flags";
import { ROUTES } from "@/lib/routes";
import { removeDiacritics } from "@/lib/utils";

export type FeatureCategory = "app" | "action" | "setting";

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
  systemRoles?: string[];
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
  "approvals-ot": "from-emerald-500 to-teal-600 shadow-emerald-500/25",
  shifts: "from-purple-500 to-indigo-600 shadow-purple-500/25",
  "shifts-rules": "from-fuchsia-500 to-pink-600 shadow-fuchsia-500/25",
  payroll: "from-emerald-600 to-teal-700 shadow-emerald-600/25",
  documents: "from-blue-500 to-slate-600 shadow-blue-500/25",
  "infra-monitor": "from-cyan-500 to-blue-700 shadow-cyan-500/25",
  "system-roles": "from-rose-600 to-red-700 shadow-rose-600/25",
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
    systemRoles: app.systemRoles,
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
    to: ROUTES.PEOPLE,
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
];

// ── Settings & System Configuration ─────────────────────────────────

export const SETTING_FEATURES: FeatureItem[] = [
  {
    id: "set-general",
    category: "setting",
    titleFallback: "Cài đặt chung Workspace",
    descriptionFallback: "Tên tổ chức, logo, múi giờ và định dạng hiển thị",
    to: ROUTES.SETTINGS.GENERAL,
    icon: Settings,
    gradient: "from-slate-600 to-slate-800 shadow-slate-600/25",
    permissions: ["workspace:settings:read"],
    keywords: ["ten cong ty", "logo", "general settings", "cai dat chung", "mui gio"],
    testId: "dsm-setting-general",
  },
  {
    id: "set-time",
    category: "setting",
    titleFallback: "Cấu hình chấm công & Wi-Fi / GPS",
    descriptionFallback: "Khóa mạng Wi-Fi, tọa độ văn phòng, ngưỡng đi muộn",
    to: ROUTES.SETTINGS.TIME,
    icon: Sliders,
    gradient: "from-cyan-600 to-blue-700 shadow-cyan-600/25",
    permissions: ["workspace:settings:read", "attendance:logs:read"],
    keywords: ["wifi", "gps", "dia chi ip", "ip lock", "gio lam viec", "cham cong wifi", "toa do"],
    testId: "dsm-setting-time",
  },
  {
    id: "set-overtime",
    category: "setting",
    titleFallback: "Quy định làm thêm giờ (Overtime Policy)",
    descriptionFallback: "Hệ số lương OT ngày thường, ngày nghỉ, ngày lễ",
    to: ROUTES.SETTINGS.OVERTIME,
    icon: Clock,
    gradient: "from-amber-600 to-orange-700 shadow-amber-600/25",
    permissions: ["workspace:settings:read", "approval_workflows:read"],
    keywords: ["quy dinh ot", "he so luong", "overtime policy", "chinh sach tang ca", "he so ot"],
    testId: "dsm-setting-overtime",
  },
  {
    id: "set-leave",
    category: "setting",
    titleFallback: "Chính sách & Chế độ nghỉ phép",
    descriptionFallback: "Số ngày phép chuẩn, thâm niên, trừ phép lũy kế",
    to: ROUTES.SETTINGS.LEAVE,
    icon: CalendarCheck,
    gradient: "from-sky-600 to-indigo-700 shadow-sky-600/25",
    permissions: ["workspace:settings:read", "leave:policies:manage"],
    keywords: ["chinh sach nghi phep", "phep nam", "che do nghi", "leave policy", "ngay phep"],
    testId: "dsm-setting-leave",
  },
  {
    id: "set-holidays",
    category: "setting",
    titleFallback: "Danh mục ngày nghỉ lễ & Tết",
    descriptionFallback: "Lịch nghỉ lễ quốc gia và quy định hưởng lương",
    to: ROUTES.SETTINGS.HOLIDAYS,
    icon: Calendar,
    gradient: "from-rose-500 to-red-600 shadow-rose-500/25",
    permissions: ["workspace:settings:read", "holidays:manage"],
    keywords: ["nghi le", "ngay tet", "lich nghi le", "public holidays", "nghi le huong luong"],
    testId: "dsm-setting-holidays",
  },
  {
    id: "set-workflows",
    category: "setting",
    titleFallback: "Quy trình xét duyệt đa cấp (Approval Workflows)",
    descriptionFallback: "Phân cấp phê duyệt đơn phép, OT và điều chỉnh công",
    to: ROUTES.SETTINGS.WORKFLOWS,
    icon: GitMerge,
    gradient: "from-violet-600 to-purple-700 shadow-violet-600/25",
    permissions: ["workspace:settings:read", "approval_workflows:read"],
    keywords: ["quy trinh duyet", "phe duyet", "workflow", "cap duyet", "multi level approval"],
    testId: "dsm-setting-workflows",
  },
  {
    id: "set-locations",
    category: "setting",
    titleFallback: "Quản lý địa điểm & Chi nhánh làm việc",
    descriptionFallback: "Thiết lập trụ sở, văn phòng, bán kính chấm công",
    to: ROUTES.SETTINGS.LOCATIONS,
    icon: MapPin,
    gradient: "from-emerald-600 to-green-700 shadow-emerald-600/25",
    permissions: ["workspace:settings:read"],
    keywords: ["chi nhanh", "van phong", "dia diem", "locations", "toa nha", "tru so"],
    testId: "dsm-setting-locations",
  },
  {
    id: "set-roles",
    category: "setting",
    titleFallback: "Phân quyền & Vai trò thành viên",
    descriptionFallback: "Ma trận quyền hạn, vai trò quản lý và nhân viên",
    to: ROUTES.SETTINGS.ROLES,
    icon: Lock,
    gradient: "from-red-600 to-rose-700 shadow-red-600/25",
    permissions: ["members:roles:manage"],
    keywords: ["phan quyen", "vai tro", "roles", "permissions", "ma tran quyen"],
    testId: "dsm-setting-roles",
  },
  {
    id: "set-audit-logs",
    category: "setting",
    titleFallback: "Nhật ký kiểm toán hệ thống (Audit Logs)",
    descriptionFallback: "Truy vết toàn bộ thao tác, thay đổi dữ liệu và đăng nhập",
    to: ROUTES.SETTINGS.AUDIT_LOGS,
    icon: ListTodo,
    gradient: "from-slate-700 to-slate-900 shadow-slate-700/25",
    permissions: ["workspace:settings:read", "infra:monitor"],
    keywords: ["audit log", "nhat ky", "lich su thao tac", "truy vet", "kiem toan"],
    testId: "dsm-setting-audit-logs",
  },
  {
    id: "set-infra-monitor",
    category: "setting",
    titleFallback: "Giám sát tiến trình nền (Worker Monitor)",
    descriptionFallback: "Trạng thái Redis, queue jobs, worker throughput",
    to: ROUTES.INFRA_MONITOR,
    icon: Activity,
    gradient: "from-cyan-500 to-blue-700 shadow-cyan-500/25",
    systemRoles: ["admin", "platform_admin"],
    keywords: ["giam sat", "worker monitor", "queue", "system health", "redis", "tien trinh"],
    testId: "dsm-setting-infra-monitor",
  },
  {
    id: "set-organization",
    category: "setting",
    titleFallback: "Cơ cấu tổ chức & Giai đoạn việc làm",
    descriptionFallback: "Quản lý phòng ban, chu kỳ thử việc và chức danh",
    to: ROUTES.SETTINGS.ORGANIZATION,
    icon: Building2,
    gradient: "from-blue-600 to-slate-800 shadow-blue-600/25",
    permissions: ["employees:read", "departments:read"],
    keywords: ["co cau to chuc", "giai doan viec lam", "employment phases", "phong ban", "organization"],
    testId: "dsm-setting-organization",
  },
  {
    id: "set-payroll-config",
    category: "setting",
    titleFallback: "Cấu hình bảng lương & Thuế",
    descriptionFallback: "Chu kỳ trả lương, công thức tính thuế và bảo hiểm xã hội",
    to: ROUTES.SETTINGS.PAYROLL,
    icon: Banknote,
    gradient: "from-emerald-600 to-teal-800 shadow-emerald-600/25",
    permissions: ["payroll:read", "payroll:settings:manage"],
    keywords: ["cau hinh luong", "chu ky luong", "bao hiem", "thue thu nhap", "payroll settings"],
    testId: "dsm-setting-payroll-config",
  },
  {
    id: "set-billing",
    category: "setting",
    titleFallback: "Gói cước & Đăng ký dịch vụ (Billing)",
    descriptionFallback: "Quản lý gói doanh nghiệp, dung lượng và thông tin thanh toán",
    to: ROUTES.SETTINGS.BILLING,
    icon: CreditCard,
    gradient: "from-purple-600 to-indigo-800 shadow-purple-600/25",
    permissions: ["workspace:settings:read"],
    keywords: ["goi cuoc", "billing", "thanh toan", "gia han", "dung luong", "hoa don"],
    testId: "dsm-setting-billing",
  },
  {
    id: "set-custom-fields",
    category: "setting",
    titleFallback: "Trường dữ liệu tùy biến (Custom Fields)",
    descriptionFallback: "Tạo thêm trường thông tin cho nhân sự, hợp đồng và phòng ban",
    to: ROUTES.SETTINGS.CUSTOM_FIELDS,
    icon: Braces,
    gradient: "from-cyan-600 to-teal-800 shadow-cyan-600/25",
    permissions: ["workspace:settings:read"],
    keywords: ["truong tuy bien", "custom fields", "them cot", "thong tin bo sung", "truong du lieu"],
    testId: "dsm-setting-custom-fields",
  },
  {
    id: "set-permissions-matrix",
    category: "setting",
    titleFallback: "Ma trận quyền hạn chi tiết (Permissions)",
    descriptionFallback: "Xem chi tiết từng quyền hạn hành động trong hệ thống",
    to: ROUTES.SETTINGS.PERMISSIONS,
    icon: KeyRound,
    gradient: "from-red-600 to-rose-800 shadow-red-600/25",
    permissions: ["members:roles:manage"],
    keywords: ["ma tran quyen", "danh sach quyen", "permissions", "chi tiet quyen", "access rights"],
    testId: "dsm-setting-permissions-matrix",
  },
  {
    id: "set-lifecycle-templates",
    category: "setting",
    titleFallback: "Mẫu danh sách kiểm tra Onboarding & Offboarding",
    descriptionFallback: "Quy trình thủ tục tiếp nhận nhân viên mới và bàn giao nghỉ việc",
    to: ROUTES.SETTINGS.LIFECYCLE_TEMPLATES,
    icon: ClipboardList,
    gradient: "from-indigo-600 to-blue-800 shadow-indigo-600/25",
    permissions: ["workspace:settings:read"],
    keywords: ["onboarding", "offboarding", "checklist", "thu tuc tiep nhan", "ban giao", "nghi viec"],
    testId: "dsm-setting-lifecycle-templates",
  },
  {
    id: "set-condition-types",
    category: "setting",
    titleFallback: "Danh mục loại điều kiện làm việc",
    descriptionFallback: "Thiết lập các điều kiện lao động đặc thù và phụ cấp",
    to: ROUTES.SETTINGS.CONDITION_TYPES,
    icon: Tags,
    gradient: "from-teal-600 to-cyan-800 shadow-teal-600/25",
    permissions: ["workspace:settings:read"],
    keywords: ["dieu kien lam viec", "condition types", "phu cap", "lao dong nang nhoc", "che do"],
    testId: "dsm-setting-condition-types",
  },
  {
    id: "set-storage",
    category: "setting",
    titleFallback: "Cấu hình lưu trữ tệp & MinIO (Storage)",
    descriptionFallback: "Quản lý kho lưu trữ tài liệu, bucket và chứng chỉ S3",
    to: ROUTES.SETTINGS.STORAGE,
    icon: Database,
    gradient: "from-slate-700 to-zinc-900 shadow-slate-700/25",
    permissions: ["workspace:settings:read"],
    keywords: ["luu tru", "minio", "s3", "storage", "dung luong tep", "bucket"],
    testId: "dsm-setting-storage",
  },
  {
    id: "set-econtracts-integration",
    category: "setting",
    titleFallback: "Tích hợp ký số hợp đồng điện tử",
    descriptionFallback: "Cấu hình nhà cung cấp dịch vụ e-Sign và chứng thư",
    to: ROUTES.SETTINGS.ECONTRACTS,
    icon: Scale,
    gradient: "from-blue-600 to-indigo-800 shadow-blue-600/25",
    permissions: ["workspace:settings:read"],
    keywords: ["ky so", "chu ky so", "e-sign", "econtracts integration", "chung thu so"],
    testId: "dsm-setting-econtracts-integration",
  },
  {
    id: "set-system-smtp",
    category: "setting",
    titleFallback: "Cấu hình máy chủ gửi thư SMTP",
    descriptionFallback: "Thiết lập tài khoản email gửi thông báo hệ thống và mã OTP",
    to: ROUTES.SYSTEM_SMTP,
    icon: Mail,
    gradient: "from-sky-600 to-blue-800 shadow-sky-600/25",
    systemRoles: ["admin", "platform_admin"],
    keywords: ["smtp", "email", "gui thu", "mail server", "otp email", "thong bao"],
    testId: "dsm-setting-system-smtp",
  },
  {
    id: "set-system-tenants",
    category: "setting",
    titleFallback: "Quản trị danh sách Workspace / Tenants",
    descriptionFallback: "Tạo và quản lý các tổ chức, không gian làm việc trên nền tảng",
    to: ROUTES.SYSTEM_TENANTS,
    icon: Building2,
    gradient: "from-indigo-600 to-violet-800 shadow-indigo-600/25",
    systemRoles: ["admin", "platform_admin"],
    keywords: ["tenants", "workspaces", "cong ty", "to chuc", "quan tri tenant", "he thong"],
    testId: "dsm-setting-system-tenants",
  },
  {
    id: "set-system-roles",
    category: "setting",
    titleFallback: "Quản trị vai trò nền tảng (Platform Roles)",
    descriptionFallback: "Phân cấp các vai trò quản trị viên cấp cao toàn hệ thống",
    to: ROUTES.SYSTEM_ROLES,
    icon: ShieldCheck,
    gradient: "from-rose-600 to-red-800 shadow-rose-600/25",
    systemRoles: ["admin", "platform_admin"],
    keywords: ["platform roles", "system roles", "vai tro he thong", "admin tong", "superadmin"],
    testId: "dsm-setting-system-roles",
  },
];

// ── Master Feature Registry ─────────────────────────────────────────

export function getAllFeatures(): FeatureItem[] {
  const appFeatures = APP_REGISTRY.map(appToFeature);
  return [...appFeatures, ...ACTION_FEATURES, ...SETTING_FEATURES];
}

// ── Permission Filtering ────────────────────────────────────────────

export function filterFeatures(
  features: FeatureItem[],
  hasAnyPermission: (...perms: string[]) => boolean,
  systemRoles: string[],
): FeatureItem[] {
  return features.filter((item) => {
    if (item.featureFlag && !isFeatureEnabled(item.featureFlag)) {
      return false;
    }
    if (item.systemRoles && item.systemRoles.length > 0) {
      return item.systemRoles.some((role) => systemRoles.includes(role));
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
