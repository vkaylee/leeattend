import { HelpCircle, User } from "lucide-react";
import { forwardRef, useCallback, useImperativeHandle, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import type { DailyAttendanceReport } from "@/lib/api";

import { EmployeeIdentityCell } from "@/components/common/employee-identity-cell";
import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import { Skeleton } from "@/components/ui/skeleton";
import type { Employee } from "@/lib/api/types";

import { useHorizontalDragScroll } from "@/hooks/use-horizontal-drag-scroll";
import { cn } from "@/lib/utils";
import { formatDayKey, makeTimesheetCellKey } from "@/pages/reports/hooks/use-monthly-timesheet-data";

import {
  getAllAttendanceCodes,
  getAttendanceCodeConfig,
  type AttendanceCodeConfig,
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

interface TimesheetDay {
  date: Date;
  dayName: string;
  dayNum: number;
  dayOfWeek: number;
  isWeekend: boolean;
}

interface LegacyEmployeeFields {
  department?: string;
  employee_code?: string;
}

type GetEmployeeDayCode = (
  employeeId: string,
  day: number,
  isWeekend: boolean,
) => AttendanceCodeConfig;

const DAY_NAMES = ["CN", "T2", "T3", "T4", "T5", "T6", "T7"];
const LEGACY_EMPLOYEE_FIELDS = (employee: Employee) => employee as Employee & LegacyEmployeeFields;

function getDaysInMonth(year: number, month: number): number {
  return new Date(year, month, 0).getDate();
}

function createDaysList(year: number, month: number, daysInMonth: number): TimesheetDay[] {
  return Array.from({ length: daysInMonth }, (_, index) => {
    const dayNum = index + 1;
    const date = new Date(year, month - 1, dayNum);
    const dayOfWeek = date.getDay();

    return {
      date,
      dayName: DAY_NAMES[dayOfWeek],
      dayNum,
      dayOfWeek,
      isWeekend: dayOfWeek === 0 || dayOfWeek === 6,
    };
  });
}

function firstTruthyValue(...values: (string | undefined | null)[]): string | undefined {
  for (const value of values) {
    if (value) return value;
  }
  return undefined;
}

function getEmployeeCode(employee: Employee, fallback: string): string {
  const legacyEmployee = LEGACY_EMPLOYEE_FIELDS(employee);
  return firstTruthyValue(
    employee.employee_id_number,
    legacyEmployee.employee_code,
    employee.id ? employee.id.slice(0, 8) : fallback,
  ) ?? fallback;
}

function getDepartmentName(employee: Employee, fallback: string): string {
  const legacyEmployee = LEGACY_EMPLOYEE_FIELDS(employee);
  return firstTruthyValue(employee.department_name, legacyEmployee.department) ?? fallback;
}

interface TimesheetCsvOptions {
  daysList: TimesheetDay[];
  employees: Employee[];
  getEmployeeDayCode: GetEmployeeDayCode;
  month: number;
  year: number;
}

function downloadTimesheetCsv({
  daysList,
  employees,
  getEmployeeDayCode,
  month,
  year,
}: TimesheetCsvOptions): void {
  const headers = [
    "Mã NV",
    "Họ và tên",
    "Phòng ban",
    ...daysList.map((day) => `${day.dayNum} (${day.dayName})`),
    "Tổng ngày công",
    "Số ngày phép",
    "Tổng công tính lương",
  ];

  const rows = employees.map((employee) => createCsvEmployeeRow(employee, daysList, getEmployeeDayCode));
  const csvContent = "﻿" + [headers.join(","), ...rows].join("\n");
  const blob = new Blob([csvContent], { type: "text/csv;charset=utf-8;" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");

  link.href = url;
  link.setAttribute("download", `Bang_cong_thang_${month}_${year}.csv`);
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

function createCsvEmployeeRow(
  employee: Employee,
  daysList: TimesheetDay[],
  getEmployeeDayCode: GetEmployeeDayCode,
): string {
  let workDays = 0;
  let leaveDays = 0;
  const dayCodes = daysList.map((day) => {
    const config = getEmployeeDayCode(employee.id, day.dayNum, day.isWeekend);
    if (config.category === "work") workDays += config.paidMultiplier;
    if (config.category === "paid_leave") leaveDays += config.paidMultiplier;
    return config.code;
  });
  const payableDays = workDays + leaveDays;

  return [
    `"${getEmployeeCode(employee, "")}"`,
    `"${employee.full_name}"`,
    `"${getDepartmentName(employee, "")}"`,
    ...dayCodes.map((code) => `"${code}"`),
    `"${workDays.toFixed(1)}"`,
    `"${leaveDays.toFixed(1)}"`,
    `"${payableDays.toFixed(1)}"`,
  ].join(",");
}

interface GridHeaderProps {
  employeeCount: number;
  onToggleLegend: () => void;
}

function GridHeader({ employeeCount, onToggleLegend }: GridHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-wrap items-center justify-between gap-4 px-5 py-3.5 border-b border-slate-200 dark:border-slate-800 bg-slate-50/50 dark:bg-slate-800/30">
      <div className="flex items-center gap-2.5">
        <span className="text-sm font-semibold text-slate-800 dark:text-slate-100">
          {t("reports.timesheet_matrix_31_days", "Bảng công 31 ngày")}
        </span>
        <span className="text-xs font-medium text-slate-500 dark:text-slate-400">
          • {employeeCount} {t("reports.employees", "nhân viên")}
        </span>
      </div>

      <div className="flex items-center gap-2">
        <Button
          aria-label={t("reports.legend", "Bảng giải thích ký hiệu")}
          className="flex items-center gap-1.5 text-xs text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 px-3 py-1.5 rounded-xl transition-all font-semibold normal-case tracking-normal"
          onClick={onToggleLegend}
          variant="ghost"
        >
          <HelpCircle size={15} />
          <span>{t("reports.legend", "Ký hiệu công")}</span>
        </Button>
      </div>
    </div>
  );
}

function LegendBar() {
  const { t } = useTranslation();
  const registeredCodes = useMemo(() => getAllAttendanceCodes(), []);

  return (
    <div className="px-4 py-2.5 border-b border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/60 flex flex-wrap items-center gap-2 text-xs animate-in slide-in-from-top-2 duration-200">
      <span className="font-semibold text-slate-700 dark:text-slate-200 shrink-0">
        {t("reports.legend_rules", "Quy ước ký hiệu:")}
      </span>
      {registeredCodes.map((code) => (
        <div key={code.code} title={code.description}>
          <span
            className={cn(
              "font-bold px-1.5 py-0.5 rounded text-[11px] border",
              code.badgeClass,
            )}
          >
            {code.code}
          </span>
          <span className="text-slate-600 dark:text-slate-300">{code.label}</span>
        </div>
      ))}
    </div>
  );
}

function LoadingState() {
  return (
    <div className="p-12 space-y-3">
      {Array.from({ length: 6 }).map((_, index) => (
        <Skeleton className="h-10 w-full rounded-xl" key={index} />
      ))}
    </div>
  );
}

function EmptyMatrixState() {
  const { t } = useTranslation();

  return (
    <EmptyState
      description={t("common.no_records_found")}
      icon={<User className="w-10 h-10 text-slate-400" />}
      title={t("common.no_records_found", "Không tìm thấy nhân viên")}
    />
  );
}

interface MatrixTableProps {
  daysList: TimesheetDay[];
  employees: Employee[];
  getEmployeeDayCode: GetEmployeeDayCode;
  month: number;
  onEmployeeClick?: (employee: Employee) => void;
  year: number;
}

function MatrixTable({
  daysList,
  employees,
  getEmployeeDayCode,
  month,
  onEmployeeClick,
  year,
}: MatrixTableProps) {
  const { t } = useTranslation();

  return (
    <table className="w-full text-left border-collapse border-spacing-0 text-xs">
      <thead>
        <tr className="bg-slate-100/80 dark:bg-slate-800/80 sticky top-0 z-20 backdrop-blur-xs border-b border-slate-200 dark:border-slate-700">
          <th className="sticky left-0 z-30 bg-slate-100 dark:bg-slate-800 px-4 py-3 font-semibold text-slate-700 dark:text-slate-200 w-64 min-w-[16rem] border-r border-slate-200 dark:border-slate-700 shadow-xs">
            {t("reports.table.employee", "Nhân viên")}
          </th>
          {daysList.map((day) => (
            <DayHeader day={day} key={day.dayNum} />
          ))}
          <SummaryHeader className="right-[112px]" label={t("reports.table.std_hours", "Công")} tone="emerald" />
          <SummaryHeader className="right-[56px]" label={t("reports.table.leave_days", "Phép")} tone="blue" />
          <SummaryHeader className="right-0" label={t("reports.table.total_hours", "Tổng tính")} tone="purple" />
        </tr>
      </thead>
      <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
        {employees.map((employee) => (
          <MatrixRow
            daysList={daysList}
            employee={employee}
            getEmployeeDayCode={getEmployeeDayCode}
            key={employee.id}
            month={month}
            onEmployeeClick={onEmployeeClick}
            year={year}
          />
        ))}
      </tbody>
    </table>
  );
}

function DayHeader({ day }: { day: TimesheetDay }) {
  return (
    <th
      className={cn(
        "px-1.5 py-2 text-center font-medium min-w-[36px] w-[36px] border-r border-slate-200/60 dark:border-slate-700/60",
        day.isWeekend
          ? "bg-amber-500/10 dark:bg-amber-500/5 text-amber-700 dark:text-amber-300 font-bold"
          : "text-slate-600 dark:text-slate-300",
      )}
    >
      <div className="text-[11px] leading-tight font-bold">{day.dayNum}</div>
      <div className="text-[9px] uppercase opacity-75">{day.dayName}</div>
    </th>
  );
}

interface SummaryHeaderProps {
  className: string;
  label: string;
  tone: "blue" | "emerald" | "purple";
}

const SUMMARY_HEADER_TONE_CLASSES: Record<SummaryHeaderProps["tone"], string> = {
  blue: "text-blue-700 dark:text-blue-300 bg-blue-500/10",
  emerald: "text-emerald-700 dark:text-emerald-300 bg-emerald-500/10",
  purple: "text-purple-700 dark:text-purple-300 bg-purple-500/10",
};

function SummaryHeader({ className, label, tone }: SummaryHeaderProps) {
  return (
    <th
      className={cn(
        "sticky z-30 px-3 py-3 text-center font-semibold min-w-[56px] shadow-[-2px_0_5px_rgba(0,0,0,0.04)]",
        className,
        SUMMARY_HEADER_TONE_CLASSES[tone],
        tone === "emerald" && "border-l border-slate-200 dark:border-slate-700",
      )}
    >
      {label}
    </th>
  );
}

interface MatrixRowProps {
  daysList: TimesheetDay[];
  employee: Employee;
  getEmployeeDayCode: GetEmployeeDayCode;
  month: number;
  onEmployeeClick?: (employee: Employee) => void;
  year: number;
}

interface DayCellData {
  config: AttendanceCodeConfig;
  day: TimesheetDay;
}

interface RowTotals {
  leave: number;
  work: number;
}

function getRowCells(
  employeeId: string,
  daysList: TimesheetDay[],
  getEmployeeDayCode: GetEmployeeDayCode,
): { cells: DayCellData[]; totals: RowTotals } {
  const totals: RowTotals = { work: 0, leave: 0 };
  const cells = daysList.map((day) => {
    const config = getEmployeeDayCode(employeeId, day.dayNum, day.isWeekend);
    if (config.category === "work") totals.work += config.paidMultiplier;
    if (config.category === "paid_leave") totals.leave += config.paidMultiplier;
    return { day, config };
  });

  return { cells, totals };
}

function MatrixRow({
  daysList,
  employee,
  getEmployeeDayCode,
  month,
  onEmployeeClick,
  year,
}: MatrixRowProps) {
  const { cells, totals } = getRowCells(employee.id, daysList, getEmployeeDayCode);

  return (
    <tr className="hover:bg-slate-50/80 dark:hover:bg-slate-800/40 transition-colors group">
      <td className="sticky left-0 z-10 bg-white dark:bg-slate-900 group-hover:bg-slate-50 dark:group-hover:bg-slate-800 px-4 py-2 border-r border-slate-200 dark:border-slate-700 shadow-xs">
        <EmployeeIdentityCell
          avatarSize="sm"
          code={getEmployeeCode(employee, "—")}
          employerName={getDepartmentName(employee, "Toàn công ty")}
          name={employee.full_name}
          onClick={() => {
            onEmployeeClick?.(employee);
          }}
        />
      </td>
      {cells.map(({ day, config }) => (
        <DayCell config={config} day={day} key={day.dayNum} month={month} year={year} />
      ))}
      <SummaryCell className="right-[112px]" tone="emerald" value={totals.work} />
      <SummaryCell className="right-[56px]" tone="blue" value={totals.leave} />
      <SummaryCell className="right-0" strong tone="purple" value={totals.work + totals.leave} />
    </tr>
  );
}

interface DayCellProps {
  config: AttendanceCodeConfig;
  day: TimesheetDay;
  month: number;
  year: number;
}

function DayCell({ config, day, month, year }: DayCellProps) {
  return (
    <td
      className={cn(
        "px-1 py-1.5 text-center border-r border-slate-100 dark:border-slate-800",
        day.isWeekend && "bg-amber-500/5 dark:bg-amber-500/5",
      )}
    >
      <span
        className={cn(
          "inline-flex items-center justify-center w-7 h-6 rounded-md font-bold text-[10px] border shadow-2xs cursor-default select-none transition-transform hover:scale-110",
          config.badgeClass,
        )}
        title={`${day.dayNum}/${month}/${year} (${day.dayName}): ${config.label} - ${config.description}`}
      >
        {config.code}
      </span>
    </td>
  );
}

interface SummaryCellProps {
  className: string;
  strong?: boolean;
  tone: "blue" | "emerald" | "purple";
  value: number;
}

const SUMMARY_CELL_TONE_CLASSES: Record<SummaryCellProps["tone"], string> = {
  blue: "text-blue-700 dark:text-blue-300 bg-blue-500/5",
  emerald: "text-emerald-700 dark:text-emerald-300 bg-emerald-500/5 border-l border-slate-200 dark:border-slate-700",
  purple: "text-purple-700 dark:text-purple-300 bg-purple-500/5",
};

function SummaryCell({ className, strong = false, tone, value }: SummaryCellProps) {
  return (
    <td
      className={cn(
        "sticky z-10 px-2 py-2 text-center font-bold shadow-[-2px_0_5px_rgba(0,0,0,0.04)]",
        className,
        SUMMARY_CELL_TONE_CLASSES[tone],
        strong && "font-black",
      )}
    >
      {value.toFixed(1)}
    </td>
  );
}

export const TimesheetMatrixGrid = forwardRef<
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
    ref,
  ) => {
    const [isLegendOpen, setIsLegendOpen] = useState(false);
    const {
      onClickCapture: handleHorizontalDragClick,
      setRef: setHorizontalDragScrollRef,
    } = useHorizontalDragScroll<HTMLDivElement>();
    const daysInMonth = useMemo(() => getDaysInMonth(year, month), [month, year]);
    const daysList = useMemo(
      () => createDaysList(year, month, daysInMonth),
      [daysInMonth, month, year],
    );
    const getEmployeeDayCode = useCallback<GetEmployeeDayCode>(
      (employeeId, day, isWeekend) => {
        if (attendanceMap) {
          const dateStr = formatDayKey(year, month, day);
          const cellKey = makeTimesheetCellKey(employeeId, dateStr);
          const row = attendanceMap.get(cellKey);
          return resolveDailyAttendanceCode(row, { isWeekend });
        }
        if (isWeekend) return getAttendanceCodeConfig("Ro");
        return getAttendanceCodeConfig("-");
      },
      [attendanceMap, month, year],
    );
    const handleExportMatrixCSV = useCallback(
      () => {
        downloadTimesheetCsv({
          daysList,
          employees,
          getEmployeeDayCode,
          month,
          year,
        });
      },
      [daysList, employees, getEmployeeDayCode, month, year],
    );

    useImperativeHandle(ref, () => ({ exportCSV: handleExportMatrixCSV }), [handleExportMatrixCSV]);

    return (
      <div
        className="flex flex-col h-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-3xl shadow-sm overflow-hidden animate-in fade-in duration-300"
        data-testid="timesheet-matrix-grid"
      >
        <GridHeader
          employeeCount={employees.length}
          onToggleLegend={() => {
            setIsLegendOpen((previous) => !previous);
          }}
        />
        {isLegendOpen ? <LegendBar /> : null}
        <div
          className="flex-1 min-h-0 overflow-auto overscroll-contain"
          onClickCapture={handleHorizontalDragClick}
          ref={setHorizontalDragScrollRef}
        >
          {isLoading || isLoadingAttendance ? (
            <LoadingState />
          ) : employees.length === 0 ? (
            <EmptyMatrixState />
          ) : (
            <MatrixTable
              daysList={daysList}
              employees={employees}
              getEmployeeDayCode={getEmployeeDayCode}
              month={month}
              onEmployeeClick={onEmployeeClick}
              year={year}
            />
          )}
        </div>
      </div>
    );
  },
);

TimesheetMatrixGrid.displayName = "TimesheetMatrixGrid";
