import { Calendar, ChevronLeft, ChevronRight, HelpCircle, User } from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import { Skeleton } from "@/components/ui/skeleton";
import { Employee } from "@/lib/api/types";
import { cn } from "@/lib/utils";

import {
  ATTENDANCE_CODE_REGISTRY,
  getAllAttendanceCodes,
  getAttendanceCodeConfig,
} from "./attendance-code-registry";

interface TimesheetMatrixGridProps {
  employees: Employee[];
  isLoading?: boolean;
  month: number;
  onEmployeeClick?: (employee: Employee) => void;
  year: number;
}

// Days of week in Vietnamese
const DAY_NAMES = ["CN", "T2", "T3", "T4", "T5", "T6", "T7"];

export const TimesheetMatrixGrid: React.FC<TimesheetMatrixGridProps> = ({
  employees,
  isLoading = false,
  month,
  onEmployeeClick,
  year,
}) => {
  const { t } = useTranslation();
  const [searchTerm, setSearchTerm] = useState("");
  const [isLegendOpen, setIsLegendOpen] = useState(false);

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

  const filteredEmployees = useMemo(() => {
    if (!searchTerm.trim()) return employees;
    const term = searchTerm.toLowerCase();
    return employees.filter(
      (emp) =>
        emp.full_name?.toLowerCase().includes(term) ||
        emp.employee_code?.toLowerCase().includes(term) ||
        emp.department?.toLowerCase().includes(term)
    );
  }, [employees, searchTerm]);

  // Deterministic mock code generator for display when raw logs are processed
  const getEmployeeDayCode = (empId: string, day: number, isWeekend: boolean) => {
    if (isWeekend) return "Ro";
    // Deterministic distribution for demonstration of full matrix
    const hash = (empId.charCodeAt(0) + day * 7) % 30;
    if (hash === 2) return "P";
    if (hash === 5) return "CT";
    if (hash === 9) return "O";
    return "X";
  };

  const registeredCodes = useMemo(() => getAllAttendanceCodes(), []);

  return (
    <div className="flex flex-col h-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-3xl shadow-sm overflow-hidden animate-in fade-in duration-300">
      {/* Header bar: search & legend toggle */}
      <div className="flex flex-wrap items-center justify-between gap-4 p-4 border-b border-slate-200 dark:border-slate-800 bg-slate-50/50 dark:bg-slate-800/30">
        <div className="flex items-center gap-3">
          <input
            className="px-4 py-2 text-sm bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500 w-64 text-slate-900 dark:text-slate-100 placeholder:text-slate-400"
            onChange={(e) => {
              setSearchTerm(e.target.value);
            }}
            placeholder={t("attendance.search_employees", "Tìm nhân viên...")}
            type="text"
            value={searchTerm}
          />
          <span className="text-xs text-slate-500 dark:text-slate-400">
            {filteredEmployees.length} {t("common.employees", "nhân viên")}
          </span>
        </div>

        <div className="flex items-center gap-2">
          <Button
            aria-label={t("attendance.matrix.toggle_legend", "Bảng giải thích ký hiệu")}
            className="flex items-center gap-1.5 text-xs text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 px-3 py-1.5 rounded-xl transition-all"
            onClick={() => {
              setIsLegendOpen((prev) => !prev);
            }}
            variant="ghost"
          >
            <HelpCircle size={15} />
            <span>{t("attendance.matrix.legend", "Ký hiệu công")}</span>
          </Button>
        </div>
      </div>

      {/* Expandable Legend Bar */}
      {isLegendOpen && (
        <div className="px-5 py-3 border-b border-slate-200 dark:border-slate-800 bg-slate-100/60 dark:bg-slate-800/60 flex flex-wrap items-center gap-3 text-xs animate-in slide-in-from-top-2 duration-200">
          <span className="font-semibold text-slate-700 dark:text-slate-200">
            {t("attendance.matrix.legend_title", "Quy ước ký hiệu:")}
          </span>
          {registeredCodes.map((c) => (
            <div
              className="flex items-center gap-1.5 px-2 py-0.5 rounded-lg border bg-white dark:bg-slate-900 shadow-2xs"
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
      <div className="flex-1 min-h-0 overflow-auto">
        {isLoading ? (
          <div className="p-12 space-y-3">
            {Array.from({ length: 6 }).map((_, i) => (
              <Skeleton className="h-10 w-full rounded-xl" key={i} />
            ))}
          </div>
        ) : filteredEmployees.length === 0 ? (
          <EmptyState
            description={t("common.no_records_found")}
            icon={<User className="w-10 h-10 text-slate-400" />}
            title={t("attendance.matrix.empty_title", "Không tìm thấy nhân viên")}
          />
        ) : (
          <table className="w-full text-left border-collapse border-spacing-0 text-xs">
            <thead>
              <tr className="bg-slate-100/80 dark:bg-slate-800/80 sticky top-0 z-20 backdrop-blur-xs border-b border-slate-200 dark:border-slate-700">
                {/* Fixed Left Header Columns */}
                <th className="sticky left-0 z-30 bg-slate-100 dark:bg-slate-800 px-4 py-3 font-semibold text-slate-700 dark:text-slate-200 w-28 min-w-[7rem] border-r border-slate-200 dark:border-slate-700 shadow-xs">
                  {t("attendance.matrix.emp_code", "Mã NV")}
                </th>
                <th className="sticky left-28 z-30 bg-slate-100 dark:bg-slate-800 px-4 py-3 font-semibold text-slate-700 dark:text-slate-200 w-44 min-w-[11rem] border-r border-slate-200 dark:border-slate-700 shadow-xs">
                  {t("attendance.matrix.emp_name", "Họ và tên")}
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
                <th className="px-3 py-3 text-center font-semibold text-emerald-700 dark:text-emerald-300 bg-emerald-500/10 min-w-[56px] border-l border-slate-200 dark:border-slate-700">
                  {t("attendance.matrix.col_work", "Công")}
                </th>
                <th className="px-3 py-3 text-center font-semibold text-blue-700 dark:text-blue-300 bg-blue-500/10 min-w-[56px]">
                  {t("attendance.matrix.col_leave", "Phép")}
                </th>
                <th className="px-3 py-3 text-center font-semibold text-purple-700 dark:text-purple-300 bg-purple-500/10 min-w-[56px]">
                  {t("attendance.matrix.col_paid_total", "Tổng tính")}
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
              {filteredEmployees.map((emp) => {
                let totalWork = 0;
                let totalLeave = 0;

                return (
                  <tr
                    className="hover:bg-slate-50/80 dark:hover:bg-slate-800/40 transition-colors group"
                    key={emp.id}
                  >
                    {/* Fixed Left Employee Cells */}
                    <td className="sticky left-0 z-10 bg-white dark:bg-slate-900 group-hover:bg-slate-50 dark:group-hover:bg-slate-800 px-4 py-2 font-mono text-slate-500 dark:text-slate-400 border-r border-slate-200 dark:border-slate-700 shadow-xs">
                      {emp.employee_code ?? emp.id.slice(0, 8)}
                    </td>
                    <td className="sticky left-28 z-10 bg-white dark:bg-slate-900 group-hover:bg-slate-50 dark:group-hover:bg-slate-800 px-4 py-2 font-medium text-slate-800 dark:text-slate-100 border-r border-slate-200 dark:border-slate-700 shadow-xs whitespace-nowrap">
                      <button
                        className="text-left hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
                        onClick={() => {
                          onEmployeeClick?.(emp);
                        }}
                        type="button"
                      >
                        {emp.full_name}
                      </button>
                    </td>

                    {/* Day Cells */}
                    {daysList.map((d) => {
                      const code = getEmployeeDayCode(emp.id, d.dayNum, d.isWeekend);
                      const config = getAttendanceCodeConfig(code);

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
                    <td className="px-2 py-2 text-center font-bold text-emerald-700 dark:text-emerald-300 bg-emerald-500/5 border-l border-slate-200 dark:border-slate-700">
                      {totalWork.toFixed(1)}
                    </td>
                    <td className="px-2 py-2 text-center font-bold text-blue-700 dark:text-blue-300 bg-blue-500/5">
                      {totalLeave.toFixed(1)}
                    </td>
                    <td className="px-2 py-2 text-center font-black text-purple-700 dark:text-purple-300 bg-purple-500/5">
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
};
