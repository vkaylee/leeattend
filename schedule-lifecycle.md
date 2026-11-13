# Hybrid Schedule Lifecycle

Status: Approved

## Decision và scope
**Planning only; chưa cho phép triển khai.** [Đặc tả triển khai](schedule-lifecycle-spec.md) chốt schema, API, quyền, transaction, cutover và acceptance.

Hybrid: dùng lại `resolveEffectiveShift` cho preview và publication; hành chính tự công bố theo policy đã duyệt, ca xoay công bố batch. Official theo `employment_id + business_date`, current head rõ ràng, revision bất biến; reuse `work_segments` cho toàn bộ interval. Sửa ca tạo revision mới, bắt buộc reason/diff; không giả thời điểm publication; công bố muộn có quyền/reason/nhãn retroactive, không xóa punches/evidence. Missing khác explicit off. Chốt công theo employer/tháng; reuse `payroll_periods` làm khóa lương riêng. Dữ liệu muộn vẫn nhận, không tự sửa kết quả đã chốt. Không thêm service, resolver thứ hai hay event-sourcing platform.

## Verified source facts
- `repository/schedules.go`: precedence roster/pattern/rule/fixed; đang dùng current shift version, chỉ interval `sort_order = 0`; rotation dùng elapsed hours. Cần sửa ngay trong resolver hiện hữu.
- `repository/work_segments.go`: còn UPDATE/DELETE và bulk replace; baseline có employment cascade, interval set-null, unique interval không phân revision.
- `repository/reports.go`: daily đọc segment chưa có revision; no-show dựng `09:00–17:00`; summary hiện nhóm theo employment, không phải employee/employer như summary cũ.
- `repository/payroll_periods.go`, `repository/payroll.go`: đã có employer/month lock, version, `FOR UPDATE`, audit cùng transaction; chưa có attendance closure dependency.
- `repository/audit.go`: nhận DBTX, actor phải UUID. `repository/outbox.go` chỉ enqueue; chưa tìm thấy PostgreSQL outbox consumer/office publisher trong Go. Generic approval không tự công bố lịch.

Các đường dẫn repository trên thuộc `apiserver/internal/`; bằng chứng và giới hạn kiểm chứng nằm trong spec.

## Implementation gates
1. [ ] Duyệt spec/ADR và machine-readable API contract; áp dụng **Architecture Governance §§1–3**, **API Governance §§1–4**.
2. [ ] Expand schema additive, up/down có guard; giữ baseline, provenance legacy và FK evidence; **Database Rules §§3–4**, **Migration and Rollback §§1–4**.
3. [ ] Sửa resolver: tie/missing/off, calendar-date/DST, effective version, temporal employment, holiday/leave, mọi interval.
4. [ ] Publication/amend atomically với head, segment, idempotency và audit; khóa đúng thứ tự; stale/audit failure rollback.
5. [ ] Scope employer/department/self; system actor tuple không giả user UUID, policy approval và office publisher có producer/horizon; **Access Control §§1–2**, **Data Governance §§1–3**.
6. [ ] Chuyển mọi reader/writer sang official contract và guards, gồm matching, reports/no-show, summaries, OT/import/adjustment/bulk.
7. [ ] Close/reopen/reclose bằng snapshot exact revision; payroll lock riêng nhưng ràng buộc closure; late punches không rewrite snapshot.
8. [ ] UI hiện hữu: provisional/published/superseded/closed, diff/reason/history/evidence; i18n, accessibility, route × role acceptance.
9. [ ] Shadow/reconcile và activate từng employer; zero integrity mismatch; chặn app cũ; rollback trước activation, roll-forward sau evidence.
10. [ ] Unit + DB/API integration + migration + frontend + full suite theo spec; đọc từng phase, không nhận skipped gate là pass; **Development Workflow §§2–7**, **Testing Standards §§1–2**.

## Done criteria
Office auto publish, rotating batch, amendment, close, authorized reopen/reclose và evidence export đạt acceptance matrix. Mọi revision/closure cũ tái hiện được sau thay đổi nguồn; không double-count interval/employer, không bypass guard, không mất raw data. Reconciliation đủ coverage, không còn writer/reader legacy hoạt động trong scope đã bật.

Tài liệu chưa phải bằng chứng triển khai: chưa code, chưa migration, chưa runtime test. Kiểm tra tài liệu duy nhất: `git diff --check -- schedule-lifecycle.md schedule-lifecycle-spec.md`.
