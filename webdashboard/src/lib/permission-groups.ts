// Permission group helpers for the workspace permission management UI.
// Split out from the permission listings (now in permissive-list-constants.ts)
// to keep the file under max-lines.
//
// Only `permissionGroups` has active consumers. The derived maps and lookups
// were removed as dead code — import directly from permissive-list-constants
// if you need individual permission definitions.

export type { PermissionDef, PermissionGroup } from "./permissive-list-constants";
export { permissionGroups } from "./permissive-list-constants";
