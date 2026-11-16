# Feature Flag Management UI

Status: Awaiting approval

## Goal
Cho phép người có quyền xem danh sách và bật/tắt feature flag trong Settings, đồng thời dùng trạng thái runtime từ backend thay cho compile-time-only flag ở frontend.

## Constraints and Interfaces
- **Authorization:** `feature_flags:read` cho trang/danh sách; `feature_flags:manage` cho bật/tắt và CRUD. Backend đã có permission và CRUD endpoints; phải giữ default-deny, không mở quyền cho employee.
- **Runtime contract:** giữ `/api/v1/feature-flags` cho quản trị; thêm hoặc điều chỉnh endpoint effective-flags cho user đã đăng nhập nếu role sử dụng feature không có `feature_flags:read`. Response chỉ chứa key/`enabled`/metadata công khai cần thiết, không trả dữ liệu quản trị nhạy cảm.
- **Frontend compatibility:** `isFeatureEnabled()` phải fail closed (`false`) khi loading, 403, lỗi mạng, hoặc flag không tồn tại; DEV localStorage override chỉ còn cơ chế dev, không được ghi đè production/runtime state.
- **UI:** dùng settings components hiện có; có loading skeleton, empty state, error/retry, confirmation khi đổi trạng thái, keyboard/accessibility, i18n cho `vi`, `en`, `zh`.
- **Governance:** audit-log mọi thay đổi trạng thái; không log PII/secrets; cập nhật catalog `docs/operations/feature-flags.md` nếu file/chính sách đã có hoặc tạo theo convention repository.
- **Rules applied:** `access-control.md` §1–2; `configuration-management.md` §5; `change-management.md` §6; `api-governance.md` §1–6; `development-workflow.md` §2–7; `testing-standards.md` §1–4; `design-rules.md` §1–4.

## Tasks
- [ ] **1. Chốt contract runtime và quyền** — xác định flag nào là managed DB flag, flag nào là build-time-only; quyết định effective-flags endpoint hoặc cơ chế bootstrap tương đương; kiểm tra role mặc định và OpenAPI/generated types. **Verify:** contract ghi rõ auth, permissions, response, 401/403/5xx, fail-closed và backward compatibility.
- [ ] **2. Hoàn thiện backend flag lifecycle** — bảo đảm list/update/upsert validate key theo convention `ff_<area>_<name>`, có giới hạn metadata, audit-log state change, và xử lý cạnh tranh/idempotency phù hợp; bổ sung endpoint effective flags nếu cần. **Verify:** Go handler/repository tests cho success, malformed/duplicate key, unauthorized/forbidden, missing flag, DB failure, and state persistence.
- [ ] **3. Tạo frontend API/query layer** — thêm typed client cho list/effective/update; React Query cache key, invalidation sau mutation, stale/error behavior; chuyển `isFeatureEnabled` sang runtime store/provider với default false trong lúc chờ. **Verify:** unit tests cho loading, enabled, disabled, unknown, 403/network error, cache refresh, và DEV override không hoạt động production.
- [ ] **4. Thêm trang Settings → Feature flags** — đăng ký route/navigation với `feature_flags:read`; render bảng/card key, mô tả, trạng thái, updated time; switch chỉ enabled với `feature_flags:manage`; confirmation/toast và optimistic update có rollback hoặc refetch. **Verify:** component tests cho permission-based rendering, empty/loading/error states, keyboard toggle, confirmation cancel/confirm, mutation failure.
- [ ] **5. Đồng bộ flag consumers** — cập nhật app launcher, sidebar, settings nav, route guards, and feature-specific pages để cùng đọc runtime evaluator; `ff_econtracts` được seed/migrate hoặc fallback false rõ ràng, tránh màn hình hiện rồi redirect sai. **Verify:** tests chứng minh flag off ẩn menu/chặn route, flag on hiển thị, API lỗi vẫn fail closed.
- [ ] **6. Cập nhật tài liệu và vận hành** — ghi active flags, owner, creation/removal date, rollout/rollback procedure; document quyền cần cấp và cảnh báo production không dùng localStorage. **Verify:** docs review; migration has `down.sql` nếu schema thay đổi; audit event có thể truy vấn.
- [ ] **7. Verification** — chạy focused backend/frontend checks, sau đó target đầy đủ qua LeeDevKit, integration/API tests, rồi browser smoke test bằng role đúng quyền. **Verify:** báo cáo exact commands, pass/fail/skip/warnings; kiểm tra admin/manager có quyền và employee nhận 403/không thấy navigation.

## Test Impact Matrix

| Dimension | Coverage required |
|---|---|
| Changed behavior | Runtime flag load; management page; toggle state; consumer visibility |
| Existing behavior | Existing compile-time defaults; disabled route redirect; existing CRUD API |
| Inputs/boundaries | Empty list, unknown key, invalid key, duplicate key, oversized metadata, rapid repeated toggle |
| State | Initial loading, stale cache, refresh, concurrent admin update, mutation rollback |
| Failures | 401/403, timeout/network error, backend 5xx, DB write failure |
| Security | Employee/team lead unauthorized; manager without manage cannot mutate; cross-organization scope remains server-owned |
| Compatibility | Existing `/api/v1/feature-flags` clients; additive response fields; old frontend during rollout |
| Operations | Audit event, cache TTL/invalidation, no raw PII/secrets in logs |

## Done When
- [ ] Authorized manager can open Settings → Feature flags and toggle an existing flag.
- [ ] Unauthorized roles cannot discover or mutate management controls; API returns expected 403.
- [ ] Runtime consumers receive updated state without redeploy; all failures default closed.
- [ ] Audit trail records actor, flag key, old/new state, timestamp, and result without secrets/PII.
- [ ] Backend, frontend unit/integration, and required browser smoke tests pass through repository wrappers.
- [ ] Documentation records flag owner and cleanup deadline.

## Review Focus
- Effective flag endpoint must not expose management-only data or accidentally grant feature access through client-only checks.
- Backend authorization and audit logging must happen before/with state mutation.
- Cache invalidation must not leave users with stale enabled state after kill switch.
- Production must not honor localStorage overrides.
- Route guard, sidebar, launcher, and settings navigation must use one evaluator.

## Notes
- Existing surfaces: `webdashboard/src/lib/feature-flags.ts`, `webdashboard/src/pages/settings/hooks/use-settings-nav.tsx`, `apiserver/internal/api/organization.go`, and `apiserver/routes_organization.go`.
- Lazier alternative: keep flags compile-time and add only a developer/admin config file; do not choose this because requirement is runtime UI control and incident kill switch.
