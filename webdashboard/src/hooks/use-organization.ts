import { useOrganizationQuery } from "@/hooks/common/use-organization-query";
import { fetchOrganization } from "@/lib/api";

import { organizationKeys } from "./use-organization-queries";

export function useOrganization() {
  const query = useOrganizationQuery({
    queryKey: organizationKeys.detail(),
    queryFn: fetchOrganization,
  });

  const organization = query.data ?? null;
  return {
    organization,
    organizationName: organization?.name ?? "",
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
  useOrganizationConfigs,
  useOrganizationConfig,
  useUpsertOrganizationConfig,
  useDeleteOrganizationConfig,
} from "./use-organization-config";
