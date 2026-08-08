import { useNavigate, useParams, useSearch } from "@tanstack/react-router";
import { useEffect, useMemo, useCallback } from "react";
import { useFieldArray, useForm, useWatch } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { zodResolver } from "@hookform/resolvers/zod";

import { useToast } from "@/components/ui/toast";
import {
  useCreateApprovalWorkflow,
  useUpdateApprovalWorkflow,
  useApprovalWorkflow,
} from "@/hooks/use-approval-workflows";
import { usePermissionCatalog, useWorkspace, useWorkspaceRoles, useWorkspaceMembers } from "@/hooks/use-workspace";
import { logError } from "@/lib/logger";
import { ROUTES } from "@/lib/routes";

import {
  workflowSchema,
  WorkflowFormValues,
  WorkspaceRoleDef,
  WorkspaceMemberDef,
  createDefaultFormValues,
  formatWorkflowPayload,
} from "./workflow-schemas";

export function useWorkflowOptions(workspaceId: string) {
  const { t } = useTranslation();
  const { roles } = useWorkspaceRoles(workspaceId);
  const { members } = useWorkspaceMembers(workspaceId);
  const { catalog, isError: isCatalogError, isLoading: isCatalogLoading } = usePermissionCatalog(workspaceId);

  const permissionOptions = useMemo(() => {
    return catalog.map((permission) => ({
      value: permission.id,
      label: `${t(`permissions.groups.${permission.group}`, { defaultValue: permission.group })} - ${t(permission.label_key, { defaultValue: permission.id })}`,
    }));
  }, [catalog, t]);

  const roleOptions = useMemo(() => {
    const rolesList = roles as WorkspaceRoleDef[] | undefined;
    return (rolesList ?? []).map((r) => ({
      value: r.id,
      label: r.name,
    }));
  }, [roles]);

  const memberOptions = useMemo(() => {
    const membersList = members as WorkspaceMemberDef[] | undefined;
    return (membersList ?? []).map((m) => ({
      description: m.email,
      label: m.full_name ?? m.email,
      value: m.user_id,
    }));
  }, [members]);

  return {
    permissionOptions,
    roleOptions,
    memberOptions,
    isPermissionCatalogLoading: isCatalogLoading,
    isPermissionCatalogError: isCatalogError,
  };
}

interface UseWorkflowFormProps {
  defaultEntityType: string;
  onSuccess: () => void;
  workflowId: string;
}

export function useWorkflowForm({ workflowId, defaultEntityType, onSuccess }: UseWorkflowFormProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const { workspace } = useWorkspace();
  const { data: workflow, isLoading: isDetailsLoading } = useApprovalWorkflow(workflowId);
  const { mutateAsync: createWorkflow, isPending: isCreating } = useCreateApprovalWorkflow();
  const { mutateAsync: updateWorkflow, isPending: isUpdating } = useUpdateApprovalWorkflow(workflowId);

  const form = useForm<WorkflowFormValues>({
    resolver: zodResolver(workflowSchema),
    defaultValues: createDefaultFormValues(defaultEntityType, t),
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
        steps:
          workflow.steps?.map((s) => {
            const config = s.approver_config as
              | { role_id?: string; permission_name?: string; user_id?: string }
              | undefined;
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
      const payload = formatWorkflowPayload(data);
      if (workflowId) {
        await updateWorkflow(payload);
        toast.success(t("settings.workflows.messages.update_success", { defaultValue: "Workflow updated successfully" }));
      } else {
        await createWorkflow(payload);
        toast.success(t("settings.workflows.messages.create_success", { defaultValue: "Workflow created successfully" }));
      }
      onSuccess();
    } catch (error) {
      logError(error, { component: "useWorkflowEditor" });
      toast.error(t("settings.workflows.messages.save_failed", { defaultValue: "Failed to save workflow" }));
    }
  };

  const isPending = isCreating || isUpdating;

  return {
    form,
    fields,
    append,
    remove,
    move,
    watchSteps,
    onSubmit,
    isPending,
    isDetailsLoading,
    workspace,
  };
}

export function useWorkflowEditor() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { workflowId } = useParams({ strict: false });
  const search = useSearch({ strict: false });

  const defaultEntityType =
    search.tab === "leave"
      ? "leave_request"
      : search.tab === "attendance"
        ? "attendance_adjustment"
        : "ot_request";

  const handleSuccess = useCallback((entityType: string) => {
    let backTab: "overtime" | "leave" | "attendance" = "overtime";
    if (entityType === "leave_request") {
      backTab = "leave";
    } else if (entityType === "attendance_adjustment") {
      backTab = "attendance";
    }

    void navigate({
      to: ROUTES.SETTINGS.WORKFLOWS,
      search: { tab: backTab },
    });
  }, [navigate]);

  const {
    form,
    fields,
    append,
    remove,
    move,
    watchSteps,
    onSubmit,
    isPending,
    isDetailsLoading,
    workspace,
  } = useWorkflowForm({
    workflowId: workflowId ?? "",
    defaultEntityType,
    onSuccess: () => {
      handleSuccess(form.getValues("entity_type"));
    },
  });

  const handleBack = () => {
    handleSuccess(form.getValues("entity_type"));
  };

  const { permissionOptions, roleOptions, memberOptions } = useWorkflowOptions(workspace?.id ?? "");

  return {
    t,
    ws: workspace,
    workflowId,
    isDetailsLoading,
    form,
    fields,
    append,
    remove,
    move,
    watchSteps,
    handleBack,
    onSubmit,
    isPending,
    permissionOptions,
    roleOptions,
    memberOptions,
  };
}
