# Employee Profile → Table-First Personnel Pages

Status: In progress — Phase 1 implemented

## Mục tiêu và phạm vi đề xuất
Giảm tab nghiệp vụ trong hồ sơ; quản lý nhiều nhân viên trên trang bảng tập trung, vẫn tra cứu một người bằng liên kết lọc sẵn. Phase 1 đã thêm liên kết và bộ lọc URL, chưa bỏ tab hoặc đổi API. Kế hoạch bổ sung `hrm-table-migration-plan.md`, không thay thế kế hoạch đó hoặc ghi đè thay đổi chưa commit.

**Ba mặc định cần duyệt:** (1) bảng tổng hợp nhiều nhân viên, không phải trang riêng từng người; (2) làm trước Phái cử, Điều kiện làm việc, Vòng đời; (3) profile giữ tóm tắt + “Xem tất cả”, chỉ bỏ tab khi trang thay thế đã đủ chức năng và phân quyền.

## Rule compliance
- `PROFILE-UI-01` — `.agent/rules/design-rules.md` §§1–4: dùng UI có sẵn; WCAG AA, keyboard/focus, skeleton, empty/error/retry, mobile.
- `PROFILE-AUTH-02` — `.agent/rules/access-control.md` §§1–2: permission và phạm vi dữ liệu kiểm tra ở API; route/menu/nút không tự cấp quyền.
- `PROFILE-DATA-03` — `.agent/rules/data-governance.md` §§1–2: tên/mã nhân viên, phòng ban, ngày và trạng thái là T3 theo nghiệp vụ; lương, liên hệ, lý do y tế là T2, không trả trong collection; không đưa PII vào URL/log/artifact.
- `PROFILE-API-04` — `.agent/rules/api-governance.md` §§1–3: contract-first, tương thích ngược; collection có giới hạn/phân trang và thứ tự ổn định.
- `PROFILE-TEST-05` — `.agent/rules/testing-standards.md` §§1–4: test hành vi + integration/E2E; dùng wrapper, đọc kết quả từng phase, không kết luận từ dòng tổng kết xanh.

## Đích đến từng tab
| Tab | Đợt / cách xử lý | Cột bảng tối thiểu, không chứa T2 |
|---|---|---|
| Thông tin cá nhân | Giữ profile | Không tạo bảng mới |
| Tài liệu | Giữ profile; quản lý toàn công ty ngoài đợt đầu | Chưa mở rộng phạm vi |
| Phái cử / Luân chuyển | Đợt 1: tái dùng trang và bảng hiện có | Nhân viên, nơi đi/đến, thời gian, trạng thái, thao tác |
| Điều kiện làm việc | Đợt 1: tái dùng trang và bảng hiện có | Nhân viên, loại điều kiện, hiệu lực, trạng thái, thao tác; không hiện lý do y tế |
| Vòng đời nhân sự | Đợt 1: tái dùng trang và bảng hiện có | Nhân viên, đợt làm việc, quy trình, tiến độ, trạng thái, thao tác |
| Hợp đồng | Đợt 2, chỉ sau đối chiếu hợp đồng thường và điện tử | Nhân viên, số/loại hợp đồng được phép hiển thị, ngày hiệu lực/hết hạn, trạng thái; không số tiền |
| Giai đoạn làm việc | Đợt 2: trang danh sách riêng nếu chưa có; không gộp với checklist vòng đời | Nhân viên, đợt làm việc, giai đoạn, bắt đầu/kết thúc, trạng thái |
| Chính sách chấm công | Đợt 2: ưu tiên trang phân bổ chính sách hiện có nếu phù hợp | Nhân viên, chính sách, nguồn áp dụng, hiệu lực, trạng thái |
| Lương + Lịch sử thay đổi lương | Giữ chi tiết có phân quyền; chỉ gộp giao diện khi được duyệt riêng | Không xây bảng mức lương nhiều người trái chính sách T2 |

## Route × role acceptance matrix
| Route | employee | team_lead | shift_manager | hr_manager | admin |
|---|---|---|---|---|---|
| `/assignments?employeeId=<uuid>` | Expected deny/redirect unless explicit `employees:read`; no raw collection data | Expected deny/redirect unless scoped read is granted | Expected read only within approved scope | Expected read and employee filter | Expected read and employee filter |
| `/working-conditions?employeeId=<uuid>` | Expected deny/redirect unless explicit `employees:read`; no raw collection data | Expected deny/redirect unless scoped read is granted | Expected deny/redirect unless explicit grant | Expected read and employee filter; condition-type lookup may return expected `403` under current catalog | Expected read and employee filter |
| `/lifecycle?employeeId=<uuid>` | Expected deny/redirect unless explicit `lifecycle:read`/employee read | Expected deny/redirect unless scoped read is granted | Expected deny/redirect unless explicit grant | Expected read and employee filter | Expected read and employee filter |
| Evidence per route | URL, redirect/403, console, network, empty/error/table state | Pending authorized browser session | Pending authorized browser session | Partially checked in prior session | Pending authorized browser session |

Playwright session policy permits only `employee`; other roles remain untested until a permitted session is supplied. Expected `403`/redirect is authorization behavior; `404`, `5xx`, crashes, blank page, repeated unexpected errors, or wrong navigation are findings.

## Hiện trạng đã khảo sát và khoảng trống
- Profile có 10 tab dùng chung page/drawer (`components/features/employees/drawer-header.tsx:27`, `employee-detail-drawer.tsx:56`); test page đang khóa kỳ vọng 10 tab (`pages/__tests__/employee-detail-page.test.tsx:146`). Các đường dẫn dưới đây tương đối với `webdashboard/src/`.
- Trang đích đợt 1 là `/assignments`, `/working-conditions`, `/lifecycle`; tái dùng `pages/personnel/*-page.tsx` và `components/features/personnel/*-table.tsx`. Hiện trang tải danh sách và lọc phía client; frontend client chưa truyền filter/phân trang vào organization collection. Phải đối chiếu handler Go và contract thật trước khi chọn thay đổi API; chưa coi việc đổi điều hướng là đủ production-ready.
- Filter `employeeId` chưa là contract chung của ba trang; `/assignments` mới có `q`. Read/action guards tập trung cần audit; không coi đăng nhập hoặc ẩn nút là đủ quyền truy cập.
- Hợp đồng thường đã có client collection `/api/v1/contracts` trong `lib/api/contracts.ts`; khác với e-signature. `lib/api/econtracts.ts:1–8,72–76` đang dùng stub trả rỗng, nên `/econtracts` không được coi là trang thay thế đủ chức năng. Không đưa việc nối e-signature provider vào nhiệm vụ này.
- Phase có API theo employee/employment trong `lib/api/employee-phases.ts`; khác checklist onboarding/offboarding của lifecycle. Chính sách chấm công có organization assignment API và `components/features/attendance/policy-assignments-section.tsx` để đối chiếu tái dùng; chưa xác nhận cần route mới.
- Salary/compensation hiện theo employee/employment, không tạo N+1 fan-out để làm bảng tổng hợp. `/documents` quản lý folder/file/distribution, không mặc định thay thế hồ sơ tài liệu nhân viên.

## Contract UI và dữ liệu
- Trang bảng là nơi thao tác nghiệp vụ; tên nhân viên mở hồ sơ, thao tác hàng mở detail/form hiện có. Không thêm bulk edit, export, dashboard/KPI hoặc dependency mới.
- Profile giữ tóm tắt nghiệp vụ được phép xem + “Xem tất cả”; tái dùng dữ liệu có sẵn hoặc truy vấn có giới hạn khi cần, không tải mọi danh sách chỉ để tính số lượng. Thiếu quyền thì không fetch/hiện nội dung hoặc liên kết.
- Bộ lọc URL đề xuất: `employeeId`, `employmentId` nếu domain hỗ trợ, `status` và ngày theo contract hiện có. Tên query phải chốt theo route thực tế, không đổi đồng loạt API chỉ để đồng nhất cách đặt tên. UUID xác thực; tên/email/lương/lý do không vào URL.
- Lọc một người có chip bỏ lọc và nhãn rõ phạm vi; reload/copy URL/back-forward giữ đúng state, đổi filter reset trang/cursor. Query key gồm toàn bộ filter và phạm vi phiên; mutation invalidate cả list/detail/summary liên quan.
- Không gom nhiều request từng nhân viên để dựng bảng. API mới chỉ khi thiếu collection hợp lệ; lọc/phân trang phía server hoặc giới hạn an toàn đã chứng minh, không tìm trên một trang rồi coi là toàn bộ dữ liệu.
- Không đồng nhất `employee_id` với `employment_id`: hồ sơ có nhiều đợt làm việc; nhân viên đã nghỉ/tái tuyển vẫn mở đúng lịch sử, không tự trỏ mọi bản ghi vào đợt đang hoạt động.
- Desktop dùng `ResponsiveTable`/`SmartTable` hoặc bảng nghiệp vụ hiện có; mobile giữ card/list. Giữ native scroll, grab-to-pan không chặn link/nút/chọn chữ/keyboard.

## Công việc theo thứ tự — mỗi mục là checkpoint có thể kiểm chứng
| # | Công việc / interface ảnh hưởng | Điều kiện nghiệm thu |
|---|---|---|
| 1 | Chốt route, menu, permission, API/filter/pagination và parity CRUD cho ba nghiệp vụ đợt 1; đối chiếu diff đang có. | Có inventory nguồn và route × role matrix; mọi thiếu hụt ghi rõ, chưa bỏ tab. |
| 2 | Khóa contract filter và API trước UI. Bổ sung filter/phân trang/permission còn thiếu bằng thay đổi additive; cập nhật OpenAPI/client nếu đổi API. | Test UUID sai, ngoài phạm vi, 403, dữ liệu rỗng, nhiều trang; response không chứa T2, không N+1. Nếu backend mở rộng vượt đợt 1, xin duyệt phần bổ sung. |
| 3 | Nối ba trang hiện có với URL filter, chip nhân viên, reset pagination; tái dùng table/form và error/loading state. | Mở URL trực tiếp, đổi/bỏ filter và reload cho kết quả đúng; retry không mất filter; sửa/xóa phản ánh lại cả list và profile. |
| 4 | Chuyển entry point trên cả page và drawer: `employee-detail-page.tsx`, `drawer-header.tsx`, `employee-detail-drawer.tsx`, hook liên quan; thay ba tab bằng tóm tắt/link có permission. | Hai bề mặt nhất quán; create/edit/delete/detail cũ vẫn tới được; không tải dữ liệu trái quyền. |
| 5 | Giữ bookmark `?tab=assignments`, `?tab=working-conditions`, `?tab=lifecycle` bằng ánh xạ sang trang lọc đúng người; xử lý tab sai/thiếu quyền; cập nhật sidebar/search/launcher nếu cần. | Link cũ không crash/404/loop; back quay đúng nơi; không redirect sang trang bị cấm, không âm thầm hiển thị dữ liệu toàn công ty khi mất filter. |
| 6 | Checkpoint đợt 2: đối chiếu hợp đồng thường/điện tử, phase/checklist và trang phân bổ chính sách. Chỉ tạo route/API thiếu sau khi duyệt phạm vi. | Không trộn domain; có parity trước bỏ tab; lương/T2 giữ chi tiết. Đợt 1 có thể kết thúc độc lập nếu đợt 2 chưa duyệt. |
| 7 | Xóa renderer/tab/import/test đã được thay thế sau khi kiểm tra mọi caller; giữ form/detail còn dùng; cập nhật i18n và test kỳ vọng 10 tab. | Không dead code hoặc compatibility shim vô thời hạn; không sửa `.leedevkit/`, không ghi đè diff ngoài nhiệm vụ. |
| 8 | **Verification cuối:** focused unit → `./leedevkit test web`; nếu API đổi, thêm `./leedevkit test api`; kiểm tra integration/E2E và UI theo ma trận đã duyệt. | Đọc log từng phase; báo pass/fail/skip; không báo hoàn tất nếu integration/E2E chưa chạy. Chỉ chuyển Status khi người dùng duyệt, không tự coi yêu cầu lập plan là duyệt code. |

## Test impact và gate phát hành
- **Hành vi/compatibility:** page + drawer, liên kết cũ, URL copy/reload/back-forward, giữ CRUD và lịch sử nhiều employment; cập nhật `pages/__tests__/employee-detail-page.test.tsx` và `components/features/employees/__tests__/employee-detail-drawer.test.tsx` cùng test trang đích.
- **Boundary/state:** không có đợt làm việc, đã nghỉ/tái tuyển, không dữ liệu, nhiều trang, UUID sai, filter hết kết quả, đổi nhanh nhân viên/filter không hiện dữ liệu cũ, mutation lỗi giữ form/dữ liệu.
- **Failure/security:** timeout/5xx có retry; 403/redirect đúng thiết kế khác với 404/crash; API từ chối ngoài quyền/phòng ban, UI không fetch T2 trái quyền; cache không rò giữa phiên. Không đổi schema lưu trữ trong đợt 1 trừ khi có phê duyệt bổ sung.
- **UI:** desktop rộng/hẹp và mobile, keyboard/focus, dark mode, i18n; console/network không lỗi bất ngờ. Tạo route × role matrix trước browser và đọc `docs/dev-accounts.md` trước đăng nhập; không ghi credentials/PII vào artifact.
- **Giới hạn browser:** Playwright MCP phiên hiện tại chỉ cho `employee`. Các vai trò khác dùng test tự động qua wrapper trong môi trường được phép hoặc công cụ/phiên được cấp quyền phù hợp; thiếu bằng chứng thì ghi chưa kiểm, không vượt giới hạn hoặc tuyên bố đã audit mọi role.
- **Rollback:** revert entry point/route UI của đợt tương ứng, giữ API additive tương thích và dữ liệu; không thay schema/xóa dữ liệu để di chuyển giao diện.

**Chờ duyệt:** ba mặc định đầu tài liệu; đợt 2 mặc định là backlog có gate riêng, không tự mở rộng sang bảng lương hoặc tài liệu toàn công ty.
