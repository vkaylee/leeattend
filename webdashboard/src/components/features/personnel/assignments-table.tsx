import { Link } from "@tanstack/react-router";
import { ArrowRight, Briefcase, ExternalLink, Pencil, Trash2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { AssignmentStatus } from "@/lib/api";
import type { EmployeeAssignmentWithEmployee } from "@/lib/api/employee-assignments";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import ResponsiveTable from "@/components/ui/responsive-table";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useHorizontalDragScroll } from "@/hooks/use-horizontal-drag-scroll";
import { cn } from "@/lib/utils";

const getStatusColor = (status: AssignmentStatus) => {
  switch (status) {
    case "active":
      return "bg-emerald-50 text-emerald-600 border-emerald-100 dark:bg-emerald-950/40 dark:text-emerald-300 dark:border-emerald-800";
    case "scheduled":
      return "bg-amber-50 text-amber-600 border-amber-100 dark:bg-amber-950/40 dark:text-amber-300 dark:border-amber-800";
    case "completed":
      return "bg-slate-50 text-slate-500 border-slate-100 dark:bg-slate-800 dark:text-slate-400 dark:border-slate-700";
    case "cancelled":
      return "bg-rose-50 text-rose-600 border-rose-100 dark:bg-rose-950/40 dark:text-rose-300 dark:border-rose-800";
    default:
      return "bg-slate-50 text-slate-500";
  }
};

function AssignmentMobileCard({
  assignment,
  formatDate,
  onDelete,
  onEdit,
  t,
}: {
  assignment: EmployeeAssignmentWithEmployee;
  formatDate: (date: string) => string;
  onDelete: (assignment: EmployeeAssignmentWithEmployee) => void;
  onEdit: (assignment: EmployeeAssignmentWithEmployee) => void;
  t: (key: string, options?: { defaultValue?: string }) => string;
}) {
  return (
    <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-xs transition-all active:scale-[0.99] dark:border-slate-800 dark:bg-slate-900 space-y-4">
      {/* Header: Employee info */}
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-start gap-3 min-w-0">
          <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-primary-100 bg-primary-50 text-primary-600 dark:border-primary-800 dark:bg-primary-950 dark:text-primary-400 font-bold uppercase">
            {assignment.employee_name ? assignment.employee_name[0] : "E"}
          </div>
          <div className="min-w-0 flex-1">
            <Link
              className="truncate text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100 hover:text-primary-600 dark:hover:text-primary-400 flex items-center gap-1"
              params={{ employeeId: assignment.employee_id }}
              search={{ tab: "assignments" }}
              to="/employees/$employeeId"
            >
              <span className="truncate">{assignment.employee_name}</span>
              <ExternalLink className="opacity-50 shrink-0" size={12} />
            </Link>
            {!!assignment.employee_code && (
              <div className="mt-0.5 font-mono text-[10px] font-bold uppercase tracking-wider text-slate-400">
                {assignment.employee_code}
              </div>
            )}
          </div>
        </div>
        <span
          className={cn(
            "text-[9px] font-bold uppercase tracking-widest px-2.5 py-0.5 border rounded-full shadow-xs shrink-0",
            getStatusColor(assignment.status)
          )}
        >
          {assignment.status}
        </span>
      </div>

      {/* Locations */}
      <div className="flex items-center gap-2 text-xs font-semibold text-slate-700 dark:text-slate-300 bg-slate-50 dark:bg-slate-800/40 px-3 py-2 rounded-xl">
        <span className="truncate">{assignment.home_location_name || t("assignments.home_location", { defaultValue: "Trụ sở chính" })}</span>
        <ArrowRight className="text-slate-400 shrink-0" size={12} />
        <span className="truncate text-primary-600 dark:text-primary-400 font-bold">
          {assignment.host_location_name || t("assignments.host_location", { defaultValue: "Điểm phái cử" })}
        </span>
      </div>

      {/* Timeline & Notes */}
      <div className="text-xs text-slate-500 dark:text-slate-400 space-y-1">
        <div>
          <span className="font-medium text-slate-400">{t("common.date", { defaultValue: "Thời hạn" })}:</span>{" "}
          <span className="font-semibold text-slate-700 dark:text-slate-300">
            {formatDate(assignment.start_date)}
            {" → "}
            {assignment.end_date ? formatDate(assignment.end_date) : t("assignments.indefinite", { defaultValue: "Chưa xác định" })}
          </span>
        </div>
        {!!assignment.reason && (
          <div className="italic text-slate-500 dark:text-slate-400 text-[11px] line-clamp-2">
            "{assignment.reason}"
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-100 dark:border-slate-800">
        <Button
          aria-label={t("assignments.actions.edit", { defaultValue: "Chỉnh sửa phái cử" })}
          className="h-8 text-xs font-semibold"
          icon={<Pencil size={13} />}
          onClick={() => { onEdit(assignment); }}
          size="sm"
          variant="outline"
        >
          {t("common.edit", { defaultValue: "Sửa" })}
        </Button>
        <Button
          aria-label={t("assignments.actions.delete", { defaultValue: "Xóa phái cử" })}
          className="h-8 text-xs font-semibold text-rose-600 hover:text-rose-700 hover:bg-rose-50 dark:hover:bg-rose-950/30"
          icon={<Trash2 size={13} />}
          onClick={() => { onDelete(assignment); }}
          size="sm"
          variant="ghost"
        >
          {t("common.delete", { defaultValue: "Xóa" })}
        </Button>
      </div>
    </div>
  );
}

export interface AssignmentsTableProps {
  assignments: EmployeeAssignmentWithEmployee[];
  isLoading: boolean;
  onClearFilters: () => void;
  onDelete: (assignment: EmployeeAssignmentWithEmployee) => void;
  onEdit: (assignment: EmployeeAssignmentWithEmployee) => void;
  search: string;
  selectedStatus: string;
}

export function AssignmentsTable({
  assignments,
  isLoading,
  search,
  selectedStatus,
  onClearFilters,
  onEdit,
  onDelete,
}: AssignmentsTableProps) {
  const { t } = useTranslation();
  const { formatDate } = useDateTimeFormatter();
  const {
    onClickCapture: handleHorizontalDragClick,
    setRef: setHorizontalDragScrollRef,
  } = useHorizontalDragScroll<HTMLDivElement>();

  if (isLoading) {
    return (
      <div className="p-8 space-y-4 animate-pulse">
        <div className="h-10 bg-slate-100 dark:bg-slate-800 rounded-xl w-full" />
        <div className="h-16 bg-slate-50 dark:bg-slate-800/60 rounded-xl w-full" />
        <div className="h-16 bg-slate-50 dark:bg-slate-800/60 rounded-xl w-full" />
        <div className="h-16 bg-slate-50 dark:bg-slate-800/60 rounded-xl w-full" />
      </div>
    );
  }

  if (assignments.length === 0) {
    return (
      <EmptyState
        action={
          search || selectedStatus !== "all" ? (
            <Button onClick={onClearFilters} variant="outline">
              {t("common.clear_filters", { defaultValue: "XÓA BỘ LỌC" })}
            </Button>
          ) : undefined
        }
        description={
          search || selectedStatus !== "all"
            ? t("assignments.search_no_results", { defaultValue: "Không tìm thấy đợt phái cử nào khớp với điều kiện tìm kiếm." })
            : t("assignments.empty_state", { defaultValue: "Chưa có đợt phái cử nào được ghi nhận trong không gian làm việc này." })
        }
        icon={<Briefcase size={48} />}
        title={t("assignments.no_assignments_found", { defaultValue: "Không tìm thấy phái cử" })}
      />
    );
  }

  return (
    <ResponsiveTable<EmployeeAssignmentWithEmployee>
      data={assignments}
      emptyState={{
        title: t("assignments.no_assignments_found", { defaultValue: "Không tìm thấy phái cử" }),
        description:
          search || selectedStatus !== "all"
            ? t("assignments.search_no_results", { defaultValue: "Không tìm thấy đợt phái cử nào khớp với điều kiện tìm kiếm." })
            : t("assignments.empty_state", { defaultValue: "Chưa có đợt phái cử nào được ghi nhận trong không gian làm việc này." }),
        icon: <Briefcase size={48} />,
        action:
          search || selectedStatus !== "all" ? (
            <Button onClick={onClearFilters} variant="outline">
              {t("common.clear_filters", { defaultValue: "XÓA BỘ LỌC" })}
            </Button>
          ) : undefined,
      }}
      getCardKey={(assignment) => assignment.id}
      isLoading={isLoading}
      renderCard={(assignment) => (
        <AssignmentMobileCard
          assignment={assignment}
          formatDate={formatDate}
          onDelete={onDelete}
          onEdit={onEdit}
          t={t}
        />
      )}
      renderDesktopTable={() => (
        <div
          className="overflow-x-auto overscroll-contain"
          onClickCapture={handleHorizontalDragClick}
          ref={setHorizontalDragScrollRef}
        >
          <table className="w-full min-w-[980px] text-left border-collapse">
            <thead>
              <tr className="border-b border-slate-100 dark:border-slate-800/60">
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {t("assignments.table.employee", { defaultValue: "NHÂN SỰ" })}
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {t("assignments.table.locations", { defaultValue: "ĐỊA ĐIỂM (TỪ → ĐẾN)" })}
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {t("assignments.table.duration", { defaultValue: "THỜI HẠN" })}
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {t("assignments.table.reason", { defaultValue: "LÝ DO / GHI CHÚ" })}
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {t("common.status", { defaultValue: "TRẠNG THÁI" })}
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest text-right">
                  {t("common.actions", { defaultValue: "THAO TÁC" })}
                </th>
              </tr>
            </thead>
            <tbody>
              {assignments.map((a) => (
                <tr
                  className="border-b border-slate-50 dark:border-slate-800/50 hover:bg-slate-50/40 dark:hover:bg-slate-800/20 transition-colors group"
                  key={a.id}
                >
                  <td className="px-8 py-5">
                    <div className="flex flex-col">
                      <Link
                        className="text-xs font-bold text-slate-800 dark:text-slate-100 uppercase tracking-wide group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors flex items-center gap-1.5"
                        params={{ employeeId: a.employee_id }}
                        search={{ tab: "assignments" }}
                        to="/employees/$employeeId"
                      >
                        <span>{a.employee_name}</span>
                        <ExternalLink className="opacity-0 group-hover:opacity-70 transition-opacity" size={11} />
                      </Link>
                      {!!a.employee_code && (
                        <span className="text-[10px] font-mono text-slate-400 mt-0.5 font-bold">
                          {a.employee_code}
                        </span>
                      )}
                    </div>
                  </td>
                  <td className="px-8 py-5">
                    <div className="flex items-center gap-2 text-xs font-semibold text-slate-600 dark:text-slate-300">
                      <span>{a.home_location_name || t("assignments.home_location", { defaultValue: "Trụ sở chính" })}</span>
                      <ArrowRight className="text-slate-400 shrink-0" size={12} />
                      <span className="font-bold text-primary-600 dark:text-primary-400">
                        {a.host_location_name || t("assignments.host_location", { defaultValue: "Điểm phái cử" })}
                      </span>
                    </div>
                  </td>
                  <td className="px-8 py-5 text-xs text-slate-600 dark:text-slate-300 font-medium">
                    {formatDate(a.start_date)}
                    {" → "}
                    {a.end_date ? formatDate(a.end_date) : t("assignments.indefinite", { defaultValue: "Chưa xác định" })}
                  </td>
                  <td className="px-8 py-5 text-xs text-slate-500 dark:text-slate-400 max-w-xs truncate">
                    {a.reason || "—"}
                  </td>
                  <td className="px-8 py-5">
                    <span
                      className={cn(
                        "text-[9px] font-bold uppercase tracking-widest px-2.5 py-0.5 border rounded-full shadow-xs",
                        getStatusColor(a.status)
                      )}
                    >
                      {a.status}
                    </span>
                  </td>
                  <td className="px-8 py-5 text-right">
                    <div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                      <Button
                        aria-label={t("assignments.actions.edit", { defaultValue: "Chỉnh sửa" })}
                        className="p-2 text-slate-400 hover:text-primary-600 hover:bg-primary-50 dark:hover:bg-primary-900/30 rounded-xl transition-colors h-auto w-auto shadow-none"
                        onClick={() => { onEdit(a); }}
                        size="icon"
                        variant="ghost"
                      >
                        <Pencil size={14} />
                      </Button>
                      <Button
                        aria-label={t("assignments.actions.delete", { defaultValue: "Xóa" })}
                        className="p-2 text-slate-400 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-900/30 rounded-xl transition-colors h-auto w-auto shadow-none"
                        onClick={() => { onDelete(a); }}
                        size="icon"
                        variant="ghost"
                      >
                        <Trash2 size={14} />
                      </Button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    />
  );
}
