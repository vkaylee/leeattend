import { CheckCircle2, Circle, AlertCircle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { PendingTaskResponse } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useMyPendingTasks, useCompletePendingTask } from "@/hooks/use-lifecycle";
import { cn } from "@/lib/utils";
import { maybeNull } from "@/types/api-overrides";

export function DesktopPendingTasksWidget() {
  const { t } = useTranslation();
  const toast = useToast();
  const { formatDate } = useDateTimeFormatter();

  const { data: tasks = [], isLoading, error } = useMyPendingTasks();
  const completeTaskMutation = useCompletePendingTask();

  const handleCompleteTask = (task: PendingTaskResponse) => {
    completeTaskMutation.mutate(
      {
        checklistId: task.checklist_id,
        itemId: task.id,
        payload: { status: "completed" },
      },
      {
        onSuccess: () => {
          toast.success(
            t("dashboard.lifecycle.task_completed_success", "Task marked as completed"),
          );
        },
        onError: () => {
          toast.error(
            t("dashboard.lifecycle.task_completed_error", "Failed to update task status"),
          );
        },
      },
    );
  };

  if (isLoading) {
    return (
      <div className="space-y-2">
        <Skeleton className="h-12 w-full rounded-xl" />
        <Skeleton className="h-12 w-full rounded-xl" />
      </div>
    );
  }

  if (error || tasks.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-5 px-4 text-center">
        <div className="w-10 h-10 rounded-2xl bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 flex items-center justify-center mb-2">
          <CheckCircle2 className="w-5 h-5" />
        </div>
        <Typography className="text-xs font-bold text-slate-800 dark:text-slate-100" variant="p">
          {t("desktop.noPendingTasks", "Không có công việc tồn đọng")}
        </Typography>
        <Typography className="text-[11px] text-slate-500 dark:text-slate-400 mt-0.5 max-w-sm leading-relaxed" variant="p">
          {t(
            "desktop.noPendingTasksDesc",
            "Bạn đã hoàn thành tất cả thủ tục và quy trình nhân sự cần giải quyết.",
          )}
        </Typography>
      </div>
    );
  }

  return (
    <div className="space-y-2 max-h-[260px] overflow-y-auto pr-1">
      {tasks.map((task) => {
        const isMutating =
          completeTaskMutation.isPending &&
          maybeNull(completeTaskMutation.variables)?.itemId === task.id;
        const isOverdue = task.due_date && new Date(task.due_date) < new Date();
        return (
          <div
            className="p-3 rounded-xl bg-slate-50/60 dark:bg-slate-900/40 border border-slate-100 dark:border-slate-800/60 flex items-start justify-between gap-3 text-xs"
            key={task.id}
          >
            <div className="flex items-start gap-2.5 flex-1 min-w-0">
              <Button
                aria-label={`Complete task: ${task.title}`}
                className="text-slate-400 hover:text-emerald-500 transition-colors p-0 h-auto min-w-0 mt-0.5 focus-visible:ring-2 focus-visible:ring-primary"
                loading={isMutating ? true : undefined}
                onClick={() => { handleCompleteTask(task); }}
                type="button"
                variant="ghost"
              >
                <Circle className="w-4 h-4" />
              </Button>
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5 flex-wrap">
                  <span className="font-semibold text-slate-800 dark:text-slate-200 truncate">
                    {task.title}
                  </span>
                  <span className="bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400 text-[9px] px-1.5 py-0.5 rounded font-medium uppercase">
                    {task.category.replace("_", " ")}
                  </span>
                </div>
                {!!task.description && (
                  <Typography className="text-[11px] text-slate-500 dark:text-slate-400 truncate mt-0.5" variant="p">
                    {task.description}
                  </Typography>
                )}
                {!!task.due_date && (
                  <div className="text-[10px] font-semibold flex items-center gap-1 mt-1">
                    <AlertCircle
                      className={cn(
                        "w-3 h-3",
                        isOverdue ? "text-rose-500" : "text-amber-500",
                      )}
                    />
                    <span className={cn(isOverdue ? "text-rose-500" : "text-slate-500")}>
                      {isOverdue
                        ? t("dashboard.lifecycle.overdue", {
                            date: formatDate(task.due_date),
                            defaultValue: `Quá hạn: ${formatDate(task.due_date)}`,
                          })
                        : t("dashboard.lifecycle.due", {
                            date: formatDate(task.due_date),
                            defaultValue: `Hạn: ${formatDate(task.due_date)}`,
                          })}
                    </span>
                  </div>
                )}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
