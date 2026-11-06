import {
  AlertTriangle,
  Calendar,
  CheckCircle2,
  Clock,
  Sparkles,
  XCircle,
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/utils";

export type AttendanceSemanticStatus =
  | "on_time"
  | "late"
  | "early_leave"
  | "absent"
  | "no_show"
  | "overtime"
  | "holiday"
  | "leave_paid"
  | "leave_unpaid"
  | "recorded";

export interface AttendanceStatusBadgeProps {
  className?: string;
  customLabel?: string;
  minutes?: number;
  showIcon?: boolean;
  status: AttendanceSemanticStatus;
}

const statusConfig: Record<
  AttendanceSemanticStatus,
  {
    icon: React.ReactNode;
    style: string;
    labelKey: string;
    defaultLabel: string;
  }
> = {
  on_time: {
    icon: <CheckCircle2 className="h-3.5 w-3.5" />,
    style:
      "bg-emerald-50 text-emerald-700 border-emerald-200/80 dark:bg-emerald-950/60 dark:text-emerald-300 dark:border-emerald-800/60",
    labelKey: "attendance.status.on_time",
    defaultLabel: "Đúng giờ",
  },
  recorded: {
    icon: <CheckCircle2 className="h-3.5 w-3.5" />,
    style:
      "bg-emerald-50 text-emerald-700 border-emerald-200/80 dark:bg-emerald-950/60 dark:text-emerald-300 dark:border-emerald-800/60",
    labelKey: "attendance_logs.status.recorded",
    defaultLabel: "Đã ghi nhận",
  },
  late: {
    icon: <AlertTriangle className="h-3.5 w-3.5" />,
    style:
      "bg-amber-50 text-amber-700 border-amber-200/80 dark:bg-amber-950/60 dark:text-amber-300 dark:border-amber-800/60",
    labelKey: "attendance.status.late",
    defaultLabel: "Đi muộn",
  },
  early_leave: {
    icon: <AlertTriangle className="h-3.5 w-3.5" />,
    style:
      "bg-amber-50 text-amber-700 border-amber-200/80 dark:bg-amber-950/60 dark:text-amber-300 dark:border-amber-800/60",
    labelKey: "attendance.status.early_leave",
    defaultLabel: "Về sớm",
  },
  absent: {
    icon: <XCircle className="h-3.5 w-3.5" />,
    style:
      "bg-rose-50 text-rose-700 border-rose-200/80 dark:bg-rose-950/60 dark:text-rose-300 dark:border-rose-800/60",
    labelKey: "attendance.status.absent",
    defaultLabel: "Vắng mặt",
  },
  no_show: {
    icon: <XCircle className="h-3.5 w-3.5" />,
    style:
      "bg-rose-50 text-rose-700 border-rose-200/80 dark:bg-rose-950/60 dark:text-rose-300 dark:border-rose-800/60",
    labelKey: "attendance.status.no_show",
    defaultLabel: "Không quét vân tay",
  },
  overtime: {
    icon: <Clock className="h-3.5 w-3.5" />,
    style:
      "bg-violet-50 text-violet-700 border-violet-200/80 dark:bg-violet-950/60 dark:text-violet-300 dark:border-violet-800/60",
    labelKey: "attendance.status.overtime",
    defaultLabel: "Tăng ca (OT)",
  },
  holiday: {
    icon: <Sparkles className="h-3.5 w-3.5" />,
    style:
      "bg-purple-50 text-purple-700 border-purple-200/80 dark:bg-purple-950/60 dark:text-purple-300 dark:border-purple-800/60",
    labelKey: "attendance.status.holiday",
    defaultLabel: "Ngày lễ",
  },
  leave_paid: {
    icon: <Calendar className="h-3.5 w-3.5" />,
    style:
      "bg-sky-50 text-sky-700 border-sky-200/80 dark:bg-sky-950/60 dark:text-sky-300 dark:border-sky-800/60",
    labelKey: "attendance.status.leave_paid",
    defaultLabel: "Nghỉ có lương",
  },
  leave_unpaid: {
    icon: <Calendar className="h-3.5 w-3.5" />,
    style:
      "bg-slate-100 text-slate-700 border-slate-200/80 dark:bg-slate-800 dark:text-slate-300 dark:border-slate-700",
    labelKey: "attendance.status.leave_unpaid",
    defaultLabel: "Nghỉ không lương",
  },
};

export function AttendanceStatusBadge({
  status,
  minutes,
  customLabel,
  showIcon = true,
  className,
}: AttendanceStatusBadgeProps) {
  const { t } = useTranslation();
  const config = statusConfig[status] ?? statusConfig.recorded;

  const baseText = customLabel || t(config.labelKey, config.defaultLabel);
  const displayText =
    minutes !== undefined && minutes > 0 ? `${baseText} (${minutes}p)` : baseText;

  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-bold border tracking-tight transition-all",
        config.style,
        className
      )}
      data-status={status}
      data-testid={`attendance-status-${status}`}
    >
      {showIcon && <span className="shrink-0">{config.icon}</span>}
      <span className="truncate">{displayText}</span>
    </span>
  );
}
