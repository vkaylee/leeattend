import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { workspaceConfigApi, CreateConfigEntryRequest } from '@/lib/api/workspace-config';
import { useAuth } from '@/hooks/use-auth';

export const workspaceConfigKeys = {
  all: ['workspace-configs'] as const,
  lists: (workspaceId: string) => [...workspaceConfigKeys.all, workspaceId, 'list'] as const,
  list: (workspaceId: string, prefix?: string) => [...workspaceConfigKeys.lists(workspaceId), { prefix }] as const,
  details: () => [...workspaceConfigKeys.all, 'detail'] as const,
  detail: (workspaceId: string, key: string, date?: string) => [...workspaceConfigKeys.details(), workspaceId, key, { date }] as const,
};

export const useWorkspaceConfigs = (prefix?: string) => {
  const { currentWorkspace } = useAuth();
  
  return useQuery({
    queryKey: workspaceConfigKeys.list(currentWorkspace?.id || '', prefix),
    queryFn: () => workspaceConfigApi.listConfigs(currentWorkspace!.id, prefix),
    enabled: !!currentWorkspace?.id,
  });
};

export const useWorkspaceConfig = (key: string, date?: string) => {
  const { currentWorkspace } = useAuth();
  
  return useQuery({
    queryKey: workspaceConfigKeys.detail(currentWorkspace?.id || '', key, date),
    queryFn: () => workspaceConfigApi.getConfig(currentWorkspace!.id, key, date),
    enabled: !!currentWorkspace?.id && !!key,
  });
};

export const useUpsertWorkspaceConfig = () => {
  const queryClient = useQueryClient();
  const { currentWorkspace } = useAuth();

  return useMutation({
    mutationFn: (payload: CreateConfigEntryRequest) => 
      workspaceConfigApi.upsertConfig(currentWorkspace!.id, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: workspaceConfigKeys.all });
    },
  });
};

export const useDeleteWorkspaceConfig = () => {
  const queryClient = useQueryClient();
  const { currentWorkspace } = useAuth();

  return useMutation({
    mutationFn: (id: string) => 
      workspaceConfigApi.deleteConfig(currentWorkspace!.id, id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: workspaceConfigKeys.all });
    },
  });
};
