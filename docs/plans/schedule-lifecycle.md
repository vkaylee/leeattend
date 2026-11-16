# Hybrid Schedule Lifecycle

Status: Architecture direction approved — 2026-11-14. Implementation in progress; backend lifecycle foundation exists, full acceptance and production cutover remain pending.

## Decision và scope
**Implementation tracking.** [Đặc tả triển khai](schedule-lifecycle-spec.md) mô tả schema, API, quyền, transaction, cutover và acceptance. Backend foundation đã được triển khai theo từng phase; full acceptance, consumer cutover, role E2E và production activation vẫn là điều kiện bắt buộc trước khi đánh dấu hoàn tất.

Hybrid: dùng lại `resolveEffectiveShift` cho preview và publication; hành chính tự công bố theo policy đã duyệt, ca xoay công bố batch. Official theo `employment_id + business_date`, current head rõ ràng, revision bất biến; reuse `work_segments` cho toàn bộ interval. Sửa ca tạo revision mới, bắt buộc reason/diff; không giả thời điểm publication; công bố muộn có quyền/reason/nhãn retroactive, không xóa punches/evidence. Missing khác explicit off. Chốt công theo employer/tháng; reuse `payroll_periods` làm khóa lương riêng. Dữ liệu muộn vẫn nhận, không tự sửa kết quả đã chốt. Không thêm service, resolver thứ hai hay event-sourcing platform.

## Verified source facts
- `repository/schedules.go`: precedence roster/pattern/rule/fixed; đang dùng current shift version, chỉ interval `sort_order = 0`; rotation dùng elapsed hours. Cần sửa ngay trong resolver hiện hữu.
- `repository/work_segments.go`: còn UPDATE/DELETE và bulk replace; baseline có employment cascade, interval set-null, unique interval không phân revision.
- `repository/reports.go`: daily đọc segment chưa có revision; no-show dựng `09:00–17:00`; summary hiện nhóm theo employment, không phải employee/employer như summary cũ.
- `repository/payroll_periods.go`, `repository/payroll.go`: đã có employer/month lock, version, `FOR UPDATE`, audit cùng transaction; chưa có attendance closure dependency.
- `repository/audit.go`: nhận DBTX, actor phải UUID. `repository/outbox.go` chỉ enqueue; chưa tìm thấy PostgreSQL outbox consumer/office publisher trong Go. Generic approval không tự công bố lịch.

Các đường dẫn repository trên thuộc `apiserver/internal/`; bằng chứng và giới hạn kiểm chứng nằm trong spec.

## Quyết định HRM và phạm vi phê duyệt

Người dùng đồng ý hướng long-term: assignment có hiệu lực + một resolver + lịch chính thức có revision + công thực tế/tính công/chốt công tách biệt. Phê duyệt này cho phép cập nhật tài liệu, không cho phép code/migration. Spec §17 chi tiết UX, invariants và acceptance; các ngưỡng pháp lý, chuyển employer giữa ca và contract mới còn phải qua gate 1. Không coi mô hình lưu lịch ngày là chuẩn bắt buộc của mọi sản phẩm HRM.

- **Phạm vi:** văn phòng + xoay ca + ca gãy; một ngày có nhiều segment. Office auto-publication cần policy đã duyệt; ca xoay công bố batch. Không thêm engine hoặc bộ tối ưu phân ca tự động.
- **Thời gian:** business date theo ngày bắt đầu ca tại timezone hiệu lực được snapshot; các segment sau nửa đêm giữ cùng ngày công của ca đó. Tính tiền chia theo thời gian thực tế tại mốc ngày lễ/giờ đêm/OT/kỳ lương; không mặc định ca đêm là OT. Timezone precedence giữ theo spec; timestamp, day offset và quan hệ cùng ca phải rõ (§17.4).
- **Trạng thái:** missing, explicit off, approved leave, thiếu chấm công và vắng không phép đã xác nhận là các khái niệm riêng. Lưu riêng công thực tế, công được trả và OT được duyệt; không sửa raw punch để khớp lương.
- **Thẩm quyền:** draft do quản lý đúng scope sửa; published phải revision + reason/diff + thông báo. Ca đã bắt đầu/có punch cần HR duyệt khi đổi nghĩa vụ làm việc. Mở lại công cần người duyệt độc lập; admin kỹ thuật không tự có quyền HR. Đổi ca tự nguyện cần bên nhận đồng ý và quản lý tái kiểm tra lúc duyệt; thông báo/đã xem không thay thế đồng ý.
- **Giới hạn và lương:** overlap thực tế hoặc vi phạm luật áp dụng không được override; cảnh báo nội bộ chỉ override đúng policy/quyền/reason. HR/pháp chế xác nhận ngưỡng, nhóm lao động và hệ số. Lương đã chi trả giữ nguyên bằng chứng, điều chỉnh qua kỳ bổ sung/kỳ sau. Ca qua mốc chuyển employer cần quyết định phân bổ riêng, không double-count; phải hòa giải với quy tắc một employer/closure-day trong spec trước activation.

- **Onboarding và UX:** gán pattern/chu kỳ theo khoảng hiệu lực một lần; cho phép phân lịch sau mà không chặn lưu hồ sơ/raw punch. Lưu lịch ngày/segment, thao tác tuần/tháng/hàng loạt. Hiện riêng assignment và độ phủ công bố; bảng tuần có missing/nháp/công bố/nghỉ/xung đột, không dùng một cờ “đã xếp lịch” vĩnh viễn trên employee.
- **Độ phủ:** tính trên khoảng cần công bố giao với employment có hiệu lực; office từ hôm nay đến cuối tháng sau, rotating theo đợt công bố rõ. Người thiếu lịch khác ca thiếu người; module nhu cầu nhân lực/open shifts/auto-optimizer chưa thuộc đợt này.
- **Chống trùng:** kiểm tra khoảng thực tế của cùng employee trên mọi employment, employer, ngày công liền kề và trong chính batch; serialize theo employee, không chỉ employment. Kiểm tra xung đột liên scope không tiết lộ lịch/PII ngoài quyền.

## Ranh giới ca và chính sách — hướng đã đồng ý

Ca sở hữu cấu trúc thời gian làm/nghỉ, không sở hữu toàn bộ quy tắc chấm công/OT. Spec §18 chốt field ownership, provenance, migration và acceptance B1–B10. Reuse `WorkPolicyGroupVersion` cùng attendance/OT/work-credit/pay-basis policies; xác minh resolver/runtime trước tích hợp, không tạo engine hoặc cấu hình trùng.

- **Giữ ở ca:** tên/ký hiệu, interval/day offset, giờ hoặc thời lượng nghỉ dự kiến, version; employer/location chỉ là scope/default, không quyết định pháp nhân trả lương.
- **Chuyển về policy:** dung sai đi muộn/cửa sổ ghép punch/cách tính đến sớm được tách nghĩa; khấu trừ nghỉ thuộc attendance/valuation; điều kiện/giới hạn/gộp OT thuộc OT policy, không đặt cách gộp cả ngày trên từng interval.
- **Suy ra:** qua ngày từ timestamps/day offsets; phút đêm từ giao thời gian hợp lệ với khung giờ đêm policy. Không dùng `is_night_shift` làm căn cứ duy nhất trả tiền.
- **Bảo toàn:** không suy nghĩa grace hoặc quyền hưởng lương từ tên field; khảo sát caller/default/precedence, lập mapping có hiệu lực, so sánh kết quả cũ/mới. Giữ snapshot cũ; thay đổi kết quả có chủ đích phải có duyệt riêng. Không drop field hoặc bỏ kiểm soát OT trước cutover toàn bộ consumer.

## Backlog tính đúng và acceptance bắt buộc

Các phát hiện dưới đây từ khảo sát source trước đó, chưa tái hiện runtime; xác minh lại trên HEAD khi triển khai. Ưu tiên sửa tính đúng trong engine/consumer hiện hữu trước khi activate lifecycle; không tạo đường đọc/ghi song song lâu dài.

| ID / gate | Vấn đề và xử lý tối thiểu | Acceptance cần chứng minh |
|---|---|---|
| H1 / 3 | `shifts.go`, `schedules.go`: chọn version theo ngày hiệu lực, giữ chronology hợp lệ, trả đủ interval; không dùng current pointer cho mọi ngày | Revision hiệu lực 01/12 không đổi lịch tháng 11; ca 08–12 + 14–18 giữ đủ hai interval; interval/break sai hoặc overlap bị từ chối |
| H2 / 3–4 | `shifts.go`, `domain/compliance`: kiểm tra lịch đã resolve từ mọi nguồn và ngày liền kề bằng khoảng thời gian thực; tải đủ tuần/tháng; xử lý assignment vô thời hạn theo horizon rõ, không chứng nhận cả tương lai từ ngày đầu | 22–06 và 05–13 hôm sau bị chặn; đủ tổng giờ tuần/tháng; `AllowManagerOverride=false` không vượt được; assignment ngoài employment bị từ chối; retry/concurrent không tạo lịch trùng |
| H3 / 6 | `reports.go`: tách grain ngày/segment/session, aggregate trước JOIN; no-show theo segment chính thức, không dựng 09–17; dùng timestamp/timezone đúng | Hai segment × hai session không nhân tổng phút hoặc sai pagination; đi sáng/vắng chiều vẫn hiện ca chiều; ca đêm/off/cancelled/approved leave không bị kết luận vắng sai |
| H4 / 4–6 | `shift_swaps.go`, `work_segments.go`: đổi ca qua cùng amendment contract, không chỉ sửa roster; chặn UPDATE/DELETE/bulk bypass revision/closure | Lịch, segment, ESS và reports thống nhất sau đổi; hai lần duyệt đồng thời hoặc lịch đổi sau yêu cầu trả conflict; thất bại rollback toàn bộ; lịch sử cũ còn nguyên |
| H5 / 3–6 | `work_segments.go`, `leaves.go`: phân biệt giá trị không truyền với compensation 0; phép dựa phần giao lịch/nghỉ/break thay vì cố định thứ Hai–thứ Sáu | On-call 0% vẫn 0%; thiếu giá trị mới dùng default; người làm Chủ nhật/nghỉ thứ Hai, nghỉ nửa ngày/hai giờ/ca đêm/ngày lễ được tính đúng, không trừ phép trùng |
| H6 / 6–7 | `domain/pairing`, `domain/payroll/buckets` và runtime: xác minh wiring trước sửa; xử lý mọi segment, log lặp/lệch thứ tự/thiếu IN-OUT, nhiều break và OT nghỉ; không coi field/hàm chưa được gọi là tính năng chạy | Raw log thành session/summary/payroll có DB/API integration; ghép đủ ca gãy, không dùng lại punch hoặc trừ break hai lần; công thực tế/được trả/OT duyệt đối soát được |
| H7 / 7–9 | Snapshot lịch/policy/calculation, khóa công và payroll riêng; ingest muộn tạo reconciliation, không rewrite closure; quy tắc ngày công và phân bổ tiền phải được duyệt trước cutover | Ca qua cuối tháng/ngày lễ/chuyển employer không mất hoặc nhân phút/tiền; đóng đồng thời amendment/import/OT không lọt ghi; reopen/reclose giữ snapshot trước và khoản đã chi trả |
| H8 / 1,5,8 | Onboarding và coverage theo §17.2–17.3: reuse assignment và màn hình lịch, không thêm dashboard; nháp là source/preview, không thêm bảng draft-day riêng | Vào làm thứ Tư không báo thiếu thứ Hai/ba; gán pattern chưa đồng nghĩa published; người nghỉ việc không bị báo thiếu sau end date; thiếu ca tuần sau hiện đúng scope |
| H9 / 2–4 | Cross-employment theo employee và half-open timestamp; bổ sung employee lock trước employer lock; validate cả interval của batch | Hai employer cùng publish ca trùng cho một người: tối đa một commit; ca 22–06 trùng 05–13 bị chặn; 06:00 kết thúc và 06:00 bắt đầu không overlap nhưng vẫn kiểm tra nghỉ |
| H10 / 1–4,6 | Định danh ca trong revision, day offsets và timestamps theo §17.4; không suy ngày chỉ từ sort_order/giờ kết thúc | 22–02 + 02–06 cùng ca giữ ngày bắt đầu, hai khoảng liên tục và đủ phút; DST ambiguous/nonexistent không tự đoán; ngày công và kỳ trả tiền không double-count |
| H11 / 1–3,6,8–9 | Tách ownership ca/policy theo §18; reuse work-policy group, version/snapshot đủ policy; migration field có mapping và cutover consumer đồng bộ | B1–B10: không nhầm grace/qua ngày/giờ đêm, không tự duyệt OT, không đổi quyền lợi khi đổi ca; parity hoặc sai khác được duyệt có bằng chứng; old clients không ghi ngầm field đã chuyển |

## Implementation gates
1. [ ] Ghi ADR hướng đã đồng ý; review spec §§17–18, field ownership/mapping/precedence và schema/API delta trước khi khóa schema lifecycle; khóa acceptance H1–H11, L1–L12 và B1–B10 trước triển khai. Áp dụng **Architecture Governance §§1–3**, **API Governance §§1–4**.
2. [ ] Expand schema additive, up/down có guard; giữ baseline, provenance legacy và FK evidence; **Database Rules §§3–4**, **Migration and Rollback §§1–4**.
3. [ ] Sửa tính đúng H1/H2/H5 trong resolver/validation hiện hữu: tie/missing/off, calendar-date/DST, effective version, temporal employment, holiday/leave, mọi interval; regression unit + DB/API integration trước activation.
4. [ ] Publication/amend atomically với head, segment, idempotency và audit; khóa đúng thứ tự; stale/audit failure rollback.
5. [ ] Scope employer/department/self; system actor tuple không giả user UUID, policy approval và office publisher có producer/horizon; **Access Control §§1–2**, **Data Governance §§1–3**.
6. [ ] Chuyển mọi reader/writer sang official contract và guards; hoàn tất H3–H6, gồm swap, matching, reports/no-show, summaries, leave, OT/import/adjustment/bulk. Không activate nếu raw-log-to-summary/payroll chưa có bằng chứng integration.
7. [ ] Close/reopen/reclose bằng snapshot exact revision; payroll lock riêng nhưng ràng buộc closure; late punches không rewrite snapshot.
8. [ ] UI hiện hữu: provisional/published/superseded/closed, diff/reason/history/evidence; i18n, accessibility, route × role acceptance.
9. [ ] Shadow/reconcile và activate từng employer; zero integrity mismatch; chặn app cũ; rollback trước activation, roll-forward sau evidence.
10. [ ] Unit + DB/API integration + migration + frontend + full suite theo spec; đọc từng phase, không nhận skipped gate là pass; **Development Workflow §§2–7**, **Testing Standards §§1–2**.

## Done criteria
Office auto publish, rotating batch, amendment, close, authorized reopen/reclose và evidence export đạt acceptance matrix và H1–H11/L1–L12/B1–B10 đã duyệt. Mọi revision/closure cũ tái hiện được sau thay đổi nguồn; không double-count interval/employer, không bypass guard, không mất raw data. Reconciliation đủ coverage, không còn writer/reader legacy hoạt động trong scope đã bật. Mỗi H-ID phải có bằng chứng kiểm thử hành vi, không chỉ test mock query hoặc sự tồn tại của bảng/field.

Tài liệu không thay thế bằng chứng runtime. Backend foundation, migrations và focused tests đã tồn tại trong repository; các gate còn mở phải có test/integration/E2E evidence riêng. Kiểm tra tài liệu: `git diff --check -- schedule-lifecycle.md schedule-lifecycle-spec.md`.
