import { Download, HelpCircle, User } from "lucide-react";
import React, { useImperativeHandle, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { EmployeeIdentityCell } from "@/components/common/employee-identity-cell";
import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import { Skeleton } from "@/components/ui/skeleton";
import { useHorizontalDragScroll } from "@/hooks/use-horizontal-drag-scroll";
import type { DailyAttendanceReport } from "@/lib/api";
import { Employee } from "@/lib/api/types";
import { cn } from "@/lib/utils";
import { formatDayKey, makeTimesheetCellKey } from "@/pages/reports/hooks/use-monthly-timesheet-data";

import {
  ATTENDANCE_CODE_REGISTRY,
  getAllAttendanceCodes,
  getAttendanceCodeConfig,
} from "./attendance-code-registry";
import { resolveDailyAttendanceCode } from "./attendance-code-resolver";

export interface TimesheetMatrixGridHandle {
  exportCSV: () => void;
}

export interface TimesheetMatrixGridProps {
  attendanceMap?: Map<string, DailyAttendanceReport>;
  employees: Employee[];
  isLoading?: boolean;
  isLoadingAttendance?: boolean;
  month: number;
  onEmployeeClick?: (employee: Employee) => void;
  year: number;
}

// Days of week in Vietnamese
const DAY_NAMES = ["CN", "T2", "T3", "T4", "T5", "T6", "T7"];

export const TimesheetMatrixGrid = React.forwardRef<
  TimesheetMatrixGridHandle,
  TimesheetMatrixGridProps
>(
  (
    {
      attendanceMap,
      employees,
      isLoading = false,
      isLoadingAttendance = false,
      month,
      onEmployeeClick,
      year,
    },
    ref
  ) => {
    const { t } = useTranslation();
    const [isLegendOpen, setIsLegendOpen] = useState(false);
    const {
      onClickCapture: handleHorizontalDragClick,
      setRef: setHorizontalDragScrollRef,
    } = useHorizontalDragScroll<HTMLDivElement>();

    // Calculate days in the selected month
    const daysInMonth = useMemo(() => {
      return new Date(year, month, 0).getDate();
    }, [year, month]);

    const daysList = useMemo(() => {
      return Array.from({ length: daysInMonth }, (_, i) => {
        const dayNum = i + 1;
        const date = new Date(year, month - 1, dayNum);
        const dayOfWeek = date.getDay(); // 0 is Sunday, 6 is Saturday
        const isWeekend = dayOfWeek === 0 || dayOfWeek === 6;
        return {
          date,
          dayName: DAY_NAMES[dayOfWeek],
          dayNum,
          dayOfWeek,
          isWeekend,
        };
      });
    }, [daysInMonth, year, month]);

    // Real attendance code resolver using O(1) indexed attendanceMap
    const getEmployeeDayCode = (empId: string, day: number, isWeekend: boolean) => {
      if (attendanceMap) {
        const dateStr = formatDayKey(year, month, day);
        const cellKey = makeTimesheetCellKey(empId, dateStr);
        const row = attendanceMap.get(cellKey);
        return resolveDailyAttendanceCode(row, { isWeekend });
      }
      if (isWeekend) return getAttendanceCodeConfig("Ro");
      return getAttendanceCodeConfig("-");
    };

    const registeredCodes = useMemo(() => getAllAttendanceCodes(), []);

    const handleExportMatrixCSV = React.useCallback(() => {
      const headers = [
        "Mã NV",
        "Họ và tên",
        "Phòng ban",
        ...daysList.map((d) => `${d.dayNum} (${d.dayName})`),
        "Tổng ngày công",
        "Số ngày phép",
        "Tổng công tính lương",
      ];

      const rows = employees.map((emp) => {
        let workDays = 0;
        let leaveDays = 0;
        const dayCodes = daysList.map((d) => {
          const config = getEmployeeDayCode(emp.id, d.dayNum, d.isWeekend);
          if (config.category === "work") workDays += config.paidMultiplier;
          if (config.category === "paid_leave") leaveDays += config.paidMultiplier;
          return config.code;
        });
        const payableDays = workDays + leaveDays;

        const empCode = emp.employee_id_number || (emp as { employee_code?: string }).employee_code || (emp.id ? emp.id.slice(0, 8) : "");
        const deptName = emp.department_name || (emp as { department?: string }).department || "";
        return [
          `"${empCode}"`,
          `"${emp.full_name ?? ""}"`,
          `"${deptName}"`,
          ...dayCodes.map((c) => `"${c}"`),
          `"${workDays.toFixed(1)}"`,
          `"${leaveDays.toFixed(1)}"`,
          `"${payableDays.toFixed(1)}"`,
        ].join(",");
      });

      const csvContent = "\uFEFF" + [headers.join(","), ...rows].join("\n");
      const blob = new Blob([csvContent], { type: "text/csv;charset=utf-8;" });
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.setAttribute("download", `Bang_cong_thang_${month}_${year}.csv`);
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    }, [daysList, employees, month, year, attendanceMap]);

    // Expose export function to parent via ref
    useImperativeHandle(
      ref,
      () => ({
        exportCSV: handleExportMatrixCSV,
      }),
      [handleExportMatrixCSV]
    );

    return (
      <div
        className="flex flex-col h-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-3xl shadow-sm overflow-hidden animate-in fade-in duration-300"
        data-testid="timesheet-matrix-grid"
      >
        {/* Header bar: info & legend toggle */}
        <div className="flex flex-wrap items-center justify-between gap-4 px-5 py-3.5 border-b border-slate-200 dark:border-slate-800 bg-slate-50/50 dark:bg-slate-800/30">
          <div className="flex items-center gap-2.5">
            <span className="text-sm font-semibold text-slate-800 dark:text-slate-100">
              {t("reports.timesheet_matrix_31_days", "Bảng công 31 ngày")}
            </span>
            <span className="text-xs font-medium text-slate-500 dark:text-slate-400">
              • {employees.length} {t("reports.employees", "nhân viên")}
            </span>
          </div>

          <div className="flex items-center gap-2">
            <Button
              aria-label={t("reports.legend", "Bảng giải thích ký hiệu")}
              className="flex items-center gap-1.5 text-xs text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 px-3 py-1.5 rounded-xl transition-all font-semibold normal-case tracking-normal"
              onClick={() => {
                setIsLegendOpen((prev) => !prev);
              }}
              variant="ghost"
            >
              <HelpCircle size={15} />
              <span>{t("reports.legend", "Ký hiệu công")}</span>
            </Button>
          </div>
        </div>

        {/* Expandable Legend Bar */}
        {isLegendOpen && (
          <div className="px-4 py-2.5 border-b border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/60 flex flex-wrap items-center gap-2 text-xs animate-in slide-in-from-top-2 duration-200">
            <span className="font-semibold text-slate-700 dark:text-slate-200 shrink-0">
              {t("reports.legend_rules", "Quy ước ký hiệu:")}
            </span>
            {registeredCodes.map((c) => (
              <div
                key={c.code}
                title={c.description}
              >
                <span
                  className={cn(
                    "font-bold px-1.5 py-0.5 rounded text-[11px] border",
                    c.badgeClass
                  )}
                >
                  {c.code}
                </span>
                <span className="text-slate-600 dark:text-slate-300">{c.label}</span>
              </div>
            ))}
          </div>
        )}

        {/* Main Matrix Table */}
        <div
          className="flex-1 min-h-0 overflow-auto overscroll-contain"
          onClickCapture={handleHorizontalDragClick}
          ref={setHorizontalDragScrollRef}
        >
          {isLoading || isLoadingAttendance ? (
            <div className="p-12 space-y-3">
              {Array.from({ length: 6 }).map((_, i) => (
                <Skeleton className="h-10 w-full rounded-xl" key={i} />
              ))}
            </div>
          ) : employees.length === 0 ? (
            <EmptyState
              description={t("common.no_records_found")}
              icon={<User className="w-10 h-10 text-slate-400" />}
              title={t("common.no_records_found", "Không tìm thấy nhân viên")}
            />
          ) : (
            <table className="w-full text-left border-collapse border-spacing-0 text-xs">
              <thead>
                <tr className="bg-slate-100/80 dark:bg-slate-800/80 sticky top-0 z-20 backdrop-blur-xs border-b border-slate-200 dark:border-slate-700">
                  {/* Fixed Left Employee Column */}
                  <th className="sticky left-0 z-30 bg-slate-100 dark:bg-slate-800 px-4 py-3 font-semibold text-slate-700 dark:text-slate-200 w-64 min-w-[16rem] border-r border-slate-200 dark:border-slate-700 shadow-xs">
                    {t("reports.table.employee", "Nhân viên")}
                  </th>

                  {/* Day Columns */}
                  {daysList.map((d) => (
                    <th
                      className={cn(
                        "px-1.5 py-2 text-center font-medium min-w-[36px] w-[36px] border-r border-slate-200/60 dark:border-slate-700/60",
                        d.isWeekend
                          ? "bg-amber-500/10 dark:bg-amber-500/5 text-amber-700 dark:text-amber-300 font-bold"
                          : "text-slate-600 dark:text-slate-300"
                      )}
                      key={d.dayNum}
                    >
                      <div className="text-[11px] leading-tight font-bold">{d.dayNum}</div>
                      <div className="text-[9px] uppercase opacity-75">{d.dayName}</div>
                    </th>
                  ))}

                  {/* Fixed Right Summary Header Columns */}
                  <th className="sticky right-[112px] z-30 px-3 py-3 text-center font-semibold text-emerald-700 dark:text-emerald-300 bg-emerald-500/10 min-w-[56px] border-l border-slate-200 dark:border-slate-700 shadow-[-2px_0_5px_rgba(0,0,0,0.04)]">
                    {t("reports.table.std_hours", "Công")}
                  </th>
                  <th className="sticky right-[56px] z-30 px-3 py-3 text-center font-semibold text-blue-700 dark:text-blue-300 bg-blue-500/10 min-w-[56px] shadow-[-2px_0_5px_rgba(0,0,0,0.04)]">
                    {t("reports.table.leave_days", "Phép")}
                  </th>
                  <th className="sticky right-0 z-30 px-3 py-3 text-center font-semibold text-purple-700 dark:text-purple-300 bg-purple-500/10 min-w-[56px] shadow-[-2px_0_5px_rgba(0,0,0,0.04)]">
                    {t("reports.table.total_hours", "Tổng tính")}
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
                {employees.map((emp) => {
                  let totalWork = 0;
                  let totalLeave = 0;

                  return (
                    <tr
                      className="hover:bg-slate-50/80 dark:hover:bg-slate-800/40 transition-colors group"
                      key={emp.id}
                    >
                      {/* Fixed Left Employee Cell using shared EmployeeIdentityCell */}
                      <td className="sticky left-0 z-10 bg-white dark:bg-slate-900 group-hover:bg-slate-50 dark:group-hover:bg-slate-800 px-4 py-2 border-r border-slate-200 dark:border-slate-700 shadow-xs">
                        <EmployeeIdentityCell
                          avatarSize="sm"
                          code={emp.employee_id_number || (emp as { employee_code?: string }).employee_code || (emp.id ? emp.id.slice(0, 8) : "—")}
                          employerName={emp.department_name || (emp as { department?: string }).department || "Toàn công ty"}
                          name={emp.full_name}
                          onClick={() => {
                            onEmployeeClick?.(emp);
                          }}
                        />
                      </td>

                      {/* Day Cells */}
                      {daysList.map((d) => {
                        const config = getEmployeeDayCode(emp.id, d.dayNum, d.isWeekend);

                        if (config.category === "work") {
                          totalWork += config.paidMultiplier;
                        } else if (config.category === "paid_leave") {
                          totalLeave += config.paidMultiplier;
                        }

                        return (
                          <td
                            className={cn(
                              "px-1 py-1.5 text-center border-r border-slate-100 dark:border-slate-800",
                              d.isWeekend && "bg-amber-500/5 dark:bg-amber-500/5"
                            )}
                            key={d.dayNum}
                          >
                            <span
                              className={cn(
                                "inline-flex items-center justify-center w-7 h-6 rounded-md font-bold text-[10px] border shadow-2xs cursor-default select-none transition-transform hover:scale-110",
                                config.badgeClass
                              )}
                              title={`${d.dayNum}/${month}/${year} (${d.dayName}): ${config.label} - ${config.description}`}
                            >
                              {config.code}
                            </span>
                          </td>
                        );
                      })}

                      {/* Right Summary Totals */}
                      <td className="sticky right-[112px] z-10 px-2 py-2 text-center font-bold text-emerald-700 dark:text-emerald-300 bg-emerald-500/5 border-l border-slate-200 dark:border-slate-700 shadow-[-2px_0_5px_rgba(0,0,0,0.04)]">
                        {totalWork.toFixed(1)}
                      </td>
                      <td className="sticky right-[56px] z-10 px-2 py-2 text-center font-bold text-blue-700 dark:text-blue-300 bg-blue-500/5 shadow-[-2px_0_5px_rgba(0,0,0,0.04)]">
                        {totalLeave.toFixed(1)}
                      </td>
                      <td className="sticky right-0 z-10 px-2 py-2 text-center font-black text-purple-700 dark:text-purple-300 bg-purple-500/5 shadow-[-2px_0_5px_rgba(0,0,0,0.04)]">
                        {(totalWork + totalLeave).toFixed(1)}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          )}
        </div>
      </div>
    );
  }
);

TimesheetMatrixGrid.displayName = "TimesheetMatrixGrid";
