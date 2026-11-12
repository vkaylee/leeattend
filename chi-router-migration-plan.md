# Chi Router Migration and Route Modularization

Status: Approved (2026-11-12)

## Rule IDs

- ARCH-02: thay router/framework phải có ADR, alternatives, consequence, rollback.
- API-01/API-06: giữ contract HTTP, authorization, error shape và compatibility tests.
- SDL-01/SDL-04: đánh giá trust boundary và chạy security verification cho auth route groups.
- DEV-02/DEV-06: test impact matrix và refactor phải giữ behavior hiện tại.
- SUPPLY-02: audit license, maintenance, vulnerabilities trước khi thêm dependency.

## Goal

Đánh giá rồi migration có kiểm soát từ `net/http.ServeMux` sang `github.com/go-chi/chi/v5`, đồng thời tách route registration khỏi `apiserver/main.go` theo domain để AI agent chỉ cần đọc module nhỏ, giảm lỗi đăng ký route và áp sai middleware.

## Constraints and Interfaces

- Không đổi public API path, HTTP method, status code, JSON error shape, auth behavior, CORS, body limits, request ID, rate limit, WebSocket hoặc handler signatures nếu không có phê duyệt riêng.
- Giữ `net/http.Handler` và middleware hiện có; không chuyển sang Echo/Gin/Fiber context.
- Không xóa `ServeMux` trước khi route parity và negative authorization tests pass.
- Không dùng prefix string để quyết định quyền truy cập nếu route group có thể thể hiện trực tiếp policy.
- Dependency mới phải được kiểm tra license, maintenance, CVE, transitive graph và pin trong `apiserver/go.mod`/`go.sum`.
- Mọi thay đổi giữ nguyên working-tree changes hiện có; không chỉnh các file ngoài scope.

## Tasks

- [x] **1. Chụp baseline và route inventory** — lập danh sách toàn bộ route từ `apiserver/main.go`, phân loại public/authenticated/system/WebSocket, middleware order và handler contract; ghi test impact matrix cho success, 404/405, malformed path ID, 401/403, CORS, body limit, rate limit, request ID, streaming/WebSocket.
  **Verify:** inventory khớp số route hiện tại; baseline `./leedevkit test api` được chạy và log phase đầy đủ.

- [x] **2. Audit và ghi ADR quyết định router** — so sánh giữ `ServeMux` + module hóa với `chi`, ghi lý do chọn `chi` hoặc dừng migration; gồm security boundary, operability, performance, dependency/license/CVE, migration, rollback, owner và removal criteria.
  **Verify:** ADR nằm trong `docs/adr/`, có trạng thái và acceptance criteria; không code trước khi quyết định được approve.

- [x] **3. Tạo route-module contract** — route modules own registration only; dependency construction remains in `apiserver/routes.go`, passed through typed handler values; modules do not initialize DB/Redis/services.
  **Verify:** each module has explicit domain scope; `main.go` only bootstraps dependencies, builds router and starts server.

- [x] **4. Tạo characterization/contract tests trước migration** — bảo vệ route matching, path parameters, 404/405, public route bypass, authenticated route 401, forbidden route 403, system boundary, CORS preflight, body limit, request ID, rate limit và WebSocket upgrade; không assert implementation detail của router.
  **Verify:** test fail khi intentionally đổi route/middleware contract; focused tests pass trên baseline.

- [x] **5. Chạy migration spike trên một domain nhỏ** — chuyển một domain ít rủi ro sang `chi`, giữ handler/middleware `net/http`, đo số file/context cần đọc và kiểm tra route parity với baseline.
  **Verify:** test parity pass cho domain spike; không có thay đổi observable; nếu chi không giảm complexity hoặc tạo regression thì rollback spike và giữ ServeMux + module hóa.

- [x] **6. Migrate theo nhóm route độc lập** — chuyển public/auth, employee/attendance, leave/overtime, system/monitor và remaining routes vào domain modules; dùng protected router cho policy auth, không dùng global auth bypass prefix; không trộn business logic.
  **Progress:** all current registrations moved from `apiserver/routes.go` into domain modules; dependency construction remains centralized.
  **Verify:** full `./leedevkit test api` passes; no direct route registrations remain in `routes.go`; WebSocket and path parameter behavior remain covered.

- [x] **7. Xóa compatibility code và cập nhật docs** — xóa registration cũ khỏi `routes.go`; giữ `registerRoute` adapter vì existing handlers use `Request.PathValue`; cập nhật `apiserver/README.md`, ADR và route ownership notes.
  **Verify:** `routes.go` có zero direct route registrations; adapter tests and full API suite pass.

- [x] **8. Verification cuối** — chạy focused checks và full `./leedevkit test api`; đọc phase logs và review route diff.
  **Verify:** unit, integration, format và vet phases đều PASS; route modules have no direct dependency construction.

## Done When

- [x] ADR được approve trước implementation.
- [x] All route registration moved to domain modules.
- [x] Full API test suite passes.
- [ ] Security/dependency audit evidence added.
- [ ] Route modules tách khỏi `main.go`, với ownership và middleware scope rõ.
- [ ] Public HTTP/API contract không đổi.
- [ ] Auth, RBAC, tenant isolation, CORS, body limits, rate limit, request ID và WebSocket tests pass.
- [ ] Không còn `ServeMux` registration cũ hoặc dead compatibility code.
- [ ] `./leedevkit test api` full pass; dependency/security checks có evidence.
- [ ] Rollback về commit trước migration vẫn xác định được và không cần data migration.

## Review Focus

- Route bị đặt nhầm dưới public/auth/system group.
- Method/path precedence, trailing slash, 404/405 thay đổi.
- WebSocket upgrade, streaming và `http.Flusher`/`http.Hijacker` behavior.
- CORS preflight và middleware ordering.
- Dependency supply-chain và chi lock-in.
- AI agent conventions có đủ rõ để tránh route registration quay lại `main.go`.

## Notes

Phương án lười hơn vẫn hợp lệ: module hóa route trên `net/http.ServeMux` trước, không thêm dependency. Nếu spike không chứng minh `chi` giảm context hoặc lỗi thực tế, dừng tại phương án này thay vì migration toàn bộ.
