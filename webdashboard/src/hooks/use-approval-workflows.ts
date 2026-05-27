import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

import { useWorkspace } from '@/hooks/use-workspace';
import { approvalWorkflowsApi, CreateWorkflowRequest, UpdateWorkflowRequest } from '@/lib/api/approval-workflows';

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
  
  return useQuery({
    queryKey: approvalWorkflowKeys.list(workspace?.id || '', entityType),
    queryFn: () => approvalWorkflowsApi.listWorkflows(workspace!.id, entityType),
    enabled: !!workspace?.id,
  });
};

export const useApprovalWorkflow = (id: string) => {
  const { workspace } = useWorkspace();
  
  return useQuery({
    queryKey: approvalWorkflowKeys.detail(workspace?.id || '', id),
    queryFn: () => approvalWorkflowsApi.getWorkflow(workspace!.id, id),
    enabled: !!workspace?.id && !!id,
  });
};

export const useCreateApprovalWorkflow = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (payload: CreateWorkflowRequest) => 
      approvalWorkflowsApi.createWorkflow(workspace!.id, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: approvalWorkflowKeys.all });
    },
  });
};

export const useUpdateApprovalWorkflow = (id: string) => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (payload: UpdateWorkflowRequest) => 
      approvalWorkflowsApi.updateWorkflow(workspace!.id, id, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: approvalWorkflowKeys.all });
    },
  });
};

export const useDeleteApprovalWorkflow = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (id: string) => 
      approvalWorkflowsApi.deleteWorkflow(workspace!.id, id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: approvalWorkflowKeys.all });
    },
  });
};

export const useApprovalHistory = (entityType: string, entityId: string) => {
  const { workspace } = useWorkspace();
  
  return useQuery({
    queryKey: approvalWorkflowKeys.history(workspace?.id || '', entityType, entityId),
    queryFn: () => approvalWorkflowsApi.getApprovalHistory(workspace!.id, entityType, entityId),
    enabled: !!workspace?.id && !!entityType && !!entityId,
  });
};

export const useProcessApproval = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (payload: { entity_type: string; entity_id: string; action: string; note?: string }) => 
      approvalWorkflowsApi.processApproval(workspace!.id, payload),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: approvalWorkflowKeys.history(workspace!.id, variables.entity_type, variables.entity_id) });
      // We'll let the specific entity (like overtime requests) invalidate its own detail
    },
  });
};
