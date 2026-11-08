import type { CreateConfigEntryRequest } from "@/lib/api/types";

import { useOrganizationQuery, useOrganizationMutation } from "@/hooks/common/use-organization-query";
import { workspaceConfigApi } from "@/lib/api/workspace-config";

export const organizationConfigKeys = {
  all: ["organization-configs"] as const,
  lists: () => [...organizationConfigKeys.all, "list"] as const,
  list: (prefix?: string) => [...organizationConfigKeys.lists(), { prefix }] as const,
  details: () => [...organizationConfigKeys.all, "detail"] as const,
  detail: (key: string, date?: string) => [...organizationConfigKeys.details(), key, { date }] as const,
};

export const useWorkspaceConfigs = (prefix?: string) => {
  return useOrganizationQuery({
    queryKey: organizationConfigKeys.list(prefix),
    queryFn: () => workspaceConfigApi.listConfigs(prefix),
  });
};

export const useWorkspaceConfig = (key: string, date?: string) => {
  return useOrganizationQuery({
    queryKey: organizationConfigKeys.detail(key, date),
    queryFn: () => workspaceConfigApi.getConfig(key, date),
    enabled: !!key,
  });
};

export const useUpsertWorkspaceConfig = () =>
  useOrganizationMutation<CreateConfigEntryRequest>({
    mutationFn: (payload) => workspaceConfigApi.upsertConfig(payload),
    keys: [organizationConfigKeys.all],
  });

export const useDeleteWorkspaceConfig = () =>
  useOrganizationMutation<string>({
    mutationFn: (id) => workspaceConfigApi.deleteConfig(id),
    keys: [organizationConfigKeys.all],
  });
