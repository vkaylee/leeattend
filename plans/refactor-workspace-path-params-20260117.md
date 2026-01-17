# Implementation Plan: Refactor Workspace Identification to Path Parameters

Refactor the backend to prioritize workspace identification via URI path parameters and remove the strict requirement for the `X-Workspace-Id` header when the ID is available in the URL.

## Approach
- **Smart Extractor**: Update the `WorkspaceId` extractor to check path parameters (`ws_id` or `id`) before falling back to the `X-Workspace-Id` header.
- **Gradual Migration**: Keep the header as a fallback to avoid breaking ESS (Employee Self-Service) routes and older client versions immediately.
- **Verification**: Update key integration tests to remove redundant headers and ensure they still pass.

## Steps
1. **Update `WorkspaceId` Extractor** (10 min)
   - Modify `apiserver/src/handlers/common.rs`.
   - Use `axum::extract::Path` to check for `ws_id` or `id` in path parameters.
   - Update error messages to reflect the new options.

2. **Backend Verification** (15 min)
   - Run existing integration tests (they still send the header, so they should pass).
   - Modify a specific test file (e.g., `apiserver/tests/workspace_management.rs`) to remove redundant headers and verify success.

3. **Frontend Audit** (15 min)
   - Identify where `X-Workspace-Id` is sent in `webdashboard/src/lib/api/helpers.ts`.
   - Consider if we can stop sending it for path-based routes (optional but cleaner).

4. **Documentation Update** (5 min)
   - Update `docs/api-reference.md` to indicate that the header is now optional for path-based routes.

## Timeline
| Phase | Duration |
|-------|----------|
| Extractor Update | 10 min |
| Backend Verification | 15 min |
| Frontend Audit | 15 min |
| Documentation | 5 min |
| **Total** | **45 min** |

## Rollback Plan
- Revert changes to `apiserver/src/handlers/common.rs`.

## Security Checklist
- [x] Ensure cross-tenant validation (MemberContext) still works correctly using the path-extracted ID.
- [x] Verify that missing IDs still result in `400 Bad Request` or `401 Unauthorized` as appropriate.
