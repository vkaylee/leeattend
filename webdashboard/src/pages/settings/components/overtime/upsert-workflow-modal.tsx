import { Plus, Trash2 } from "lucide-react";
import React, { useEffect } from "react";
import { useForm, useFieldArray } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";

import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { useCreateApprovalWorkflow, useUpdateApprovalWorkflow } from "@/hooks/use-approval-workflows";

const workflowStepSchema = z.object({
  step_order: z.number().min(1),
  step_name: z.string().min(1, "Step name is required"),
  approver_type: z.enum(["department_manager", "role", "permission", "specific_user"]),
  approver_config_str: z.string().optional(),
  action_required: z.enum(["approve", "review", "acknowledge"]),
  can_skip: z.boolean(),
});

const workflowSchema = z.object({
  name: z.string().min(1, "Name is required"),
  entity_type: z.string().min(1, "Entity type is required"),
  description: z.string().optional(),
  is_active: z.boolean(),
  steps: z.array(workflowStepSchema).min(1, "At least one step is required"),
});

type WorkflowFormValues = z.infer<typeof workflowSchema>;

export function UpsertWorkflowModal({
  workflow,
  isOpen,
  onClose,
  wsId,
  defaultEntityType = "ot_request",
}: {
  workflow: any | null;
  isOpen: boolean;
  onClose: () => void;
  wsId: string;
  defaultEntityType?: string;
}) {
  const { t } = useTranslation();
  const toast = useToast();
  const { mutateAsync: createWorkflow, isPending: isCreating } = useCreateApprovalWorkflow();
  const { mutateAsync: updateWorkflow, isPending: isUpdating } = useUpdateApprovalWorkflow(workflow?.id || "");

  const form = useForm<WorkflowFormValues>({
    resolver: zodResolver(workflowSchema),
    defaultValues: {
      name: "",
      entity_type: defaultEntityType,
      description: "",
      is_active: true,
      steps: [
        {
          step_order: 1,
          step_name: "L1 Approval",
          approver_type: "department_manager",
          approver_config_str: "{}",
          action_required: "approve",
          can_skip: false,
        }
      ],
    },
  });

  const { fields, append, remove } = useFieldArray({
    control: form.control,
    name: "steps",
  });

  useEffect(() => {
    if (workflow) {
      form.reset({
        name: workflow.name,
        entity_type: workflow.entity_type,
        description: workflow.description || "",
        is_active: workflow.is_active,
        steps: workflow.steps?.map((s: any) => ({
          step_order: s.step_order,
          step_name: s.step_name,
          approver_type: s.approver_type,
          approver_config_str: JSON.stringify(s.approver_config),
          action_required: s.action_required,
          can_skip: s.can_skip,
        })) || [],
      });
    }
  }, [workflow, form]);

  const onSubmit = async (data: WorkflowFormValues) => {
    try {
      const payload = {
        name: data.name,
        entity_type: data.entity_type,
        description: data.description,
        is_active: data.is_active,
        steps: data.steps.map(s => {
          let parsedConfig = {};
          if (s.approver_config_str) {
            try {
              parsedConfig = JSON.parse(s.approver_config_str);
            } catch (e) {}
          }
          return {
            step_order: s.step_order,
            step_name: s.step_name,
            approver_type: s.approver_type,
            approver_config: parsedConfig,
            action_required: s.action_required,
            can_skip: s.can_skip,
          };
        }),
      };

      if (workflow) {
        await updateWorkflow(payload);
        toast.success("Workflow updated");
      } else {
        await createWorkflow(payload);
        toast.success("Workflow created");
      }
      onClose();
    } catch (error) {
      console.error(error);
      toast.error("Failed to save workflow");
    }
  };

  const isPending = isCreating || isUpdating;

  return (
    <Modal isOpen={isOpen} onClose={onClose} maxWidth="xl" title={workflow ? "Edit Workflow" : "New Workflow"}>
      <form className="space-y-8" onSubmit={form.handleSubmit(onSubmit)}>
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-1">
            <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest">Name</label>
            <input
              {...form.register("name")}
              className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
              placeholder="e.g. Standard OT Approval"
            />
            {!!form.formState.errors.name && <p className="text-rose-500 text-xs mt-1">{form.formState.errors.name.message}</p>}
          </div>
          <div className="space-y-1">
            <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest">Entity Type</label>
            <select
              {...form.register("entity_type")}
              className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
            >
              <option value="ot_request">Overtime Request</option>
              <option value="leave_request">Leave Request</option>
              <option value="attendance_adjustment">Attendance Adjustment</option>
            </select>
          </div>
        </div>

        <div className="space-y-1">
          <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest">Description</label>
          <input
            {...form.register("description")}
            className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
            placeholder="Optional description"
          />
        </div>

        <div className="flex items-center gap-2">
          <input id="is_active" type="checkbox" {...form.register("is_active")} className="w-4 h-4 text-blue-600 rounded border-slate-300" />
          <label className="text-sm font-medium text-slate-700 dark:text-slate-300" htmlFor="is_active">Active Workflow</label>
        </div>

        <div className="pt-4 border-t border-slate-100 dark:border-slate-800">
          <div className="flex justify-between items-center mb-4">
            <h4 className="text-sm font-bold text-slate-800 dark:text-slate-200">Approval Steps</h4>
            <button
              className="flex items-center gap-1 text-xs font-bold text-blue-600 hover:text-blue-700 bg-blue-50 hover:bg-blue-100 px-3 py-1.5 rounded-lg transition-colors"
              onClick={() => { append({
                step_order: fields.length + 1,
                step_name: `Step ${fields.length + 1}`,
                approver_type: "department_manager",
                approver_config_str: "{}",
                action_required: "approve",
                can_skip: false,
              }); }}
              type="button"
            >
              <Plus className="w-3 h-3" />
              Add Step
            </button>
          </div>

          <div className="space-y-4">
            {fields.map((field, index) => (
              <div className="bg-slate-50 dark:bg-slate-900/50 border border-slate-200 dark:border-slate-800 rounded-2xl p-4 relative" key={field.id}>
                <button
                  className="absolute top-4 right-4 text-slate-400 hover:text-rose-500 transition-colors"
                  onClick={() => { remove(index); }}
                  type="button"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
                
                <div className="flex items-center gap-2 mb-4">
                  <div className="w-6 h-6 rounded-full bg-blue-100 text-blue-700 flex items-center justify-center text-xs font-bold">
                    {index + 1}
                  </div>
                  <h5 className="text-sm font-bold text-slate-700">Step {index + 1}</h5>
                  <input type="hidden" {...form.register(`steps.${index}.step_order` as const, { valueAsNumber: true })} value={index + 1} />
                </div>

                <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                  <div className="space-y-1">
                    <label className="block text-[10px] font-bold text-slate-500 uppercase">Step Name</label>
                    <input
                      {...form.register(`steps.${index}.step_name` as const)}
                      className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
                    />
                  </div>
                  <div className="space-y-1">
                    <label className="block text-[10px] font-bold text-slate-500 uppercase">Approver Type</label>
                    <select
                      {...form.register(`steps.${index}.approver_type` as const)}
                      className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
                    >
                      <option value="department_manager">Dept Manager</option>
                      <option value="role">Role</option>
                      <option value="permission">Permission</option>
                      <option value="specific_user">Specific User</option>
                    </select>
                  </div>
                  <div className="space-y-1">
                    <label className="block text-[10px] font-bold text-slate-500 uppercase">Action</label>
                    <select
                      {...form.register(`steps.${index}.action_required` as const)}
                      className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
                    >
                      <option value="approve">Approve</option>
                      <option value="review">Review</option>
                      <option value="acknowledge">Acknowledge</option>
                    </select>
                  </div>
                  <div className="flex items-center gap-2 pt-6">
                    <input id={`skip-${index}`} type="checkbox" {...form.register(`steps.${index}.can_skip` as const)} className="w-4 h-4" />
                    <label className="text-xs font-medium text-slate-600" htmlFor={`skip-${index}`}>Can Skip if Not Found</label>
                  </div>
                </div>
                
                <div className="mt-4 space-y-1">
                  <label className="block text-[10px] font-bold text-slate-500 uppercase">Approver Config (JSON)</label>
                  <input
                    {...form.register(`steps.${index}.approver_config_str` as const)}
                    className="w-full font-mono bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
                    placeholder='{"role_name": "hr_manager"}'
                  />
                  <p className="text-[10px] text-slate-400">Example: {`{"role_name": "hr_manager"}`} or {`{"permission": "overtime:approve"}`}</p>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="flex justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800">
          <button
            className="px-5 py-2.5 rounded-xl font-bold text-sm text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
            onClick={onClose}
            type="button"
          >
            {t("common.cancel")}
          </button>
          <button
            className="px-5 py-2.5 rounded-xl font-bold text-sm bg-blue-600 hover:bg-blue-700 text-white shadow-sm transition-all disabled:opacity-50"
            disabled={isPending}
            type="submit"
          >
            {isPending ? t("common.saving") : t("common.save")}
          </button>
        </div>
      </form>
    </Modal>
  );
}
