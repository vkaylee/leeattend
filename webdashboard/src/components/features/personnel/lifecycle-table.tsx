import { Link } from "@tanstack/react-router";
import {
  ArrowLeftRight,
  ArrowRight,
  CheckSquare,
  ExternalLink,
  ListTodo,
  UserMinus,
  UserPlus,
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { LifecycleInstanceSummary } from "@/lib/api";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import ResponsiveTable from "@/components/ui/responsive-table";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useHorizontalDragScroll } from "@/hooks/use-horizontal-drag-scroll";
import { cn } from "@/lib/utils";

import { formatChecklistTypeLabel } from "../employees/lifecycle-formatters";

function getTypeIcon(type: string): React.ReactNode {
  switch (type.toLowerCase()) {
    case "onboarding":
      return <UserPlus size={16} />;
    case "offboarding":
      return <UserMinus size={16} />;
    case "probation_end":
      return <CheckSquare size={16} />;
    case "transfer":
      return <ArrowLeftRight size={16} />;
    default:
      return <ListTodo size={16} />;
  }
}

function getTypeBadgeClass(type: string): string {
  switch (type.toLowerCase()) {
    case "onboarding":
      return "bg-emerald-50 text-emerald-700 border-emerald-100 dark:bg-emerald-950/40 dark:text-emerald-300 dark:border-emerald-800";
    case "offboarding":
      return "bg-rose-50 text-rose-700 border-rose-100 dark:bg-rose-950/40 dark:text-rose-300 dark:border-rose-800";
    case "probation_end":
      return "bg-primary-50 text-primary-700 border-primary-100 dark:bg-primary-950/40 dark:text-primary-300 dark:border-primary-800";
    case "transfer":
      return "bg-amber-50 text-amber-700 border-amber-100 dark:bg-amber-950/40 dark:text-amber-300 dark:border-amber-800";
    default:
      return "bg-slate-50 text-slate-700 border-slate-100 dark:bg-slate-800 dark:text-slate-300 dark:border-slate-700";
  }
}

function LifecycleMobileCard({
  instance,
  formatDate,
  t,
}: {
  formatDate: (date: string) => string;
  instance: LifecycleInstanceSummary;
  t: (key: string, options?: { defaultValue?: string }) => string;
}) {
  const isCompleted = instance.status === "completed";

  return (
    <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-xs transition-all active:scale-[0.99] dark:border-slate-800 dark:bg-slate-900 space-y-4">
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-start gap-3 min-w-0">
          <div
            className={cn(
              "flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border shadow-xs",
              getTypeBadgeClass(instance.checklist_type)
            )}
          >
            {getTypeIcon(instance.checklist_type)}
          </div>
          <div className="min-w-0 flex-1">
            <Link
              className="truncate text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100 hover:text-primary-600 dark:hover:text-primary-400 flex items-center gap-1"
              params={{ employeeId: instance.employee_id }}
              search={{ tab: "lifecycle" }}
              to="/employees/$employeeId"
            >
              <span className="truncate">{instance.employee_name}</span>
              <ExternalLink className="opacity-50 shrink-0" size={12} />
            </Link>
            <div className="flex items-center gap-2 mt-0.5 text-[10px] font-mono text-slate-400 font-bold uppercase">
              {!!instance.employee_code && <span>{instance.employee_code}</span>}
              {!!instance.department_name && (
                <>
                  <span>•</span>
                  <span className="font-sans normal-case text-slate-500 dark:text-slate-400 font-medium truncate">
                    {instance.department_name}
                  </span>
                </>
              )}
            </div>
          </div>
        </div>

        <span
          className={cn(
            "text-[9px] font-bold uppercase tracking-widest px-2.5 py-0.5 border rounded-full shadow-xs shrink-0",
            isCompleted
              ? "bg-emerald-50 text-emerald-600 border-emerald-100 dark:bg-emerald-950/40 dark:text-emerald-300 dark:border-emerald-800"
              : "bg-blue-50 text-blue-600 border-blue-100 dark:bg-blue-950/40 dark:text-blue-300 dark:border-blue-800"
          )}
        >
          {isCompleted
            ? t("lifecycle.status.completed", { defaultValue: "Đã hoàn thành" })
            : t("lifecycle.status.in_progress", { defaultValue: "Đang tiến hành" })}
        </span>
      </div>

      <div className="text-xs font-semibold text-slate-700 dark:text-slate-300 bg-slate-50 dark:bg-slate-800/40 px-3 py-2 rounded-xl flex items-center justify-between">
        <span className="font-bold">{instance.template_name || formatChecklistTypeLabel(instance.checklist_type, t)}</span>
        <span className="text-[10px] text-slate-400 font-medium">
          {instance.completed_items}/{instance.total_items} {t("lifecycle.items_label", { defaultValue: "việc" })}
        </span>
      </div>

      {/* Progress Bar */}
      <div className="space-y-1">
        <div className="flex items-center justify-between text-[11px] font-bold">
          <span className="text-slate-400 uppercase tracking-wider">{t("lifecycle.progress", { defaultValue: "Tiến độ" })}</span>
          <span
            className={cn(
              isCompleted ? "text-emerald-600 dark:text-emerald-400" : "text-primary-600 dark:text-primary-400"
            )}
          >
            {instance.progress_percent}%
          </span>
        </div>
        <div className="w-full bg-slate-100 dark:bg-slate-800 h-2 rounded-full overflow-hidden">
          <div
            className={cn(
              "h-full transition-all duration-500 rounded-full",
              isCompleted ? "bg-emerald-500" : "bg-primary-600"
            )}
            style={{ width: `${instance.progress_percent}%` }}
          />
        </div>
      </div>

      <div className="text-xs text-slate-500 dark:text-slate-400 flex items-center justify-between pt-2 border-t border-slate-100 dark:border-slate-800">
        <span className="text-[11px] text-slate-400">
          {t("lifecycle.started_at", { defaultValue: "Bắt đầu" })}: {formatDate(instance.started_at)}
        </span>
        <Link
          className="text-xs font-bold text-primary-600 dark:text-primary-400 hover:underline flex items-center gap-1"
          params={{ employeeId: instance.employee_id }}
          search={{ tab: "lifecycle" }}
          to="/employees/$employeeId"
        >
          <span>{t("lifecycle.view_detail", { defaultValue: "Xem chi tiết" })}</span>
          <ArrowRight size={13} />
        </Link>
      </div>
    </div>
  );
}

export interface LifecycleTableProps {
  instances: LifecycleInstanceSummary[];
  isLoading: boolean;
  onClearFilters: () => void;
  search: string;
  selectedStatus: string;
  selectedType: string;
}

export function LifecycleTable({
  instances,
  isLoading,
  search,
  selectedType,
  selectedStatus,
  onClearFilters,
}: LifecycleTableProps) {
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

  const hasFilter = Boolean(search || selectedType !== "all" || selectedStatus !== "all");

  if (instances.length === 0) {
    return (
      <EmptyState
        action={
          hasFilter ? (
            <Button onClick={onClearFilters} variant="outline">
              {t("common.clear_filters", { defaultValue: "XÓA BỘ LỌC" })}
            </Button>
          ) : undefined
        }
        description={
          hasFilter
            ? t("lifecycle.search_no_results", {
                defaultValue: "Không tìm thấy quy trình vòng đời nhân sự nào phù hợp với bộ lọc.",
              })
            : t("lifecycle.empty_state", {
                defaultValue: "Chưa có quy trình onboarding, thử việc hay bàn giao nào được khởi tạo.",
              })
        }
        icon={<ListTodo size={48} />}
        title={t("lifecycle.no_instances_found", { defaultValue: "Không tìm thấy quy trình nhân sự" })}
      />
    );
  }

  return (
    <ResponsiveTable<LifecycleInstanceSummary>
      data={instances}
      emptyState={{
        title: t("lifecycle.no_instances_found", { defaultValue: "Không tìm thấy quy trình nhân sự" }),
        description: hasFilter
          ? t("lifecycle.search_no_results", {
              defaultValue: "Không tìm thấy quy trình vòng đời nhân sự nào phù hợp với bộ lọc.",
            })
          : t("lifecycle.empty_state", {
              defaultValue: "Chưa có quy trình onboarding, thử việc hay bàn giao nào được khởi tạo.",
            }),
        icon: <ListTodo size={48} />,
        action: hasFilter ? (
          <Button onClick={onClearFilters} variant="outline">
            {t("common.clear_filters", { defaultValue: "XÓA BỘ LỌC" })}
          </Button>
        ) : undefined,
      }}
      getCardKey={(item) => item.id}
      isLoading={isLoading}
      renderCard={(item) => (
        <LifecycleMobileCard
          formatDate={formatDate}
          instance={item}
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
                  {t("lifecycle.table.employee", { defaultValue: "NHÂN SỰ" })}
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {t("lifecycle.table.process_name", { defaultValue: "QUY TRÌNH CHECKLIST" })}
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {t("lifecycle.table.progress", { defaultValue: "TIẾN ĐỘ THỰC HIỆN" })}
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {t("lifecycle.table.status", { defaultValue: "TRẠNG THÁI" })}
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {t("lifecycle.table.started_date", { defaultValue: "NGÀY BẮT ĐẦU" })}
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest text-right">
                  {t("common.actions", { defaultValue: "THAO TÁC" })}
                </th>
              </tr>
            </thead>
            <tbody>
              {instances.map((c) => {
                const isCompleted = c.status === "completed";

                return (
                  <tr
                    className="border-b border-slate-50 dark:border-slate-800/50 hover:bg-slate-50/40 dark:hover:bg-slate-800/20 transition-colors group"
                    key={c.id}
                  >
                    <td className="px-8 py-5">
                      <div className="flex flex-col">
                        <Link
                          className="text-xs font-bold text-slate-800 dark:text-slate-100 uppercase tracking-wide group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors flex items-center gap-1.5"
                          params={{ employeeId: c.employee_id }}
                          search={{ tab: "lifecycle" }}
                          to="/employees/$employeeId"
                        >
                          <span>{c.employee_name}</span>
                          <ExternalLink className="opacity-0 group-hover:opacity-70 transition-opacity" size={11} />
                        </Link>
                        <div className="flex items-center gap-2 mt-0.5 text-[10px] font-mono text-slate-400 font-bold">
                          {!!c.employee_code && <span>{c.employee_code}</span>}
                          {!!c.department_name && (
                            <>
                              <span>•</span>
                              <span className="font-sans normal-case text-slate-500 dark:text-slate-400 font-medium">
                                {c.department_name}
                              </span>
                            </>
                          )}
                        </div>
                      </div>
                    </td>
                    <td className="px-8 py-5">
                      <div className="flex items-center gap-2.5">
                        <div
                          className={cn(
                            "w-8 h-8 rounded-xl flex items-center justify-center shrink-0 border",
                            getTypeBadgeClass(c.checklist_type)
                          )}
                        >
                          {getTypeIcon(c.checklist_type)}
                        </div>
                        <div className="flex flex-col">
                          <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-wide">
                            {c.template_name || formatChecklistTypeLabel(c.checklist_type, t)}
                          </span>
                          <span className="text-[10px] text-slate-400 font-medium">
                            {formatChecklistTypeLabel(c.checklist_type, t)}
                          </span>
                        </div>
                      </div>
                    </td>
                    <td className="px-8 py-5 min-w-[180px]">
                      <div className="space-y-1.5">
                        <div className="flex items-center justify-between text-[11px] font-bold">
                          <span className="text-slate-500 dark:text-slate-400 font-mono text-[10px]">
                            {c.completed_items}/{c.total_items} {t("lifecycle.items_label", { defaultValue: "việc" })}
                          </span>
                          <span
                            className={cn(
                              "font-mono",
                              isCompleted
                                ? "text-emerald-600 dark:text-emerald-400"
                                : "text-primary-600 dark:text-primary-400"
                            )}
                          >
                            {c.progress_percent}%
                          </span>
                        </div>
                        <div className="w-full bg-slate-100 dark:bg-slate-800 h-2 rounded-full overflow-hidden">
                          <div
                            className={cn(
                              "h-full transition-all duration-500 rounded-full",
                              isCompleted ? "bg-emerald-500" : "bg-primary-600"
                            )}
                            style={{ width: `${c.progress_percent}%` }}
                          />
                        </div>
                      </div>
                    </td>
                    <td className="px-8 py-5">
                      <span
                        className={cn(
                          "text-[9px] font-bold uppercase tracking-widest px-2.5 py-0.5 border rounded-full shadow-xs",
                          isCompleted
                            ? "bg-emerald-50 text-emerald-600 border-emerald-100 dark:bg-emerald-950/40 dark:text-emerald-300 dark:border-emerald-800"
                            : "bg-blue-50 text-blue-600 border-blue-100 dark:bg-blue-950/40 dark:text-blue-300 dark:border-blue-800"
                        )}
                      >
                        {isCompleted
                          ? t("lifecycle.status.completed", { defaultValue: "Đã hoàn thành" })
                          : t("lifecycle.status.in_progress", { defaultValue: "Đang tiến hành" })}
                      </span>
                    </td>
                    <td className="px-8 py-5 text-xs text-slate-600 dark:text-slate-300 font-medium">
                      {formatDate(c.started_at)}
                    </td>
                    <td className="px-8 py-5 text-right">
                      <Link
                        className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-bold text-slate-600 dark:text-slate-300 hover:text-primary-600 dark:hover:text-primary-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
                        params={{ employeeId: c.employee_id }}
                        search={{ tab: "lifecycle" }}
                        to="/employees/$employeeId"
                      >
                        <span>{t("lifecycle.view_detail", { defaultValue: "Xem checklist" })}</span>
                        <ArrowRight size={13} />
                      </Link>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    />
  );
}
