# Permission Catalog Synchronization Plan

## Context

The editable frontend permission matrix omitted `overtime:request:create_on_behalf`, although Rust defines and enforces it. Three manually-maintained lists drifted: Rust has 102 IDs, while the frontend list and its test have 101.

## Recommended architecture

- Make one ordered Rust `PermissionCatalogItem` metadata definition authoritative for IDs, groups, and i18n name keys.
- Add authorized `GET /v1/workspaces/{ws_id}/permissions/catalog` returning `{ id, group, label_key }`.
- Keep `/permissions/available` as a compatibility string list derived from the catalog.
- Generate API types via `scripts/_sync-api.sh`; do not hand-edit generated types.
- Render the matrix, workflow permission choices, and member grant selector from the catalog. The frontend may retain only a group-to-icon map.
- Add name/description and group i18n keys in en/vi/zh. Replace the frontend hardcoded sync list with contract/coverage tests.

## Dependency graph

```text
Rust catalog -> catalog endpoint/OpenAPI -> generated TS -> query hook
  -> permission matrix
  -> workflow options
  -> member permission selector
Rust catalog -> legacy string endpoint
Rust catalog + UI group map -> i18n and drift tests
```

## Checkpoints

1. **Backend:** 102 unique metadata entries; endpoint returns the required pre-OT ID; non-manager gets 403; legacy endpoint equals catalog IDs.
2. **Contract:** generated OpenAPI types and typed React Query hook compile and are tested.
3. **UI:** matrix grants/saves the pre-OT permission and handles loading/error accessibly.
4. **Drift:** all runtime consumers use catalog data; locales cover every label key; remove the 101-item frontend copy.
5. **Completion:** `./leedevkit test api`, `./leedevkit test web`, and `./leedevkit test all` all pass with phase-level results inspected.

## Constraints

- Preserve unrelated worker-monitor and responsive-prompt worktree changes.
- Use `AppError`/authorization before handler behavior; use `MembersRolesManage` for the catalog.
- Use shell output redirection and wrappers only; no raw cargo/npm/diesel.
