import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { approvalWorkflowsApi, CreateWorkflowRequest, UpdateWorkflowRequest } from '@/lib/api/approval-workflows';
import { useAuth } from '@/hooks/use-auth';

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
  const { currentWorkspace } = useAuth();
  
  return useQuery({
    queryKey: approvalWorkflowKeys.list(currentWorkspace?.id || '', entityType),
    queryFn: () => approvalWorkflowsApi.listWorkflows(currentWorkspace!.id, entityType),
    enabled: !!currentWorkspace?.id,
  });
};

export const useApprovalWorkflow = (id: string) => {
  const { currentWorkspace } = useAuth();
  
  return useQuery({
    queryKey: approvalWorkflowKeys.detail(currentWorkspace?.id || '', id),
    queryFn: () => approvalWorkflowsApi.getWorkflow(currentWorkspace!.id, id),
    enabled: !!currentWorkspace?.id && !!id,
  });
};

export const useCreateApprovalWorkflow = () => {
  const queryClient = useQueryClient();
  const { currentWorkspace } = useAuth();

  return useMutation({
    mutationFn: (payload: CreateWorkflowRequest) => 
      approvalWorkflowsApi.createWorkflow(currentWorkspace!.id, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: approvalWorkflowKeys.all });
    },
  });
};

export const useUpdateApprovalWorkflow = (id: string) => {
  const queryClient = useQueryClient();
  const { currentWorkspace } = useAuth();

  return useMutation({
    mutationFn: (payload: UpdateWorkflowRequest) => 
      approvalWorkflowsApi.updateWorkflow(currentWorkspace!.id, id, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: approvalWorkflowKeys.all });
    },
  });
};

export const useDeleteApprovalWorkflow = () => {
  const queryClient = useQueryClient();
  const { currentWorkspace } = useAuth();

  return useMutation({
    mutationFn: (id: string) => 
      approvalWorkflowsApi.deleteWorkflow(currentWorkspace!.id, id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: approvalWorkflowKeys.all });
    },
  });
};

export const useApprovalHistory = (entityType: string, entityId: string) => {
  const { currentWorkspace } = useAuth();
  
  return useQuery({
    queryKey: approvalWorkflowKeys.history(currentWorkspace?.id || '', entityType, entityId),
    queryFn: () => approvalWorkflowsApi.getApprovalHistory(currentWorkspace!.id, entityType, entityId),
    enabled: !!currentWorkspace?.id && !!entityType && !!entityId,
  });
};

export const useProcessApproval = () => {
  const queryClient = useQueryClient();
  const { currentWorkspace } = useAuth();

  return useMutation({
    mutationFn: (payload: { entity_type: string; entity_id: string; action: string; note?: string }) => 
      approvalWorkflowsApi.processApproval(currentWorkspace!.id, payload),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: approvalWorkflowKeys.history(currentWorkspace!.id, variables.entity_type, variables.entity_id) });
      // We'll let the specific entity (like overtime requests) invalidate its own detail
    },
  });
};
