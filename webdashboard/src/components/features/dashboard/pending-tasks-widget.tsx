import { ClipboardList, Circle, AlertCircle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { PendingTaskResponse } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useMyPendingTasks, useCompletePendingTask } from "@/hooks/use-lifecycle";
import { cn } from "@/lib/utils";
import { maybeNull } from "@/types/api-overrides";

// ── PendingTaskItem ──────────────────────────────────────────────────────

interface PendingTaskItemProps {
  isMutating: boolean;
  onComplete: (task: PendingTaskResponse) => void;
  task: PendingTaskResponse;
}

function PendingTaskItem({ task, isMutating, onComplete }: PendingTaskItemProps) {
  const { t } = useTranslation();
  const { formatDate } = useDateTimeFormatter();
  const isOverdue = task.due_date && new Date(task.due_date) < new Date();

  return (
    <div
      className="p-4 rounded-2xl bg-slate-50/50 dark:bg-slate-950/40 border border-slate-100 dark:border-slate-800/50 flex items-start justify-between gap-4 hover:border-slate-200 dark:hover:border-slate-800 hover:shadow-sm transition-all"
      key={task.id}
    >
      <div className="flex items-start gap-3 flex-1">
        <Button
          className="text-slate-400 hover:text-emerald-500 transition-colors p-0 h-auto min-w-0"
          loading={isMutating ? true : undefined}
          onClick={() => { onComplete(task); }}
          variant="ghost"
        >
          <Circle className="w-5 h-5" />
        </Button>

        <div className="space-y-1">
          <div className="flex items-center gap-2 flex-wrap">
            <span className="font-semibold text-sm text-slate-800 dark:text-slate-200">
              {task.title}
            </span>
            <span className="bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400 text-[9px] px-2 py-0.5 rounded-full font-bold uppercase tracking-wider">
              {task.category.replace("_", " ")}
            </span>
            {task.is_required ? (
              <span className="bg-rose-50 dark:bg-rose-950/20 text-rose-600 dark:text-rose-400 text-[9px] px-2 py-0.5 rounded-full font-bold uppercase tracking-wider">
                Required
              </span>
            ) : null}
          </div>

          <Typography className="text-xs text-slate-400 dark:text-slate-500" variant="p">
            {t("dashboard.lifecycle.task_for_employee", { name: task.employee_name, defaultValue: `For ${task.employee_name}` })}
            {!!task.description && ` • ${task.description}`}
          </Typography>

           {!!task.due_date && (
            <div className="text-[10px] font-semibold flex items-center gap-1 mt-1">
              <AlertCircle className={cn("w-3.5 h-3.5", isOverdue ? "text-rose-500 animate-pulse" : "text-amber-500")} />
              <span className={cn(isOverdue ? "text-rose-500" : "text-slate-400")}>
                {isOverdue
                  ? t("dashboard.lifecycle.overdue", { date: formatDate(task.due_date), defaultValue: `Overdue: ${formatDate(task.due_date)}` })
                  : t("dashboard.lifecycle.due", { date: formatDate(task.due_date), defaultValue: `Due: ${formatDate(task.due_date)}` })}
              </span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// ── PendingTasksWidget ───────────────────────────────────────────────────

export default function PendingTasksWidget() {
  const { t } = useTranslation();
  const toast = useToast();

  const { data: tasks = [], isLoading, error } = useMyPendingTasks();
  const completeTaskMutation = useCompletePendingTask();

  const handleCompleteTask = (task: PendingTaskResponse) => {
    completeTaskMutation.mutate(
      {
        checklistId: task.checklist_id,
        itemId: task.id,
        payload: {
          status: "completed",
        },
      },
      {
        onSuccess: () => {
          toast.success(t("dashboard.lifecycle.task_completed_success", "Task marked as completed"));
        },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : t("dashboard.lifecycle.task_completed_failed", "Failed to complete task"));
        },
      }
    );
  };

  if (isLoading) {
    return (
      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2rem] p-6 shadow-sm animate-pulse space-y-4">
        <div className="h-5 w-40 bg-slate-100 dark:bg-slate-800 rounded-md" />
        <div className="space-y-3">
          <div className="h-12 w-full bg-slate-50 dark:bg-slate-800/50 rounded-xl" />
          <div className="h-12 w-full bg-slate-50 dark:bg-slate-800/50 rounded-xl" />
        </div>
      </div>
    );
  }

  if (error || tasks.length === 0) {
    return null;
  }

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2rem] p-6 shadow-sm space-y-4 animate-in fade-in duration-500">
      <div className="flex items-center gap-2 pb-2 border-b border-slate-50 dark:border-slate-800/60">
        <ClipboardList className="w-5 h-5 text-primary-500" />
        <Typography className="text-sm font-bold text-slate-800 dark:text-slate-200" variant="h3">
          {t("dashboard.lifecycle.pending_tasks_title", "YOUR PENDING TASKS")}
        </Typography>
        <span className="bg-rose-50 dark:bg-rose-950/20 text-rose-600 dark:text-rose-400 text-[10px] px-2 py-0.5 rounded-full font-bold">
          {tasks.length}
        </span>
      </div>

      <div className="space-y-3 max-h-[300px] overflow-y-auto pr-1">
        {tasks.map((task) => {
          const isMutating = completeTaskMutation.isPending && maybeNull(completeTaskMutation.variables)?.itemId === task.id;
          return (
            <PendingTaskItem
              isMutating={isMutating}
              key={task.id}
              onComplete={handleCompleteTask}
              task={task}
            />
          );
        })}
      </div>
    </div>
  );
}
