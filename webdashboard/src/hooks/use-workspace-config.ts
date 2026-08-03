import type { CreateConfigEntryRequest } from '@/lib/api/types';

import { useWorkspaceQuery, useWorkspaceMutation } from '@/hooks/common/use-workspace-query';
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
  return useWorkspaceQuery({
    queryKey: workspaceConfigKeys.list(workspace?.id ?? '', prefix),
    queryFn: (wsId) => workspaceConfigApi.listConfigs(wsId, prefix),
  });
};

export const useWorkspaceConfig = (key: string, date?: string) => {
  const { workspace } = useWorkspace();
  return useWorkspaceQuery({
    queryKey: workspaceConfigKeys.detail(workspace?.id ?? '', key, date),
    queryFn: (wsId) => workspaceConfigApi.getConfig(wsId, key, date),
    enabled: !!key,
  });
};

export const useUpsertWorkspaceConfig = () =>
  useWorkspaceMutation<CreateConfigEntryRequest>({
    mutationFn: (wsId, payload) => workspaceConfigApi.upsertConfig(wsId, payload),
    keys: [workspaceConfigKeys.all],
  });

export const useDeleteWorkspaceConfig = () =>
  useWorkspaceMutation<string>({
    mutationFn: (wsId, id) => workspaceConfigApi.deleteConfig(wsId, id),
    keys: [workspaceConfigKeys.all],
  });
