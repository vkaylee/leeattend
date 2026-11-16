# Chuyển form modal sang page

Status: Awaiting approval

## Phạm vi và mặc định đề xuất
- Bắt buộc: hợp đồng **tạo/sửa**, nghỉ phép **tạo đơn**, điều chỉnh chấm công **tạo yêu cầu**. Không thêm chức năng sửa đơn chưa tồn tại.
- Nhân viên: tùy chọn, chưa được phê duyệt; không nằm trong đợt này. Giữ dialog xác nhận, duyệt/từ chối, import và drawer xem chi tiết.
- Lưu thành công/Hủy quay về nơi mở form, giữ search/tab/bộ lọc/phân trang nếu màn nguồn có; truy cập trực tiếp có đường về mặc định. Không dùng `history.back()` làm phương án duy nhất.
- Tái sử dụng form/schema/hook và API hiện có; không thêm dependency, wizard, lưu nháp hoặc localStorage chứa dữ liệu form. Không sửa `.leedevkit/`, không ghi đè thay đổi chưa commit.

## Rule compliance
- `design-rules.md §2–4`: keyboard/focus/label, responsive, loading/error/empty states.
- `coding-standards.md §2, §4, §6`: bỏ wrapper chết sau chuyển đổi, bảo vệ PII, i18n.
- `access-control.md §1–2`: route/action guard không thay thế authorization server; fail closed.
- `development-workflow.md §2–3` + `testing-standards.md §1–4`: Test Impact Matrix, test cùng thay đổi, chạy target đầy đủ gồm integration/E2E.

## Route và bố cục đề xuất
| URL mới | Hành vi / điểm mở |
|---|---|
| `/employees/$employeeId/contracts/new` | Tạo từ tab hợp đồng trong hồ sơ hoặc danh sách `/econtracts` sau chọn nhân viên |
| `/employees/$employeeId/contracts/$contractId/edit` | Sửa từ cả hai nơi; tải lại bản ghi bằng ID khi refresh/mở trực tiếp |
| `/leave-requests/new` | Đăng ký cá nhân/đăng ký thay theo quyền và phạm vi hiện có |
| `/attendance-adjustments/new` | Tạo yêu cầu với ngày/giờ, lý do và minh chứng hiện có |

Page dùng chiều rộng có giới hạn, một cột trên mobile; hai cột chỉ khi trường phù hợp. Hợp đồng chia thông tin/thời hạn/cột mốc/ký điện tử; nghỉ phép chia đối tượng/thời gian/lý do/minh chứng; điều chỉnh chia đối tượng/mốc chấm công/lý do/minh chứng. Header có Quay lại; Lưu/Hủy dễ tìm, không che field/error/focus. Dùng form submit chuẩn, chặn gửi lặp.

## Công việc (thực hiện sau phê duyệt)
1. [ ] **Chốt baseline và quyền thực tế.** Đối chiếu current tree và test trước sửa; lập matrix self/on-behalf/contracts theo permission + assignment scope, cả API lẫn UI. Không suy quyền hợp đồng từ endpoint tạo nhân viên: `contracts_employments.go:223–269` hiện kiểm tra membership trong handler, cần đối chiếu middleware toàn tuyến. Nếu server thiếu bảo vệ, ghi rõ blocker và xin duyệt phạm vi bảo mật; không tuyên bố route guard đã bảo vệ API. **Đạt:** bảng allowed/denied và baseline lỗi có bằng chứng.
2. [ ] **Thêm route/navigation.** Theo cặp route + lazy hiện có; dùng file-route escaping khi cần để form không bị lồng dưới list/detail thiếu `Outlet`; sinh lại `route-tree.gen.ts` bằng wrapper đã có, không sửa tay. Validate UUID/search, feature flag, auth/action guard; loading/403/404/retry rõ. Return context chỉ cho phép các route nguồn định sẵn, không nhận URL chuyển hướng tùy ý; không đưa PII/lý do/nội dung form vào URL. **Đạt:** mở trực tiếp, refresh và route collision đều có test.
3. [ ] **Chuyển hợp đồng trước.** Tách phần form/mutation khỏi `components/features/employees/contract-form-modal.tsx`; tái dùng `contract-form-schema.ts`, `EsignSection`, `useEmployeeContracts`. Thay call sites tại `employee-contracts-tab.tsx` và `pages/econtracts-page.tsx`. Giữ chọn nhân viên gọn từ registry; đưa bộ lọc type/status hiện nằm local state vào search schema để không mất khi quay về. Preserve employment/cột mốc, thời hạn, ký điện tử và payload; xác nhận trùng hợp đồng vẫn là dialog, chỉ tự đóng hợp đồng cũ khi người dùng xác nhận. **Đạt:** tạo/sửa từ cả hai nguồn, direct link và overlap success/failure có test; không tự ghi thêm khi hủy.
4. [ ] **Chuyển nghỉ phép.** Tái dùng `create-leave-request-form.tsx`; chuyển state/mutation từ `create-leave-request-modal.tsx` sang page hoặc hook tối thiểu khi cần. Đổi nút trong `pages/leave-requests-page.tsx`; giữ leave types/balance, nhánh ngày/giờ, attachments, self/on-behalf và query invalidation. **Đạt:** đơn cá nhân và đăng ký thay dùng đúng đối tượng/phạm vi; lỗi giữ dữ liệu; quay về giữ `tab/status/employee`.
5. [ ] **Chuyển điều chỉnh chấm công.** Di chuyển state/mutation từ `create-adjustment-modal.tsx`; đổi nút trong `pages/attendance-adjustments-page.tsx`, không trộn với hook danh sách/duyệt. Preserve timezone, upload PDF, attachments, query invalidation attendance-adjustments/time-history. **Đạt:** đúng ngày/giờ, file, đối tượng; upload/submit lỗi không mất form; quay về giữ `tab/status/employee`.
6. [ ] **Bảo vệ dữ liệu và hoàn thiện UX.** Tái dùng pattern `useBlocker` + `ConfirmDialog` ở `assignment-rule-editor-page.tsx`, dùng cảnh báo native khi reload/đóng tab. Dirty gồm file/chọn thời hạn; query refetch không reset dữ liệu đã sửa. Success reset dirty trước navigation; failure không reset. Pending tránh submit lặp/rời trang gây kết quả mơ hồ. i18n vi/en/zh, label/error/focus đúng; focus heading sau navigation, field lỗi đầu tiên sau validation. **Đạt:** Back/Hủy/sidebar/reload, Stay/Leave và lưu thành công không hỏi nhầm đều có test.
7. [ ] **Dọn implementation cũ và cập nhật test.** Bỏ ba vỏ modal/state/import không còn dùng sau xác minh toàn bộ call sites; giữ form dùng lại và dialog xác nhận. Chuyển test modal sang form/page/route; cập nhật `contract-form-modal.test.tsx`, `employee-contracts-tab.test.tsx`, `econtracts-page.test.tsx`, `leave-requests-page.test.tsx`, `attendance-adjustments-page.test.tsx` theo test thực tế tìm được; bổ sung regression cho navigation/dirty/direct-link. **Đạt:** không còn nút mở modal cũ cho ba workflow, không mất coverage nghiệp vụ.
8. [ ] **Verification cuối.** Chạy focused test khi phát triển, sau đó `./leedevkit test web </dev/null` không cờ skip/unit-only; đọc kết quả từng phase và `.test_logs/`, không dựa riêng dòng xanh cuối. Nếu sửa API được duyệt thêm, chạy `./leedevkit test all </dev/null`. Bổ sung integration/E2E submit rồi đọc lại bản ghi bằng dữ liệu giả, desktop/mobile, mỗi route mới có allowed/denied, console/network evidence. Trước browser đọc `docs/dev-accounts.md` và lập route × role matrix; chọn công cụ được phép cho từng role, không vượt giới hạn employee-only của phiên Playwright MCP. **Đạt:** mọi gate áp dụng pass; phần bị chặn/skip ghi unverified, không báo hoàn tất.

## Test Impact Matrix / điều kiện nghiệm thu
- **Changed/existing:** mở page thay modal; payload, validation, upload, toast, invalidation và workflow duyệt vẫn giữ nguyên.
- **Input/state:** required/range/UUID sai, hết hạn quyền, không có employment/cột mốc/assignment, loading/empty/stale refetch; ID hợp đồng không thuộc nhân viên phải từ chối, không hydrate nhầm bản ghi.
- **Failures/security:** 401/403/404/409/5xx, upload lỗi, submit lỗi, duplicate click; server quyết định quyền và scope, client guard fail closed; return context giả mạo bị bỏ. Không tự retry mutation có nguy cơ tạo trùng.
- **Compatibility/operations:** danh sách/chi tiết và link cũ không đổi; route mới qua router generation/build/type/lint gate; không log PII hoặc lưu draft/token trong artifact. Dùng dữ liệu giả cho test, không credential trong trace/screenshot.
- **Done:** 3 workflow không còn dùng modal nhập liệu; direct URL/refresh, giữ ngữ cảnh, unsaved warning, vi/en/zh, keyboard/mobile và integration/E2E đã xác minh. Backend/schema migration không nằm trong phạm vi mặc định.

Approval pending: phê duyệt 3 nhóm và các mặc định quay về nơi mở form/không lưu nháp; Nhân viên cần duyệt riêng.
