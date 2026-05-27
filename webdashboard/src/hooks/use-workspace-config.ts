import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

import { useWorkspace } from '@/hooks/use-workspace';
import { workspaceConfigApi, CreateConfigEntryRequest } from '@/lib/api/workspace-config';

export const workspaceConfigKeys = {
  all: ['workspace-configs'] as const,
  lists: (workspaceId: string) => [...workspaceConfigKeys.all, workspaceId, 'list'] as const,
  list: (workspaceId: string, prefix?: string) => [...workspaceConfigKeys.lists(workspaceId), { prefix }] as const,
  details: () => [...workspaceConfigKeys.all, 'detail'] as const,
  detail: (workspaceId: string, key: string, date?: string) => [...workspaceConfigKeys.details(), workspaceId, key, { date }] as const,
};

export const useWorkspaceConfigs = (prefix?: string) => {
  const { workspace } = useWorkspace();
  
  return useQuery({
    queryKey: workspaceConfigKeys.list(workspace?.id || '', prefix),
    queryFn: () => workspaceConfigApi.listConfigs(workspace!.id, prefix),
    enabled: !!workspace?.id,
  });
};

export const useWorkspaceConfig = (key: string, date?: string) => {
  const { workspace } = useWorkspace();
  
  return useQuery({
    queryKey: workspaceConfigKeys.detail(workspace?.id || '', key, date),
    queryFn: () => workspaceConfigApi.getConfig(workspace!.id, key, date),
    enabled: !!workspace?.id && !!key,
  });
};

export const useUpsertWorkspaceConfig = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (payload: CreateConfigEntryRequest) => 
      workspaceConfigApi.upsertConfig(workspace!.id, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: workspaceConfigKeys.all });
    },
  });
};

export const useDeleteWorkspaceConfig = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (id: string) => 
      workspaceConfigApi.deleteConfig(workspace!.id, id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: workspaceConfigKeys.all });
    },
  });
};
