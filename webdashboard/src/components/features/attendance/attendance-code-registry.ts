/**
 * attendance-code-registry.ts
 *
 * Implements Open/Closed Principle (OCP) for Attendance and Timesheet Grid codes.
 * New codes can be registered here with their display metadata, paid multiplier,
 * and categories without modifying grid renderers or calculating modules.
 */

export interface AttendanceCodeConfig {
  badgeClass: string;
  category: "work" | "paid_leave" | "unpaid_leave" | "rest" | "holiday";
  code: string;
  description: string;
  isHoliday: boolean;
  isLeave: boolean;
  label: string;
  paidMultiplier: number; // e.g. 1.0, 0.5, 0
}

export const ATTENDANCE_CODE_REGISTRY: Record<string, AttendanceCodeConfig> = {
  X: {
    code: "X",
    label: "Đủ công",
    description: "Làm việc đủ ca tiêu chuẩn (1.0 công)",
    paidMultiplier: 1.0,
    isLeave: false,
    isHoliday: false,
    category: "work",
    badgeClass: "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border-emerald-500/30",
  },
  "X/2": {
    code: "X/2",
    label: "Nửa công",
    description: "Làm việc nửa ca tiêu chuẩn (0.5 công)",
    paidMultiplier: 0.5,
    isLeave: false,
    isHoliday: false,
    category: "work",
    badgeClass: "bg-teal-500/15 text-teal-700 dark:text-teal-300 border-teal-500/30",
  },
  P: {
    code: "P",
    label: "Phép năm",
    description: "Nghỉ phép năm hưởng nguyên lương (1.0 công có lương)",
    paidMultiplier: 1.0,
    isLeave: true,
    isHoliday: false,
    category: "paid_leave",
    badgeClass: "bg-blue-500/15 text-blue-700 dark:text-blue-300 border-blue-500/30",
  },
  O: {
    code: "Ô",
    label: "Ốm đau",
    description: "Nghỉ ốm đau có giấy chứng nhận y tế hưởng chế độ BHXH",
    paidMultiplier: 0,
    isLeave: true,
    isHoliday: false,
    category: "paid_leave",
    badgeClass: "bg-purple-500/15 text-purple-700 dark:text-purple-300 border-purple-500/30",
  },
  TS: {
    code: "TS",
    label: "Thai sản",
    description: "Nghỉ thai sản theo quy định BHXH",
    paidMultiplier: 0,
    isLeave: true,
    isHoliday: false,
    category: "paid_leave",
    badgeClass: "bg-pink-500/15 text-pink-700 dark:text-pink-300 border-pink-500/30",
  },
  CT: {
    code: "CT",
    label: "Công tác",
    description: "Đi công tác hoặc đào tạo bên ngoài theo lệnh điều động (1.0 công)",
    paidMultiplier: 1.0,
    isLeave: false,
    isHoliday: false,
    category: "work",
    badgeClass: "bg-indigo-500/15 text-indigo-700 dark:text-indigo-300 border-indigo-500/30",
  },
  KL: {
    code: "KL",
    label: "Không lương",
    description: "Nghỉ việc riêng không hưởng lương theo thỏa thuận",
    paidMultiplier: 0,
    isLeave: true,
    isHoliday: false,
    category: "unpaid_leave",
    badgeClass: "bg-slate-500/15 text-slate-700 dark:text-slate-300 border-slate-500/30",
  },
  Ro: {
    code: "Ro",
    label: "Nghỉ ca",
    description: "Ngày nghỉ ca tuần theo mẫu lịch làm việc (Roster Off)",
    paidMultiplier: 0,
    isLeave: false,
    isHoliday: false,
    category: "rest",
    badgeClass: "bg-amber-500/15 text-amber-700 dark:text-amber-300 border-amber-500/30",
  },
  L: {
    code: "L",
    label: "Nghỉ Lễ",
    description: "Nghỉ ngày lễ, tết theo Luật Lao động (1.0 công hưởng lương)",
    paidMultiplier: 1.0,
    isLeave: false,
    isHoliday: true,
    category: "holiday",
    badgeClass: "bg-rose-500/15 text-rose-700 dark:text-rose-300 border-rose-500/30",
  },
  V: {
    code: "V",
    label: "Vắng",
    description: "Vắng mặt không phép / Chưa ghi nhận quẹt thẻ",
    paidMultiplier: 0,
    isLeave: false,
    isHoliday: false,
    category: "unpaid_leave",
    badgeClass: "bg-red-500/20 text-red-700 dark:text-red-300 border-red-500/40",
  },
};

const DEFAULT_CODE_CONFIG: AttendanceCodeConfig = {
  code: "-",
  label: "Chưa có dữ liệu",
  description: "Chưa ghi nhận dữ liệu công",
  paidMultiplier: 0,
  isLeave: false,
  isHoliday: false,
  category: "rest",
  badgeClass: "bg-slate-100 dark:bg-slate-800 text-slate-400 border-slate-200 dark:border-slate-700",
};

/**
 * Returns configuration metadata for a given attendance code.
 */
export function getAttendanceCodeConfig(code: string): AttendanceCodeConfig {
  const normalized = code.trim().toUpperCase();
  return ATTENDANCE_CODE_REGISTRY[normalized] ?? {
    ...DEFAULT_CODE_CONFIG,
    code,
    label: code,
  };
}

/**
 * Returns all active codes registered in the system.
 */
export function getAllAttendanceCodes(): AttendanceCodeConfig[] {
  return Object.values(ATTENDANCE_CODE_REGISTRY);
}
