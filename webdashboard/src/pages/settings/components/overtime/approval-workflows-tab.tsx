import { Loader2, Plus, Edit2, Trash2, ArrowRight } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { useApprovalWorkflows, useDeleteApprovalWorkflow } from "@/hooks/use-approval-workflows";
import { useConfirm } from "@/hooks/use-confirm";

import { UpsertWorkflowModal } from "./upsert-workflow-modal";

export function ApprovalWorkflowsTab({ wsId, entityType = "ot_request" }: { wsId: string; entityType?: string }) {
  const { t } = useTranslation();
  const { data: workflows, isLoading } = useApprovalWorkflows(entityType);
  const { mutate: deleteWorkflow, isPending: isDeleting } = useDeleteApprovalWorkflow();
  const { confirm } = useConfirm();

  const [upsertModalOpen, setUpsertModalOpen] = useState(false);
  const [selectedWorkflow, setSelectedWorkflow] = useState<any | null>(null);

  const handleEdit = (workflow: any) => {
    setSelectedWorkflow(workflow);
    setUpsertModalOpen(true);
  };

  const handleCreate = () => {
    setSelectedWorkflow(null);
    setUpsertModalOpen(true);
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
      <div className="flex justify-between items-center border-b border-slate-50 dark:border-slate-800 pb-6">
        <div>
          <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase" variant="h3">
            {t("settings.overtime.tabs.workflows", { defaultValue: "Approval Workflows" })}
          </Typography>
          <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 mt-1 uppercase tracking-tight" variant="p">
            {t("settings.overtime.workflows_desc", { defaultValue: "Define multi-step approval processes for requests." })}
          </Typography>
        </div>
        <button
          className="flex items-center gap-2 bg-blue-600 hover:bg-blue-700 text-white px-5 py-2.5 rounded-2xl font-bold text-sm tracking-tight transition-all shadow-sm"
          onClick={handleCreate}
        >
          <Plus className="w-4 h-4" />
          {t("common.add_new")}
        </button>
      </div>

      {isLoading ? (
        <div className="flex justify-center p-12">
          <Loader2 className="w-8 h-8 animate-spin text-slate-400" />
        </div>
      ) : !workflows || workflows.length === 0 ? (
        <div className="text-center py-16 border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-3xl">
          <Typography className="text-slate-500 font-medium">
            {t("common.no_data")}
          </Typography>
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-4">
          {workflows.map((wf) => (
            <div className="bg-slate-50 dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 p-6 flex flex-col md:flex-row gap-6 justify-between items-start md:items-center" key={wf.id}>
              <div className="flex-1 space-y-3">
                <div className="flex items-center gap-3">
                  <span className={`px-2 py-1 rounded-md text-[10px] font-bold uppercase tracking-widest ${
                    wf.is_active 
                      ? "bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400" 
                      : "bg-slate-200 text-slate-600 dark:bg-slate-800 dark:text-slate-400"
                  }`}>
                    {wf.is_active ? t("common.active") : t("common.inactive")}
                  </span>
                  <Typography className="font-bold text-lg text-slate-800 dark:text-slate-100">
                    {wf.name}
                  </Typography>
                  <span className="font-mono text-xs bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 px-2 py-1 rounded-md text-slate-500">
                    {wf.entity_type}
                  </span>
                </div>
                {!!wf.description && (
                  <Typography className="text-sm text-slate-500 dark:text-slate-400">
                    {wf.description}
                  </Typography>
                )}
                
                {/* Workflow Steps Preview */}
                {!!wf.steps && wf.steps.length > 0 && (
                  <div className="flex items-center gap-2 pt-2 overflow-x-auto custom-scrollbar pb-2">
                    {wf.steps.map((step: any, idx: any) => (
                      <React.Fragment key={step.id}>
                        <div className="flex items-center gap-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl px-3 py-2 shrink-0">
                          <span className="w-5 h-5 rounded-full bg-slate-100 dark:bg-slate-900 flex items-center justify-center text-[10px] font-bold text-slate-500">
                            {step.step_order}
                          </span>
                          <span className="text-xs font-bold text-slate-700 dark:text-slate-300">
                            {step.step_name}
                          </span>
                        </div>
                        {idx < wf.steps!.length - 1 && (
                          <ArrowRight className="w-4 h-4 text-slate-300 dark:text-slate-600 shrink-0" />
                        )}
                      </React.Fragment>
                    ))}
                  </div>
                )}
              </div>
              
              <div className="flex gap-2 shrink-0">
                <button
                  className="p-2 text-slate-400 hover:text-blue-600 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl transition-colors"
                  onClick={() => { handleEdit(wf); }}
                >
                  <Edit2 className="w-4 h-4" />
                </button>
                <button
                  className="p-2 text-slate-400 hover:text-rose-600 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl transition-colors"
                  disabled={isDeleting}
                  onClick={() => handleDelete(wf.id)}
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {!!upsertModalOpen && (
        <UpsertWorkflowModal
          isOpen={upsertModalOpen}
          onClose={() => { setUpsertModalOpen(false); }}
          workflow={selectedWorkflow}
          wsId={wsId}
          defaultEntityType={entityType}
        />
      )}
    </div>
  );
}
