# Quản trị mức lương & phụ cấp tập trung

Status: Approved
Updated: 2026-11-12

## Mục tiêu và quyết định

- Thêm **Nhân sự → Lương & phụ cấp** tại `/salary-management`; `/payroll` tiếp tục là bảng lương theo kỳ. Không xây lại payroll engine.
- Bảng có mã lao động, họ tên, pháp nhân, đơn vị, mức lương và các khoản phụ cấp theo **ngày xem hiệu lực**. Mỗi dòng giữ employment context riêng; không gộp nhiều pháp nhân hoặc đợt tái tuyển dụng. ID hiện tại vẫn là segment ID, không tráo thành relationship ID.
- Mặc định hôm nay theo múi giờ tổ chức, lao động đang làm việc; tìm mã/tên, lọc pháp nhân/đơn vị/trạng thái/chưa cấu hình, phân trang server, giữ bộ lọc trên URL. Chỉ dùng định danh và trạng thái trong URL, không số tiền hay nội dung nhạy cảm.
- Cột khoản lương chọn được từ danh mục hiện có, cuộn ngang khi cần; không cộng lương tháng, lương ngày/giờ và khoản khác cơ sở thành “tổng thu nhập”. Chi tiết hiển thị đơn vị tính và loại khoản.
- Mở dòng để xem lịch sử; tạo/thay đổi mức trên page riêng `/salary-management/$employmentId/edit`. Có ngày hiệu lực và lý do; lưu xong quay về đúng bộ lọc. Tab hồ sơ thành tóm tắt chỉ đọc + liên kết lọc sẵn, không giữ form ghi thứ hai.
- Đợt này chỉnh từng lao động; **chưa có bulk edit, import/export, tính/chốt/chi trả lương**, không thêm dependency hoặc lưu lương trong localStorage. Bảng tập trung đã giải quyết nhu cầu quản lý nhiều người; bulk bổ sung khi có workflow cụ thể.

## Ranh giới bắt buộc

- **Gate chính sách:** `data-governance.md §2` hiện cấm T2 trong collection API. Đề xuất ngoại lệ hẹp cho bảng lương/phụ cấp: chỉ quyền đọc lương chuyên biệt + phạm vi pháp nhân/đơn vị, trường tối thiểu, audit đọc/ghi không số tiền/PII thô, `Cache-Control: no-store`, không persist query cache. Phải phê duyệt ngoại lệ và cập nhật đúng mục chính sách trước triển khai endpoint số tiền; không tự nới cho danh sách nhân viên khác. Nếu không được duyệt, bảng chỉ hiện trạng thái và số tiền nằm ở chi tiết; không lách bằng N request detail.
- Đối chiếu grants thực tế; tái dùng permission đủ hẹp hoặc bổ sung `salary:read`/`salary:manage` nếu thiếu. Không tự cấp từ `employees:read` hoặc mọi manager; HR/admin chỉ có quyền theo grants được duyệt. Server kiểm quyền + ownership/scope ở mọi endpoint, không tin route guard.
- Phối hợp `hrm-employment-contract-normalization-plan.md`: UI chọn đúng employment tại ngày xem, không lấy active employment hôm nay cho mọi lịch sử. Transfer/rehire/multi-employer phải có contract continuity rõ trước triển khai; không tự copy lương hoặc dựng mô hình relationship song song.
- Mức mới không sửa đè lịch sử; ngày tương lai là “Sắp hiệu lực”, không “Đang áp dụng”. Ngừng hưởng là sự kiện nghiệp vụ riêng, không xóa lịch sử và không giả mức `0`. Không tự tính lại payslip. Guard hồi tố phải dùng trạng thái payroll có thẩm quyền trên server; localStorage hiện tại không phải khóa. Chưa chứng minh guard thì chặn thay đổi hồi tố có thể ảnh hưởng kỳ đã tính, không dựng cả hệ thống khóa kỳ trong phạm vi này.

## Rule IDs

`design-rules.md §1–4`: component sẵn có, accessibility, responsive, loading/empty/error. `api-governance.md §1–3, §6`: contract-first, pagination, tương thích và contract tests. `access-control.md §1–2` + `data-governance.md §1–2`: quyền/scope, T2 và gate ngoại lệ. `database-rules.md §2–4`: không N+1, transaction/integrity, migration rollback. `testing-standards.md §1–4`: test cùng thay đổi, integration và full targets.

## Thực hiện — theo thứ tự

1. [ ] **Baseline và contract.** Rà current working tree, grants, schema, consumers tính lương và normalization đang làm; không ghi đè thay đổi khác. Chốt context employee/employer/relationship/segment, đơn vị tiền/tính, chính sách T2 và hồi tố. **Đạt:** permission/scope matrix, Test Impact Matrix và API schema được ghi trước code; blocker không bị che bằng giả định.
2. [ ] **API đọc tập trung.** Thêm `GET /api/v1/salary-management` có `as_of`, filters, pagination và stable ordering; response phân biệt chưa cấu hình, hiện hành, có thay đổi tương lai. Dùng JOIN/batch theo trang, không fetch toàn bộ nhân viên hoặc gọi API từng dòng. Thêm history endpoint phân trang; giữ nguyên contract legacy. **Đạt:** đúng mức tại ngày chọn, scope trước pagination/count, không leak số tiền khi thiếu quyền; audit và no-store được kiểm thử.
3. [ ] **Ghi và lịch sử.** Tái dùng salary-component assignment; bổ sung action thay mức/ngừng hưởng và history contract khi thiếu. Nếu cần schema, thêm migration mới có rollback, không sửa baseline đã áp dụng. Resolver dùng chung cho bảng/chi tiết/payroll, kiểm ngày nằm trong employment, decimal chính xác, component hợp lệ, lý do, xung đột cùng ngày và optimistic concurrency. Transaction khóa chuỗi employee/employment/component để hai người sửa không tạo trạng thái mâu thuẫn; retry không nhân bản. **Đạt:** ngừng hưởng không làm mức cũ sống lại, lịch sử và payslip đã có không đổi; mọi writer legacy/import liên quan không vượt guard.
4. [ ] **Bảng và navigation.** Thêm route/lazy page theo convention; tái dùng bảng, toolbar, filters, combobox và pagination hiện có. Sinh route tree bằng tooling repository, không sửa generated file tay. Phân biệt chưa có khoản, khoản bằng 0 và đã ngừng; lỗi không hiện thành số 0. **Đạt:** direct URL/refresh/Back giữ context, server pagination đúng, quyền đọc/sửa tách riêng.
5. [ ] **Chi tiết và form.** Tái dùng phần hữu ích của `employee-salary-config.tsx`; detail hiển thị mức tại ngày chọn, lịch sử và lịch sắp tới. Form thêm/thay mức/ngừng hưởng có validation, xác nhận tác động, chống submit lặp, cảnh báo rời khi chưa lưu; lỗi giữ dữ liệu. **Đạt:** lưu đọc lại đúng người/employment/ngày, 409 yêu cầu tải lại thay vì ghi đè; vi/en/zh, keyboard/focus, mobile và dark mode đầy đủ.
6. [ ] **Hợp nhất hồ sơ nhân viên.** Thay form tab “Mức lương & Đãi ngộ” bằng tóm tắt và link quản trị; giữ tab lịch sử đãi ngộ có vai trò riêng. Bỏ nút xóa chưa hoạt động và logic lịch sử giả sau xác minh call sites; không bỏ cấu hình ngày/giờ công chuẩn đang nằm cùng component, giữ ở phần cấu hình hiện hữu. **Đạt:** một đường ghi chuẩn, link từ hồ sơ giữ đúng context, chức năng payroll-config cũ không mất.
7. [ ] **Verification cuối.** Unit/contract rồi PostgreSQL integration cho as-of, future/stop/zero, trùng ngày, concurrency/retry, transfer/rehire/multi-employer, không employment, ngoài scope, 401/403/404/409 và rollback không side effect. Chạy `./leedevkit test api`, `./leedevkit test web`, `./leedevkit test all`, `git diff --check`; đọc từng phase/log, không tin riêng summary xanh. Trước browser đọc account/audit rules, lập route × role matrix cho năm vai trò trên các route bị tác động; kiểm allow/deny, submit rồi reload, desktop/mobile, console/network bằng dữ liệu giả. **Đạt:** từng gate có PASS/FAIL/SKIP và lý do; không ghi credentials/token/PII trong artifacts.

## Nghiệm thu

HR có quyền xem nhiều lao động trong một bảng, chọn ngày đúng lịch sử, thay mức/ngừng phụ cấp từng người và đọc lại sau refresh; không mất context hoặc cấu hình cũ. Không rò số tiền qua API/cache/URL, không ghi nhầm employment, không thay payslip lịch sử. Triển khai chỉ bắt đầu sau duyệt kế hoạch, bao gồm quyết định ngoại lệ T2; chưa duyệt thì không sửa chính sách hoặc mã nghiệp vụ.
