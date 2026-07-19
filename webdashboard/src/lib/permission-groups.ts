// Permission group helpers for the workspace permission management UI.
// Split out from the permission listings (now in permissive-list-constants.ts)
// to keep the file under max-lines.

import type { LucideIcon } from "lucide-react";

import type { PermissionDef, PermissionGroup } from "./permissive-list-constants";

import { permissionGroups } from "./permissive-list-constants";

export type { PermissionDef, PermissionGroup } from "./permissive-list-constants";
export { permissionGroups } from "./permissive-list-constants";

export type PermissionGroupById = Record<string, PermissionGroup>;

export const permissionGroupById: PermissionGroupById = Object.fromEntries(
  permissionGroups.map((g) => [g.id, g])
);

export type PermissionIconMap = Record<string, LucideIcon>;

export const permissionIconMap: PermissionIconMap = Object.fromEntries(
  permissionGroups.map((g) => [g.id, g.icon])
);

export type AllPermissionsById = Record<string, PermissionDef>;

export const allPermissionsById: AllPermissionsById = Object.fromEntries(
  permissionGroups.flatMap((g) => g.permissions.map((p) => [p.id, p]))
);

export function getPermissionName(permissionId: string): string {
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
  return allPermissionsById[permissionId]?.name ?? permissionId;
}

export function getPermissionDescription(permissionId: string): string {
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
  return allPermissionsById[permissionId]?.desc ?? "";
}

export function getGroupForPermission(permissionId: string): PermissionGroup | undefined {
  return permissionGroups.find((g) => g.permissions.some((p) => p.id === permissionId));
}
