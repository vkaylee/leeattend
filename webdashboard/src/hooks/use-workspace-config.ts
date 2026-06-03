import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

import type { CreateConfigEntryRequest } from '@/lib/api/types';

import { useWorkspace } from '@/hooks/use-workspace';
import { workspaceConfigApi } from '@/lib/api/workspace-config';


export const workspaceConfigKeys = {
  all: ['workspace-configs'] as const,
  lists: (workspaceId: string) => [...workspaceConfigKeys.all, workspaceId, 'list'] as const,
  list: (workspaceId: string, prefix?: string) => [...workspaceConfigKeys.lists(workspaceId), { prefix }] as const,
  details: () => [...workspaceConfigKeys.all, 'detail'] as const,
  detail: (workspaceId: string, key: string, date?: string) => [...workspaceConfigKeys.details(), workspaceId, key, { date }] as const,
};

export const useWorkspaceConfigs = (prefix?: string) => {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id;
  
  return useQuery({
    queryKey: workspaceConfigKeys.list(workspaceId ?? '', prefix),
    queryFn: () => {
      if (!workspaceId) throw new Error('No workspace ID');
      return workspaceConfigApi.listConfigs(workspaceId, prefix);
    },
    enabled: !!workspaceId,
  });
};

export const useWorkspaceConfig = (key: string, date?: string) => {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id;
  
  return useQuery({
    queryKey: workspaceConfigKeys.detail(workspaceId ?? '', key, date),
    queryFn: () => {
      if (!workspaceId) throw new Error('No workspace ID');
      return workspaceConfigApi.getConfig(workspaceId, key, date);
    },
    enabled: !!workspaceId && !!key,
  });
};

export const useUpsertWorkspaceConfig = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (payload: CreateConfigEntryRequest) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error('No workspace ID');
      return workspaceConfigApi.upsertConfig(workspaceId, payload);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: workspaceConfigKeys.all });
    },
  });
};

export const useDeleteWorkspaceConfig = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (id: string) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error('No workspace ID');
      return workspaceConfigApi.deleteConfig(workspaceId, id);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: workspaceConfigKeys.all });
    },
  });
};
