import { Plus, Trash2, ArrowUp, ArrowDown, Users } from "lucide-react";
import React, { useEffect, useMemo } from "react";
import { useForm, useFieldArray, Controller, useWatch } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useCreateApprovalWorkflow, useUpdateApprovalWorkflow } from "@/hooks/use-approval-workflows";
import { useWorkspaceRoles, useWorkspaceMembers } from "@/hooks/use-workspace";
import { ApprovalWorkflow } from "@/lib/api/types";
import { permissionGroups } from "@/lib/permission-groups";

const workflowStepSchema = z.object({
  step_order: z.number().min(1),
  step_name: z.string().min(1, "Step name is required"),
  approver_type: z.enum(["department_manager", "role", "permission", "specific_user"]),
  role_id: z.string().optional(),
  permission_name: z.string().optional(),
  user_id: z.string().optional(),
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

interface WorkspaceRoleDef {
  id: string;
  name: string;
}

interface WorkspaceMemberDef {
  email: string;
  full_name?: string;
  user_id: string;
}

export function UpsertWorkflowModal({
  workflow,
  isOpen,
  onClose,
  wsId,
  defaultEntityType = "ot_request",
}: {
  workflow: ApprovalWorkflow | null;
  isOpen: boolean;
  onClose: () => void;
  wsId: string;
  defaultEntityType?: string;
}) {
  const { t } = useTranslation();
  const toast = useToast();
  const { mutateAsync: createWorkflow, isPending: isCreating } = useCreateApprovalWorkflow();
  const { mutateAsync: updateWorkflow, isPending: isUpdating } = useUpdateApprovalWorkflow(workflow?.id ?? "");

  const { roles } = useWorkspaceRoles(wsId);
  const { members } = useWorkspaceMembers(wsId);

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
          role_id: "",
          permission_name: "",
          user_id: "",
          action_required: "approve",
          can_skip: false,
        }
      ],
    },
  });

  const { fields, append, remove, move } = useFieldArray({
    control: form.control,
    name: "steps",
  });

  const watchSteps = useWatch({
    control: form.control,
    name: "steps",
  });

  useEffect(() => {
    if (workflow) {
      form.reset({
        name: workflow.name,
        entity_type: workflow.entity_type,
        description: workflow.description ?? "",
        is_active: workflow.is_active,
        steps: workflow.steps?.map((s) => {
          const config = s.approver_config as { role_id?: string; permission_name?: string; user_id?: string } | undefined;
          return {
            step_order: s.step_order,
            step_name: s.step_name,
            approver_type: s.approver_type,
            role_id: s.approver_type === "role" ? config?.role_id ?? "" : "",
            permission_name: s.approver_type === "permission" ? config?.permission_name ?? "" : "",
            user_id: s.approver_type === "specific_user" ? config?.user_id ?? "" : "",
            action_required: s.action_required,
            can_skip: s.can_skip,
          };
        }) ?? [],
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
        steps: data.steps.map((s, index) => {
          let parsedConfig: Record<string, unknown> = {};
          if (s.approver_type === "specific_user") {
            parsedConfig = { user_id: s.user_id ?? "" };
          } else if (s.approver_type === "role") {
            parsedConfig = { role_id: s.role_id ?? "" };
          } else if (s.approver_type === "permission") {
            parsedConfig = { permission_name: s.permission_name ?? "" };
          }
          return {
            step_order: index + 1,
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

  const permissionOptions = useMemo(() => {
    return permissionGroups.flatMap(group => 
      group.permissions.map(p => ({
        value: p.id,
        label: `${group.name} - ${p.name}`,
      }))
    );
  }, []);

  const roleOptions = useMemo(() => {
    const rolesList = roles as WorkspaceRoleDef[] | undefined;
    return (rolesList ?? []).map(r => ({
      value: r.id,
      label: r.name,
    }));
  }, [roles]);

  const memberOptions = useMemo(() => {
    const membersList = members as WorkspaceMemberDef[] | undefined;
    return (membersList ?? []).map(m => ({
      description: m.email,
      label: m.full_name ?? m.email,
      value: m.user_id,
    }));
  }, [members]);

  return (
    <Modal isOpen={isOpen} maxWidth="xl" onClose={onClose} title={workflow ? t("settings.overtime.workflows.edit", { defaultValue: "Edit Workflow" }) : t("settings.overtime.workflows.new", { defaultValue: "New Workflow" })}>
      <form className="space-y-8" onSubmit={form.handleSubmit(onSubmit)}>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="space-y-1.5">
            <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest" htmlFor="workflow-name">
              {t("settings.overtime.workflows.name_label", { defaultValue: "Name" })}
            </label>
            <input
              className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus-visible:ring-2 focus-visible:ring-blue-500/50"
              id="workflow-name"
              placeholder="e.g. Standard OT Approval"
              {...form.register("name")}
            />
            {!!form.formState.errors.name && (
              <Typography className="text-rose-500 text-xs mt-1" variant="p">
                {form.formState.errors.name.message}
              </Typography>
            )}
          </div>
          <div className="space-y-1.5">
            <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest" htmlFor="workflow-entity">
              {t("settings.overtime.workflows.entity_label", { defaultValue: "Entity Type" })}
            </label>
            <select
              className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus-visible:ring-2 focus-visible:ring-blue-500/50"
              id="workflow-entity"
              {...form.register("entity_type")}
            >
              <option value="ot_request">Overtime Request</option>
              <option value="leave_request">Leave Request</option>
              <option value="attendance_adjustment">Attendance Adjustment</option>
            </select>
          </div>
        </div>

        <div className="space-y-1.5">
          <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest" htmlFor="workflow-description">
            {t("settings.overtime.workflows.desc_label", { defaultValue: "Description" })}
          </label>
          <input
            className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus-visible:ring-2 focus-visible:ring-blue-500/50"
            id="workflow-description"
            placeholder="Optional description"
            {...form.register("description")}
          />
        </div>

        <div className="flex items-center gap-3">
          <input className="w-4 h-4 text-blue-600 rounded border-slate-300 focus:ring-blue-500" id="is_active" type="checkbox" {...form.register("is_active")} />
          <label className="text-sm font-medium text-slate-700 dark:text-slate-300" htmlFor="is_active">
            {t("settings.overtime.workflows.active_label", { defaultValue: "Active Workflow" })}
          </label>
        </div>

        <div className="pt-6 border-t border-slate-100 dark:border-slate-800">
          <div className="flex justify-between items-center mb-6">
            <Typography className="text-sm font-bold text-slate-800 dark:text-slate-200 uppercase tracking-wider" variant="h4">
              {t("settings.overtime.workflows.steps_header", { defaultValue: "Approval Steps" })}
            </Typography>
            <Button
              className="flex items-center gap-1.5 text-xs font-bold text-blue-600 hover:text-blue-700 bg-blue-50 hover:bg-blue-100 dark:bg-blue-900/20 dark:hover:bg-blue-900/30 px-4 py-2 rounded-xl transition-colors"
              onClick={() => { append({
                step_order: fields.length + 1,
                step_name: `Step ${fields.length + 1}`,
                approver_type: "department_manager",
                role_id: "",
                permission_name: "",
                user_id: "",
                action_required: "approve",
                can_skip: false,
              }); }}
            >
              <Plus className="w-4 h-4" />
              {t("settings.overtime.workflows.add_step", { defaultValue: "Add Step" })}
            </Button>
          </div>

          <div className="relative border-l border-slate-200 dark:border-slate-800 ml-4 pl-8 space-y-8">
            {fields.map((field, index) => {
              const approverType = watchSteps[index]?.approver_type ?? "department_manager";

              return (
                <div 
                  className="bg-slate-50/50 dark:bg-slate-900/50 border border-slate-200/60 dark:border-slate-800 rounded-3xl p-5 relative group hover:border-slate-300 dark:hover:border-slate-700 transition-all duration-200" 
                  key={field.id}
                >
                  <div className="absolute -left-[45px] top-6 w-8 h-8 rounded-full bg-blue-600 dark:bg-blue-600 border-4 border-white dark:border-slate-950 flex items-center justify-center text-[10px] font-bold text-white shadow-sm ring-1 ring-slate-100 dark:ring-slate-900 transition-transform group-hover:scale-105">
                    {index + 1}
                  </div>

                  <div className="flex justify-between items-center mb-5 border-b border-slate-100 dark:border-slate-800 pb-3">
                    <div className="flex items-center gap-2">
                      <Typography className="text-sm font-bold text-slate-800 dark:text-slate-200" variant="p">
                        {t("settings.overtime.workflows.step_title", { defaultValue: "Step" })} {index + 1}
                      </Typography>
                    </div>

                    <div className="flex items-center gap-1">
                      <Button
                        className="p-1.5 text-slate-400 hover:text-blue-600 hover:bg-white dark:hover:bg-slate-800 rounded-lg transition-colors disabled:opacity-30 disabled:pointer-events-none"
                        disabled={index === 0}
                        onClick={() => { move(index, index - 1); }}
                        size="icon"
                        title="Move Up"
                        variant="ghost"
                      >
                        <ArrowUp className="w-4 h-4" />
                      </Button>

                      <Button
                        className="p-1.5 text-slate-400 hover:text-blue-600 hover:bg-white dark:hover:bg-slate-800 rounded-lg transition-colors disabled:opacity-30 disabled:pointer-events-none"
                        disabled={index === fields.length - 1}
                        onClick={() => { move(index, index + 1); }}
                        size="icon"
                        title="Move Down"
                        variant="ghost"
                      >
                        <ArrowDown className="w-4 h-4" />
                      </Button>

                      {fields.length > 1 && (
                        <Button
                          className="p-1.5 text-slate-400 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-950/20 rounded-lg transition-colors"
                          onClick={() => { remove(index); }}
                          size="icon"
                          title="Delete Step"
                          variant="ghost"
                        >
                          <Trash2 className="w-4 h-4" />
                        </Button>
                      )}
                    </div>
                  </div>

                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                    <div className="space-y-1">
                      <label className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider" htmlFor={`step-name-${index}`}>Step Name</label>
                      <input
                        className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3.5 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30"
                        id={`step-name-${index}`}
                        {...form.register(`steps.${index}.step_name` as const)}
                      />
                    </div>

                    <div className="space-y-1">
                      <label className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider" htmlFor={`step-approver-${index}`}>Approver Type</label>
                      <select
                        className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3.5 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30"
                        id={`step-approver-${index}`}
                        {...form.register(`steps.${index}.approver_type` as const)}
                      >
                        <option value="department_manager">Dept Manager</option>
                        <option value="role">Role</option>
                        <option value="permission">Permission</option>
                        <option value="specific_user">Specific User</option>
                      </select>
                    </div>

                    <div className="space-y-1">
                      <label className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider" htmlFor={`step-action-${index}`}>Action</label>
                      <select
                        className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3.5 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30"
                        id={`step-action-${index}`}
                        {...form.register(`steps.${index}.action_required` as const)}
                      >
                        <option value="approve">Approve</option>
                        <option value="review">Review</option>
                        <option value="acknowledge">Acknowledge</option>
                      </select>
                    </div>

                    <div className="flex items-center gap-2 pt-5">
                      <input 
                        className="w-4 h-4 rounded border-slate-300 text-blue-600 focus:ring-blue-500" 
                        id={`skip-${index}`} 
                        type="checkbox" 
                        {...form.register(`steps.${index}.can_skip` as const)} 
                      />
                      <label className="text-xs font-semibold text-slate-600 dark:text-slate-400" htmlFor={`skip-${index}`}>Can Skip if Not Found</label>
                    </div>
                  </div>

                  <div className="mt-5 pt-4 border-t border-slate-200/50 dark:border-slate-800/50">
                    {approverType === "department_manager" && (
                      <div className="flex items-center gap-3 bg-blue-50/50 dark:bg-blue-900/10 border border-blue-100/50 dark:border-blue-900/20 rounded-2xl p-4 text-xs text-blue-700 dark:text-blue-400">
                        <Users className="w-5 h-5 shrink-0 text-blue-500" />
                        <div>
                          <Typography className="font-bold" variant="p">Department Manager Routing</Typography>
                          <Typography className="mt-0.5 opacity-90" variant="p">
                            Requests will be sent automatically to the manager of the requester&apos;s department.
                          </Typography>
                        </div>
                      </div>
                    )}

                    {approverType === "role" && (
                      <div className="space-y-1 max-w-md">
                        <label className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider" htmlFor={`step-role-${index}`}>Select Role</label>
                        <select
                          className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3.5 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30"
                          id={`step-role-${index}`}
                          {...form.register(`steps.${index}.role_id` as const)}
                        >
                          <option value="">Choose a Role...</option>
                          {roleOptions.map(opt => (
                            <option key={opt.value} value={opt.value}>{opt.label}</option>
                          ))}
                        </select>
                      </div>
                    )}

                    {approverType === "permission" && (
                      <div className="space-y-1 max-w-md">
                        <label className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider" htmlFor={`step-permission-${index}`}>Select Permission</label>
                        <select
                          className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3.5 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30"
                          id={`step-permission-${index}`}
                          {...form.register(`steps.${index}.permission_name` as const)}
                        >
                          <option value="">Choose a Permission...</option>
                          {permissionOptions.map(opt => (
                            <option key={opt.value} value={opt.value}>{opt.label}</option>
                          ))}
                        </select>
                      </div>
                    )}

                    {approverType === "specific_user" && (
                      <div className="space-y-1.5 max-w-md">
                        <label 
                          className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider"
                          htmlFor={`step-specific-user-${index}`}
                        >
                          Search Workspace User
                        </label>
                        <Controller
                          control={form.control}
                          name={`steps.${index}.user_id` as const}
                          render={({ field }) => (
                            <Combobox
                              emptyText="No members found"
                              id={`step-specific-user-${index}`}
                              onChange={field.onChange}
                              options={memberOptions}
                              placeholder="Type to search user..."
                              searchPlaceholder="Search members..."
                              value={field.value}
                            />
                          )}
                        />
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        <div className="flex justify-end gap-3 pt-6 border-t border-slate-100 dark:border-slate-800">
          <Button
            className="px-5 py-2.5 rounded-xl font-bold text-sm text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
            onClick={onClose}
            variant="ghost"
          >
            {t("common.cancel")}
          </Button>
          <Button
            className="px-5 py-2.5 rounded-xl font-bold text-sm bg-blue-600 hover:bg-blue-700 text-white shadow-sm transition-all disabled:opacity-50"
            disabled={isPending}
            type="submit"
            variant="primary"
          >
            {isPending ? t("common.saving") : t("common.save")}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
