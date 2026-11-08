import { useNavigate } from "@tanstack/react-router";
import { Loader2, Plus, Edit2, Trash2, ArrowRight, Workflow } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { ApprovalWorkflow, ApprovalWorkflowStep } from "@/lib/api/types";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import { Typography } from "@/components/ui/typography";
import { useApprovalWorkflows, useDeleteApprovalWorkflow } from "@/hooks/use-approval-workflows";
import { useConfirm } from "@/hooks/use-confirm";
interface WorkflowCardProps {
  isDeleting: boolean;
  onDelete: (id: string) => void;
  onEdit: (workflow: ApprovalWorkflow) => void;
  workflow: ApprovalWorkflow;
}

function WorkflowCard({ workflow, isDeleting, onEdit, onDelete }: WorkflowCardProps) {
  const { t } = useTranslation();

  return (
    <div className="bg-slate-50 dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 p-6 flex flex-col md:flex-row gap-6 justify-between items-start md:items-center">
      <div className="flex-1 space-y-3">
        <div className="flex items-center gap-3">
          <span className={`px-2 py-1 rounded-md text-[10px] font-bold uppercase tracking-widest ${
            workflow.is_active 
              ? "bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400" 
              : "bg-slate-200 text-slate-600 dark:bg-slate-800 dark:text-slate-400"
          }`}>
            {workflow.is_active ? t("common.active") : t("common.inactive")}
          </span>
          <Typography className="font-bold text-lg text-slate-800 dark:text-slate-100">
            {workflow.name}
          </Typography>
          <span className="font-mono text-xs bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 px-2 py-1 rounded-md text-slate-500">
            {workflow.entity_type}
          </span>
        </div>
        {!!workflow.description && (
          <Typography className="text-sm text-slate-500 dark:text-slate-400">
            {workflow.description}
          </Typography>
        )}
        
        {/* Workflow Steps Preview */}
        {workflow.steps && workflow.steps.length > 0 ? (
          <div className="flex items-center gap-2 pt-2 overflow-x-auto custom-scrollbar pb-2">
            {workflow.steps.map((step: ApprovalWorkflowStep, idx: number, arr: ApprovalWorkflowStep[]) => (
              <React.Fragment key={step.id}>
                <div className="flex items-center gap-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3 py-2 shrink-0">
                  <span className="w-5 h-5 rounded-full bg-slate-100 dark:bg-slate-900 flex items-center justify-center text-[10px] font-bold text-slate-500">
                    {step.step_order}
                  </span>
                  <span className="text-xs font-bold text-slate-700 dark:text-slate-300">
                    {step.step_name}
                  </span>
                </div>
                {idx < arr.length - 1 ? (
                  <ArrowRight className="w-4 h-4 text-slate-300 dark:text-slate-600 shrink-0" />
                ) : null}
              </React.Fragment>
            ))}
          </div>
        ) : null}
      </div>
      
      <div className="flex gap-2 shrink-0">
        <Button
          aria-label={t("common.edit", "Chỉnh sửa")}
          className="text-slate-400 hover:text-blue-600"
          onClick={() => { onEdit(workflow); }}
          size="icon"
          variant="outline"
        >
          <Edit2 className="w-4 h-4" />
        </Button>
        <Button
          aria-label={t("common.delete", "Xóa")}
          className="text-slate-400 hover:text-rose-600"
          disabled={isDeleting}
          onClick={() => { onDelete(workflow.id); }}
          size="icon"
          variant="outline"
        >
          <Trash2 className="w-4 h-4" />
        </Button>
      </div>
    </div>
  );
}

export function ApprovalWorkflowsTab({ entityType = "ot_request" }: { entityType?: string }) {
  const { t } = useTranslation();
  const { data: workflows, isLoading } = useApprovalWorkflows(entityType);
  const { mutate: deleteWorkflow, isPending: isDeleting } = useDeleteApprovalWorkflow();
  const { confirm } = useConfirm();
  const navigate = useNavigate();

  const currentTab =
    entityType === "leave_request"
      ? "leave"
      : entityType === "attendance_adjustment"
        ? "attendance"
        : "overtime";

  const handleEdit = (workflow: ApprovalWorkflow) => {
    void navigate({
      to: `/settings/workflows/${workflow.id}/edit` as never,
      search: { tab: currentTab } as never,
    });
  };

  const handleCreate = () => {
    void navigate({
      to: "/settings/workflows/new" as never,
      search: { tab: currentTab } as never,
    });
  };

  const handleDelete = async (id: string) => {
    const isConfirmed = await confirm({
      title: t("common.delete_confirm"),
      message: t("settings.overtime.delete_workflow_desc", { defaultValue: "Are you sure you want to delete this approval workflow? Pending requests may be affected." }),
      confirmLabel: t("common.delete"),
      cancelLabel: t("common.cancel"),
      variant: "danger"
    });

    if (isConfirmed) {
      deleteWorkflow(id);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center pb-4 border-b border-slate-100 dark:border-slate-800">
        <div>
          <Typography className="text-base font-bold text-slate-800 dark:text-slate-100 tracking-tight" variant="h3">
            {t("settings.overtime.tabs.workflows", { defaultValue: "Danh sách quy trình phê duyệt" })}
          </Typography>
          <Typography className="text-xs font-medium text-slate-500 dark:text-slate-400 mt-0.5" variant="p">
            {t("settings.overtime.workflows_desc", { defaultValue: "Cấu hình luồng duyệt nhiều cấp cho đơn từ và yêu cầu." })}
          </Typography>
        </div>
        <Button
          className="rounded-xl px-4 py-2 text-xs font-semibold shadow-xs normal-case tracking-normal"
          icon={<Plus className="w-4 h-4" />}
          onClick={handleCreate}
        >
          {t("common.add_new", "Thêm quy trình")}
        </Button>
      </div>

      {isLoading ? (
        <div className="flex justify-center p-12">
          <Loader2 className="w-8 h-8 animate-spin text-slate-400" />
        </div>
      ) : !workflows || workflows.length === 0 ? (
        <EmptyState
          action={
            <Button
              className="rounded-xl px-4 py-2 text-xs font-semibold shadow-xs normal-case tracking-normal"
              icon={<Plus className="w-4 h-4" />}
              onClick={handleCreate}
              variant="primary"
            >
              {t("common.add_new", "Thêm quy trình")}
            </Button>
          }
          description={t("settings.overtime.no_workflows_desc", { defaultValue: "Chưa có quy trình phê duyệt nào được cấu hình cho danh mục này." })}
          icon={<Workflow className="w-8 h-8" />}
          title={t("settings.overtime.no_workflows", { defaultValue: "Chưa có quy trình phê duyệt" })}
        />
      ) : (
        <div className="grid grid-cols-1 gap-4">
          {workflows.map((wf: ApprovalWorkflow) => (
            <WorkflowCard
              isDeleting={isDeleting}
              key={wf.id}
              onDelete={handleDelete}
              onEdit={handleEdit}
              workflow={wf}
            />
          ))}
        </div>
      )}
    </div>
  );
}

