# UI route audit & fix — 2026-11-11

Status: Approved

## Mục tiêu và giới hạn
Hoàn tất audit route × 5 role, sửa lỗi được tái hiện; không mở rộng quyền chỉ để hết `403`. Chưa triển khai source trong bước lập plan. Giữ nguyên thay đổi có sẵn, gồm `AGENTS.md` và hai test Playwright watchdog/endpoint trong `scripts/tests/`. Không sửa `.leedevkit/`.

Rule IDs: `AGENTS.md` Development Test Accounts; `ui-route-audit.md` Mandatory Workflow; `access-control.md` §1 Default Deny; `development-workflow.md` §2–3; `testing-standards.md` §1–2.

## Evidence hiện có — không coi là audit hoàn tất
- Browser đã ghi nhận quota `404` tại `/ess/ot`; documents placeholder được xác nhận bằng source.
- Admin dashboard nhận `403: No employee record linked to your account` từ ESS time entries; HR `/employees` từng nhận `/locations` `403`. Cần tái hiện riêng từng route sau khi tải xong.
- Employee management APIs trả `403` có thể đúng authorization; repeated toast/query hoặc navigation sai mới là lỗi UX cần xác nhận.
- `/onboarding` và route-level guards là mục điều tra, chưa đủ evidence để kết luận defect.
- Sweep chờ cố định 350–800 ms và browser context đóng làm kết quả thiếu hoặc lệch route. Không đánh dấu PASS từ URL, không có console error, hoặc chỉ `/me` `200`. Browser đóng chưa chứng minh app crash.

## Công việc theo thứ tự
1. [ ] **Ổn định thu evidence:** kiểm tra lifecycle/idle watchdog MCP đang có và thay đổi chưa commit; dùng batch nhỏ, dừng ngay khi context đóng/session hết hạn. Chờ page-specific ready/error/empty state và API cần thiết, không dùng sleep làm điều kiện pass. Verify: cùng route chạy lặp có evidence đúng route, không lẫn request trang trước.
2. [ ] **Hoàn tất matrix:** lấy unique URL patterns từ generated route tree, gộp layout/index/lazy trùng URL nhưng giữ aliases/public routes. Bao phủ `employee`, `team_lead`, `shift_manager`, `hr_manager`, `admin`; expected access dựa permission thực và scope dữ liệu. Dynamic routes dùng record hợp lệ lấy từ UI; missing/malformed ID là negative cases riêng. Verify: mỗi ô có PASS/DEFECT/EXPECTED-DENY/BLOCKED và lý do; không ô trống.
3. [ ] **Audit từng role qua login UI:** ghi route/final URL, visible state, console warning/error, API status, request failure; đối chiếu sidebar, launcher, command palette và direct URL. Chỉ mở form/filter/tab, không submit punch/approval/create/delete trong audit đọc. Verify: không bypass quyền, không ghi credential/token/PII; role/tool bị chặn được báo BLOCKED, không thay bằng role khác.
4. [ ] **P1 — Quota OT:** đối chiếu `lib/api/overtime-requests.ts`, `hooks/use-overtime-requests.ts`, `pages/ess/ot-page.tsx` với backend route/OpenAPI. Tái hiện `404`, phân biệt missing endpoint và missing resource. Sửa URL/contract hiện có trước; nếu feature chưa hỗ trợ, thể hiện unavailable rõ ràng thay vì tự xây quota subsystem. Verify: quota hợp lệ, rỗng, forbidden và server error có UI đúng, không giấu lỗi như thành công.
5. [ ] **P1 — ESS và dependency quyền:** kiểm tra Quick Punch/useMyClock với user không có employee link; render unavailable và ngừng query/action không hợp lệ, giữ backend deny. Tái hiện HR `/employees` dependency `/locations`; chỉ dùng quyền đọc đã được policy cho phép hoặc bỏ query không cần thiết. Verify: employee clock vẫn chạy; admin không-linked không query lặp; HR không mất chức năng; role thiếu quyền vẫn bị chặn.
6. [ ] **P1 — Navigation/authorization:** chỉ sửa những mismatch matrix xác nhận; tái dùng guard hiện có, không tạo permission framework mới. Kiểm tra consumers `/onboarding` trước khi sửa redirect hoặc bỏ dead reference. Verify: allowed route hoạt động; denied route có forbidden/redirect rõ, không query/toast lặp; aliases giữ đúng params/search.
7. [ ] **P2 — Documents và warning:** ưu tiên tái dùng `useMyDocuments`/API hiện có cho `/ess/documents`, có i18n, loading/empty/error và accessibility; không thêm acknowledgement workflow ngoài phạm vi. Kiểm tra dependency/import Devtools trước khi migrate warning. Verify: documents API thật được gọi; lỗi không thành empty success; không còn warning đã xử lý.
8. [ ] **Verification cuối:** thêm regression test fail trước fix, chạy focused checks rồi `./leedevkit test web`; nếu đổi backend/contract chạy thêm `./leedevkit test api` hoặc `./leedevkit test all` cho shared changes. Đọc từng phase/log, báo fail/skip/warning đúng; rerun toàn bộ matrix 5 role và xuất report theo role/route, gồm mọi route BLOCKED.

## Test impact & tiêu chí hoàn tất
Bao phủ success, loading/empty, user không-linked, ID thiếu/sai/không tồn tại, `401`/`403`/`404`/`5xx`, timeout, retry và session expiry; giữ least privilege, scope dữ liệu và API compatibility. Persistence/concurrency/migration chỉ áp dụng nếu fix thực sự chạm các phần đó. Không test mutation dữ liệu thật nếu chưa được phép.

Hoàn tất khi mọi route × role có evidence đáng tin, mọi defect được sửa hoặc ghi rõ chưa xử lý, focused + full applicable suites đạt và browser regression đạt. Còn BLOCKED thì audit vẫn chưa hoàn tất.
