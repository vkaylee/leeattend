import { useOrganizationQuery } from "@/hooks/common/use-organization-query";
import { fetchOrganization } from "@/lib/api";

import { organizationKeys } from "./use-organization-queries";

export function useOrganization() {
  const query = useOrganizationQuery({
    queryKey: organizationKeys.detail(),
    queryFn: fetchOrganization,
  });

  return {
    organization: query.data ?? null,
    organizationName: query.data?.name ?? "",
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,
  };
}

export { organizationKeys } from "./use-organization-queries";
export { useOrganizationMembers, useMemberRoles, useMemberPermissions } from "./use-organization-members";
export {
  usePermissionCatalog,
  useOrganizationRoles,
  useRolePermissions,
  useOrganizationPermissionsMatrix,
} from "./use-organization-roles";
export {
  organizationConfigKeys,
  useWorkspaceConfigs as useOrganizationConfigs,
  useWorkspaceConfig as useOrganizationConfig,
  useUpsertWorkspaceConfig as useUpsertOrganizationConfig,
  useDeleteWorkspaceConfig as useDeleteOrganizationConfig,
} from "./use-organization-config";
