# Permission Catalog Tasks

- [x] **1. Backend catalog vertical slice**
  - Added `PermissionCatalogItem` metadata and `GET /v1/workspaces/{ws_id}/permissions/catalog` with OpenAPI registration, `MembersRolesManage` authorization, compatibility parity, and integration tests.
  - Verified 102 ordered entries including `overtime:request:create_on_behalf`; unauthorized access is 403; `/available` IDs equal catalog IDs.

- [x] **2. Typed frontend contract**
  - Ran `scripts/_sync-api.sh`; added generated `PermissionCatalogItem`, typed wrapper, workspace query key, and cached catalog hooks.
  - Verified generated API typecheck and focused hook/API consumers.

- [x] **3. Matrix vertical slice**
  - Replaced hardcoded matrix permission definitions with catalog response; retained only group icon mapping and preserved role save/toggle gates.
  - Verified pre-OT-on-behalf rendering/selection, loading/error behavior, and workspace matrix tests.

- [x] **4. Remaining consumers**
  - Migrated workflow options and member grant selector to catalog-derived IDs/localized labels while preserving canonical mutation payloads.
  - Updated affected tests and verified focused consumers.

- [x] **5. i18n and drift protection**
  - Added permission namespace translations in en/vi/zh, including group labels and `overtime:request:create_on_behalf` name/description; removed the frontend 101-ID list and added icon/translation contract assertions.
  - Added namespace-aware translation-test coverage and backend catalog parity/metadata tests.

- [x] **6. Full verification**
  - Regenerated API types, inspected the catalog diff, and ran the complete API, web, and all suites through `leedevkit`.
  - API: 2954/2954; Web: 5936/5936; All: 5936/5936. Unrelated existing worktree changes were preserved.
