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

- [ ] **3. Tạo route-module contract** — thiết kế package boundary và dependency object tối thiểu, ví dụ `internal/httpserver/router.go`, `routes_public.go`, `routes_auth.go`, `routes_employees.go`, `routes_attendance.go`, `routes_system.go`, `routes_monitor.go`; mỗi module chỉ đăng ký route thuộc domain, không khởi tạo DB/Redis/service.  
  **Verify:** mỗi module có owner/scope rõ; `main.go` chỉ bootstrap dependencies, build router và start server.

- [x] **4. Tạo characterization/contract tests trước migration** — bảo vệ route matching, path parameters, 404/405, public route bypass, authenticated route 401, forbidden route 403, system boundary, CORS preflight, body limit, request ID, rate limit và WebSocket upgrade; không assert implementation detail của router.  
  **Verify:** test fail khi intentionally đổi route/middleware contract; focused tests pass trên baseline.

- [x] **5. Chạy migration spike trên một domain nhỏ** — chuyển một domain ít rủi ro sang `chi`, giữ handler/middleware `net/http`, đo số file/context cần đọc và kiểm tra route parity với baseline.  
  **Verify:** test parity pass cho domain spike; không có thay đổi observable; nếu chi không giảm complexity hoặc tạo regression thì rollback spike và giữ ServeMux + module hóa.

- [ ] **6. Migrate theo nhóm route độc lập** — chuyển lần lượt public/auth, employee/attendance, leave/overtime, system/monitor và remaining routes; dùng `chi.Route`/`Group` để gắn middleware theo policy, không dùng global auth bypass prefix; mỗi nhóm là commit/diff nhỏ, không trộn business logic.  
  **Progress:** public/protected topology migrated in `apiserver/routes.go`; domain files remain pending.  
  **Verify:** focused contract tests pass sau từng nhóm; route inventory không còn thiếu hoặc duplicate; WebSocket, upload/streaming và path parameter behavior giữ nguyên.

- [ ] **7. Xóa compatibility code và cập nhật docs** — chỉ sau parity pass, xóa registration cũ, obsolete helpers/tests/dependency; cập nhật `apiserver/README.md`, ADR, route ownership notes và agent-facing conventions để agent biết file nào sở hữu domain nào.  
  **Verify:** grep không còn route registration cũ hoặc unused router adapter; `go mod tidy`/dependency diff không để dead dependency; docs khớp deployed route tree.

- [ ] **8. Verification cuối** — chạy security/dependency/license checks phù hợp, focused tests, sau đó bắt buộc full `./leedevkit test api`; đọc phase logs, kiểm tra route parity report và review diff độc lập.  
  **Verify:** test target pass không chỉ dựa vào dòng summary; báo rõ pass/fail/skip/warning, remaining risks và rollback readiness.

## Done When

- [ ] ADR được approve trước implementation.
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
