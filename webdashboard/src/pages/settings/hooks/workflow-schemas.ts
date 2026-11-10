import { z } from "zod";

export const workflowStepSchema = z.object({
  step_order: z.number().min(1),
  step_name: z.string().min(1, "settings.workflows.errors.step_name_required"),
  approver_type: z.enum(["department_manager", "role", "permission", "specific_user"]),
  role_id: z.string().optional(),
  permission_name: z.string().optional(),
  user_id: z.string().optional(),
  action_required: z.enum(["approve", "review", "acknowledge"]),
  can_skip: z.boolean(),
});

export const workflowSchema = z.object({
  name: z.string().min(1, "settings.workflows.errors.name_required"),
  entity_type: z.string().min(1, "settings.workflows.errors.entity_type_required"),
  description: z.string().optional(),
  is_active: z.boolean(),
  steps: z.array(workflowStepSchema).min(1, "settings.workflows.errors.steps_required"),
});

export type WorkflowFormValues = z.infer<typeof workflowSchema>;

export interface OrganizationRoleDef {
  id: string;
  name: string;
}

export interface OrganizationMemberDef {
  email: string;
  full_name?: string;
  user_id: string;
}

export const createDefaultFormValues = (entityType: string, t?: (key: string, options?: Record<string, unknown>) => string): WorkflowFormValues => ({
  name: "",
  entity_type: entityType,
  description: "",
  is_active: true,
  steps: [
    {
      step_order: 1,
      step_name: t ? t("settings.workflows.step_default_name", { index: 1, defaultValue: "L1 Approval" }) : "L1 Approval",
      approver_type: "department_manager",
      role_id: "",
      permission_name: "",
      user_id: "",
      action_required: "approve",
      can_skip: false,
    },
  ],
});

export const formatWorkflowPayload = (data: WorkflowFormValues) => ({
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
});
