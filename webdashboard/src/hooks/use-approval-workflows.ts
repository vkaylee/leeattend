import { useQuery, useQueryClient } from '@tanstack/react-query';

import type { CreateWorkflowRequest, UpdateWorkflowRequest } from '@/lib/api/types';

import { useWorkspace } from '@/hooks/use-workspace';
import { useWorkspaceQuery, useWorkspaceMutation } from '@/hooks/common/use-workspace-query';
import { approvalWorkflowsApi } from '@/lib/api/approval-workflows';



export const approvalWorkflowKeys = {
  all: ['approval-workflows'] as const,
  lists: (workspaceId: string) => [...approvalWorkflowKeys.all, workspaceId, 'list'] as const,
  list: (workspaceId: string, entityType?: string) => [...approvalWorkflowKeys.lists(workspaceId), { entityType }] as const,
  details: () => [...approvalWorkflowKeys.all, 'detail'] as const,
  detail: (workspaceId: string, id: string) => [...approvalWorkflowKeys.details(), workspaceId, id] as const,
  histories: () => [...approvalWorkflowKeys.all, 'history'] as const,
  history: (workspaceId: string, entityType: string, entityId: string) => [...approvalWorkflowKeys.histories(), workspaceId, entityType, entityId] as const,
};

export const useApprovalWorkflows = (entityType?: string) => {
  const { workspace } = useWorkspace();
  return useWorkspaceQuery({
    queryKey: approvalWorkflowKeys.list(workspace?.id ?? '', entityType),
    queryFn: (wsId) => approvalWorkflowsApi.listWorkflows(wsId, entityType),
  });
};

export const useApprovalWorkflow = (id: string) => {
  const { workspace } = useWorkspace();
  return useWorkspaceQuery({
    queryKey: approvalWorkflowKeys.detail(workspace?.id ?? '', id),
    queryFn: (wsId) => approvalWorkflowsApi.getWorkflow(wsId, id),
    enabled: !!id,
  });
};

export const useCreateApprovalWorkflow = () =>
  useWorkspaceMutation<CreateWorkflowRequest, unknown>({
    mutationFn: (wsId, payload) => approvalWorkflowsApi.createWorkflow(wsId, payload),
    keys: [approvalWorkflowKeys.all],
  });

export const useUpdateApprovalWorkflow = (id: string) =>
  useWorkspaceMutation<UpdateWorkflowRequest, unknown>({
    mutationFn: (wsId, payload) => approvalWorkflowsApi.updateWorkflow(wsId, id, payload),
    keys: [approvalWorkflowKeys.all],
  });

export const useDeleteApprovalWorkflow = () =>
  useWorkspaceMutation<string, unknown>({
    mutationFn: (wsId, id) => approvalWorkflowsApi.deleteWorkflow(wsId, id),
    keys: [approvalWorkflowKeys.all],
  });

export const useApprovalHistory = (entityType: string, entityId: string) => {
  const { workspace } = useWorkspace();
  return useWorkspaceQuery({
    queryKey: approvalWorkflowKeys.history(workspace?.id ?? '', entityType, entityId),
    queryFn: (wsId) => approvalWorkflowsApi.getApprovalHistory(wsId, entityType, entityId),
    enabled: !!entityType && !!entityId,
  });
};

export const useProcessApproval = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();
  return useWorkspaceMutation<{ entity_type: string; entity_id: string; action: string; note?: string }, unknown>({
    mutationFn: (wsId, payload) => approvalWorkflowsApi.processApproval(wsId, payload),
    keys: [], // Custom onSuccess handles invalidation
    onSuccess: (_, variables) => {
      if (workspace?.id) {
        void queryClient.invalidateQueries({
          queryKey: approvalWorkflowKeys.history(workspace.id, variables.entity_type, variables.entity_id),
        });
      }
    },
  });
};
