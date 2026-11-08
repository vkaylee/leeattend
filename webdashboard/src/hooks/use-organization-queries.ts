export const organizationKeys = {
  all: ["organization"] as const,
  detail: () => ["organization"] as const,
  members: () => ["organization", "members"] as const,
  memberRoles: (userId: string) => ["organization", "members", userId, "roles"] as const,
  memberPermissions: (userId: string) => ["organization", "members", userId, "permissions"] as const,
  availablePermissions: () => ["organization", "permissions", "available"] as const,
  permissionCatalog: () => ["organization", "permissions", "catalog"] as const,
  roles: () => ["organization", "roles"] as const,
  rolePermissions: (roleId: string) => ["organization", "roles", roleId, "permissions"] as const,
  permissionsMatrix: () => ["organization", "permissions", "matrix"] as const,
};
