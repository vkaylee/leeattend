import { useQueryClient } from '@tanstack/react-query';

import type { CreateWorkflowRequest, UpdateWorkflowRequest } from '@/lib/api/types';

import { useOrganizationQuery, useOrganizationMutation } from '@/hooks/common/use-organization-query';
import { approvalWorkflowsApi } from '@/lib/api/approval-workflows';

export const approvalWorkflowKeys = {
  all: ['approval-workflows'] as const,
  lists: () => [...approvalWorkflowKeys.all, 'list'] as const,
  list: (entityType?: string) => [...approvalWorkflowKeys.lists(), { entityType }] as const,
  details: () => [...approvalWorkflowKeys.all, 'detail'] as const,
  detail: (id: string) => [...approvalWorkflowKeys.details(), id] as const,
  histories: () => [...approvalWorkflowKeys.all, 'history'] as const,
  history: (entityType: string, entityId: string) => [...approvalWorkflowKeys.histories(), entityType, entityId] as const,
};

export const useApprovalWorkflows = (entityType?: string) => {
  return useOrganizationQuery({
    queryKey: approvalWorkflowKeys.list(entityType),
    queryFn: () => approvalWorkflowsApi.listWorkflows(entityType),
  });
};

export const useApprovalWorkflow = (id: string) => {
  return useOrganizationQuery({
    queryKey: approvalWorkflowKeys.detail(id),
    queryFn: () => approvalWorkflowsApi.getWorkflow(id),
    enabled: !!id,
  });
};

export const useCreateApprovalWorkflow = () =>
  useOrganizationMutation<CreateWorkflowRequest>({
    mutationFn: (payload) => approvalWorkflowsApi.createWorkflow(payload),
    keys: [approvalWorkflowKeys.all],
  });

export const useUpdateApprovalWorkflow = (id: string) =>
  useOrganizationMutation<UpdateWorkflowRequest>({
    mutationFn: (payload) => approvalWorkflowsApi.updateWorkflow(id, payload),
    keys: [approvalWorkflowKeys.all],
  });

export const useDeleteApprovalWorkflow = () =>
  useOrganizationMutation<string>({
    mutationFn: (id) => approvalWorkflowsApi.deleteWorkflow(id),
    keys: [approvalWorkflowKeys.all],
  });

export const useApprovalHistory = (entityType: string, entityId: string) => {
  return useOrganizationQuery({
    queryKey: approvalWorkflowKeys.history(entityType, entityId),
    queryFn: () => approvalWorkflowsApi.getApprovalHistory(entityType, entityId),
    enabled: !!entityType && !!entityId,
  });
};

export const useProcessApproval = () => {
  const queryClient = useQueryClient();
  return useOrganizationMutation<{ entity_type: string; entity_id: string; action: string; note?: string }>({
    mutationFn: (payload) => approvalWorkflowsApi.processApproval(payload),
    keys: [], // Custom onSuccess handles invalidation
    onSuccess: (_, variables) => {
      void queryClient.invalidateQueries({
        queryKey: approvalWorkflowKeys.history(variables.entity_type, variables.entity_id),
      });
    },
  });
};
