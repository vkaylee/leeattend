import type { CreateConfigEntryRequest } from "@/lib/api/types";

import { useOrganizationMutation, useOrganizationQuery } from "@/hooks/common/use-organization-query";
import { organizationConfigApi } from "@/lib/api/organization-config";

export const organizationConfigKeys = {
  all: ["organization-configs"] as const,
  lists: () => [...organizationConfigKeys.all, "list"] as const,
  list: (prefix?: string) => [...organizationConfigKeys.lists(), { prefix }] as const,
  details: () => [...organizationConfigKeys.all, "detail"] as const,
  detail: (key: string, date?: string) => [...organizationConfigKeys.details(), key, { date }] as const,
};

export const useOrganizationConfigs = (prefix?: string) => useOrganizationQuery({
  queryKey: organizationConfigKeys.list(prefix),
  queryFn: () => organizationConfigApi.listConfigs(prefix),
});

export const useOrganizationConfig = (key: string, date?: string) => useOrganizationQuery({
  queryKey: organizationConfigKeys.detail(key, date),
  queryFn: () => organizationConfigApi.getConfig(key, date),
  enabled: !!key,
});

export const useUpsertOrganizationConfig = () => useOrganizationMutation<CreateConfigEntryRequest>({
  mutationFn: (payload) => organizationConfigApi.upsertConfig(payload),
  keys: [organizationConfigKeys.all],
});

export const useDeleteOrganizationConfig = () => useOrganizationMutation<string>({
  mutationFn: (id) => organizationConfigApi.deleteConfig(id),
  keys: [organizationConfigKeys.all],
});
