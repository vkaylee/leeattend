// ── Organization hooks ──────────────────────────────────────────────────

export { useOrganization } from "@/hooks/use-organization";
export {
  useWorkspaceConfig as useOrganizationConfig,
  useWorkspaceConfigs as useOrganizationConfigs,
  useUpsertWorkspaceConfig as useUpsertOrganizationConfig,
  useDeleteWorkspaceConfig as useDeleteOrganizationConfig,
} from "@/hooks/use-organization-config";
export {
  useOrganizationMembers,
  useMemberRoles,
  useMemberPermissions,
} from "@/hooks/use-organization-members";
export {
  usePermissionCatalog,
  useOrganizationRoles,
  useRolePermissions,
  useOrganizationPermissionsMatrix,
} from "@/hooks/use-organization-roles";
