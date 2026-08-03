import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

import type { CreateWorkflowRequest, UpdateWorkflowRequest } from '@/lib/api/types';

import { useWorkspace } from '@/hooks/use-workspace';
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
  const workspaceId = workspace?.id;
  
  return useQuery({
    queryKey: approvalWorkflowKeys.list(workspaceId ?? '', entityType),
    queryFn: () => {
      if (!workspaceId) throw new Error('No workspace ID');
      return approvalWorkflowsApi.listWorkflows(workspaceId, entityType);
    },
    enabled: !!workspaceId,
  });
};

export const useApprovalWorkflow = (id: string) => {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id;
  
  return useQuery({
    queryKey: approvalWorkflowKeys.detail(workspaceId ?? '', id),
    queryFn: () => {
      if (!workspaceId) throw new Error('No workspace ID');
      return approvalWorkflowsApi.getWorkflow(workspaceId, id);
    },
    enabled: !!workspaceId && !!id,
  });
};

export const useCreateApprovalWorkflow = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (payload: CreateWorkflowRequest) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error('No workspace ID');
      return approvalWorkflowsApi.createWorkflow(workspaceId, payload);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: approvalWorkflowKeys.all });
    },
  });
};

export const useUpdateApprovalWorkflow = (id: string) => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (payload: UpdateWorkflowRequest) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error('No workspace ID');
      return approvalWorkflowsApi.updateWorkflow(workspaceId, id, payload);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: approvalWorkflowKeys.all });
    },
  });
};

export const useDeleteApprovalWorkflow = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (id: string) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error('No workspace ID');
      return approvalWorkflowsApi.deleteWorkflow(workspaceId, id);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: approvalWorkflowKeys.all });
    },
  });
};

export const useApprovalHistory = (entityType: string, entityId: string) => {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id;
  
  return useQuery({
    queryKey: approvalWorkflowKeys.history(workspaceId ?? '', entityType, entityId),
    queryFn: () => {
      if (!workspaceId) throw new Error('No workspace ID');
      return approvalWorkflowsApi.getApprovalHistory(workspaceId, entityType, entityId);
    },
    enabled: !!workspaceId && !!entityType && !!entityId,
  });
};

export const useProcessApproval = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (payload: { entity_type: string; entity_id: string; action: string; note?: string }) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error('No workspace ID');
      return approvalWorkflowsApi.processApproval(workspaceId, payload);
    },
    onSuccess: (_, variables) => {
      const workspaceId = workspace?.id;
      if (workspaceId) {
        void queryClient.invalidateQueries({ queryKey: approvalWorkflowKeys.history(workspaceId, variables.entity_type, variables.entity_id) });
      }
    },
  });
};
