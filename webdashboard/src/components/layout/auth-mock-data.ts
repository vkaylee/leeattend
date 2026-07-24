// ── Mock data for auth layout dashboard preview ──────────────────────

export const ATTENDANCE_BARS = [72, 88, 65, 92, 78, 95, 83] as const;
export const DAY_LABELS = ["M", "T", "W", "T", "F", "S", "S"] as const;

export interface MockEmployee {
  hue: string;
  initials: string;
  name: string;
  role: string;
  status: "present" | "late" | "leave";
  time: string;
}

export const MOCK_EMPLOYEES: MockEmployee[] = [
  { name: "Nguyen Van A", role: "Engineering",  status: "present", time: "08:02", initials: "NA", hue: "150" },
  { name: "Tran Thi B",  role: "Marketing",   status: "late",    time: "09:14", initials: "TB", hue: "220" },
  { name: "Le Van C",   role: "HR",           status: "leave",   time: "--:--", initials: "LC", hue: "280" },
  { name: "Pham Thi D", role: "Finance",      status: "present", time: "07:58", initials: "PD", hue: "35"  },
];

export const STATUS_STYLE = {
  present: { dot: "bg-emerald-400", badge: "text-emerald-300 bg-emerald-400/15", label: "Present" },
  late:    { dot: "bg-amber-400",   badge: "text-amber-300   bg-amber-400/15",   label: "Late"    },
  leave:   { dot: "bg-slate-400",   badge: "text-slate-300   bg-slate-400/15",   label: "Leave"   },
} as const;

export const STAT_CARDS = [
  { value: "247", label: "Employees",    color: "from-emerald-400/25 to-teal-400/10",    border: "border-emerald-400/20" },
  { value: "94%", label: "On Time",      color: "from-blue-400/25    to-indigo-400/10",   border: "border-blue-400/20"    },
  { value: "12",  label: "On Leave",     color: "from-violet-400/25  to-purple-400/10",  border: "border-violet-400/20"  },
] as const;

export const FEATURE_PILLS = [
  { icon: "✦", text: "auth.brand.feature_1_title" },
  { icon: "✦", text: "auth.brand.feature_2_title" },
  { icon: "✦", text: "auth.brand.feature_3_title" },
  { icon: "✦", text: "auth.brand.feature_4_title" },
] as const;
