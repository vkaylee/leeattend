# Contract UI Separation and Employment Context

Status: Approved
Updated: 2026-11-12

## Goal

Tách đúng UI hợp đồng pháp lý thường khỏi UI e-contract; giữ compatibility cho URL cũ; hiển thị contract theo employee và employment relationship mà không giả lập ký điện tử.

## Evidence hiện tại

- `/econtracts` đang render ordinary contract registry trong `webdashboard/src/pages/econtracts-page.tsx`.
- Route `/econtracts` và settings `/settings/econtracts` đang bị gate bởi `ff_econtracts`.
- Chưa có route `/contracts`.
- `EmployeeContractsTab` đang hiển thị ordinary contracts nhưng còn import/query e-contract placeholder và link registry tới `/econtracts`.
- Ordinary contract API đã có list/create/update/delete; e-contract API/backend chưa wired.

## Constraints và Rule IDs

- Không dùng `ff_econtracts` để ẩn ordinary contract registry.
- `/contracts` = ordinary legal contract registry; `/econtracts` = envelope/signature workflow; `/settings/econtracts` = provider/configuration.
- Không tạo nút ký giả, trạng thái signed giả, download giả, hoặc gọi provider khi backend chưa wired.
- Giữ UUID, `employment_id`, API legacy và bookmark cũ; redirect phải rõ ràng, không remap ID.
- Mọi mutation vẫn qua permission/backend; UI không dùng localStorage làm authority.
- Rule IDs: `design-rules.md §1–3`; `coding-standards.md §6, §9`; `access-control.md §1–3`; `development-workflow.md §2–7`; `testing-standards.md §1–4`.

## Tasks

- [ ] **1. Chốt route và permission contract.** Xác minh permission thực tế ở contract handler, app registry và route guard; thêm canonical `ROUTES.CONTRACTS = "/contracts"`; tạo route/search schema cho `q`, employee filter và relationship context. Verify: route typecheck; unauthorized direct access redirects/403 đúng; không còn ordinary page phụ thuộc `ff_econtracts`.

- [ ] **2. Di chuyển registry sang `/contracts`.** Đổi tên semantic page/component từ e-contract sang contracts hoặc giữ file tạm thời với tên export rõ nghĩa; tái sử dụng table/filter/form hiện có; thay text “digital/e-sign/sign” bằng legal contract wording; hiển thị employee, contract number, type, status, effective range và employment context nếu API đã trả về. Verify: list/filter/create/edit/delete và refresh/back giữ filter.

- [ ] **3. Giữ compatibility URL.** `/econtracts` không tiếp tục render ordinary registry. Trong giai đoạn e-contract backend chưa sẵn sàng, route cũ xử lý redirect có kiểm soát tới `/contracts` hoặc trang placeholder theo permission; không redirect mù qua auth boundary. Cập nhật app launcher, navigation, breadcrumb và route metadata. Verify: bookmark cũ không 404; `/contracts` không bị feature flag e-contract; route tests cover flag on/off.

- [ ] **4. Làm sạch employee contract tab.** Giữ tab `Contracts` là ordinary contracts thuộc employee; link “View in registry” tới `/contracts?q=<employee_id>`; bỏ envelope query, employer fetch không dùng, drawer/import dead code và wording e-contract. Giữ edit/delete permission gate, confirm dialog, loading/empty/error states. Verify: employee context đúng; wrong employee không lẫn dữ liệu; keyboard/focus/aria labels hoạt động.

- [ ] **5. Tách e-contract UI riêng.** `/econtracts` chỉ nhận envelope workflow; `/settings/econtracts` chỉ cấu hình. Khi backend chưa wired, giữ feature flag OFF và placeholder trung thực; không đưa ordinary contract API vào đây. Khi API envelope được duyệt, thêm page/table/detail drawer theo envelope ID, status, audit timeline và retry/error state; không xây provider integration trong slice này.

- [ ] **6. Chuẩn hóa UX/i18n/a11y.** Di chuyển hardcoded title/subtitle/action/filter/status strings sang locale keys; dùng UI components hiện có; bổ sung skeleton, empty, error/retry, delete pending/disabled, responsive table/mobile view, visible focus, labels cho icon-only actions và modal focus restore. Verify: locale hiện có không lộ key; WCAG keyboard/contrast checks trong component tests.

- [ ] **7. Test impact và verification.** Cập nhật page, route guard, app registry, employee tab, API-hook tests: success, filter/empty, API error, unauthorized, old URL compatibility, mutation failure, stale query invalidation và feature-flag isolation. Chạy `./leedevkit test web`, đọc toàn bộ phase logs; sau đó `./leedevkit test all` nếu route/shared infrastructure thay đổi. Trước browser audit: đọc `docs/dev-accounts.md`, lập route × role matrix cho 5 role và ghi console/network evidence.

## Done when

- `/contracts` là canonical ordinary contract UI, không phụ thuộc `ff_econtracts`.
- `/econtracts` không còn hiển thị ordinary contracts.
- Employee contract tab và registry giữ đúng employee/employment context.
- Không có fake signing/provider behavior.
- Loading/empty/error/i18n/a11y và permission behavior có test.
- Full web verification và relevant all-target evidence được ghi rõ PASS/FAIL/SKIP.

## Deliberate simplification

Không tạo global contract state/store mới; dùng TanStack Query và API hooks hiện có. Chỉ thêm abstraction khi route separation hoặc envelope API thật sự cần.
