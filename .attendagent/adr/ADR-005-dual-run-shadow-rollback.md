# ADR-005: Dual-Run Shadow Mode & Rollback Protocol

**Status:** Accepted (V5 Final)
**Date:** 2026-07-27
**Context:** Attendance Policy Engine Refactor — cutover migration strategy
**Related:** [[ADR-001]], [[ADR-004]]

---

## Context

Cutover từ engine cũ (`ShiftPolicyOverrides` + `shift_default_intervals` hardcode) sang engine mới (cascade layered) không thể "flip switch" — attendance là dữ liệu payroll, sai 1 dòng = HR phải explain với lương nhân viên.

Vòng phản biện đầu đề xuất "shadow run 7 ngày, target 100% match". → REJECT vì engine mới **cố tình sửa bug cũ** (ví dụ bug overnight rest gap [[commit d3eeec732]]). Nếu target 100% match với engine cũ, đồng nghĩa keep bug — mất mục đích refactor.

Cần định nghĩa **diff taxonomy** rõ: cái nào là expected fix, cái nào là regression.

---

## Decision

### 1. Diff Taxonomy (3 nhóm)

- **Group A — Expected:** engine mới sửa bug cũ → phải document trong `.attendagent/migration_expected_diffs.md` với reference commit/issue.
- **Group B — Unexpected new-engine bug:** engine mới sai với ngữ nghĩa policy → **BLOCK cutover**.
- **Group C — Regression:** engine mới sai vs engine cũ đúng → **BLOCK cutover**.

### 2. Cutover Threshold

- 0 diff Group B/C bắt buộc.
- Group A chấp nhận **nếu 100% documented** trong migration expected diffs.
- Nếu Group A > 5% tổng row → holdup, HR review chấp thuận trước khi cutover.

### 3. Timeline

1. **14 ngày Shadow Run:**
   - Engine mới ghi ngầm vào `daily_attendance_shadow`.
   - Engine cũ giữ production, ghi `daily_attendance`.
   - Diff report daily gửi HR + engineering.
2. **Cutover:** khi thoả threshold → engine mới lên production, ghi trực tiếp `daily_attendance`.
3. **30 ngày Dual-Run post-cutover:**
   - Engine cũ vẫn chạy ngầm, ghi `daily_attendance_legacy_shadow`.
   - Rollback window: nếu có sự cố nghiêm trọng, có thể revert.
4. **Rollback path (nếu cần trong 30 ngày):**
   - Point-in-time revert script khôi phục output engine cũ về `daily_attendance`.
   - Script auto-export `attendance_exceptions` và `employee_policy_modifiers` tạo từ cutover đến hiện tại → CSV cho HR re-apply thủ công.
   - Warning banner UI trong window: "Dual-run window — modifier/exception mới có thể mất nếu rollback."
5. **Sau 30 ngày ổn định:** xoá engine cũ + `daily_attendance_legacy_shadow`, đóng rollback window.

### 4. Legacy Manual Edit Migration

Attendance rows hiện có HR đã sửa tay (`is_manual_override = true`) cần phân loại:

- **Cat 1 — Duyệt trễ/nghỉ 1 ngày:** migrate → `attendance_exceptions` (L3, [[ADR-001]]).
- **Cat 2 — Điều chỉnh dài hạn** (thai sản, thử việc, khu vực đặc biệt): migrate → `employee_policy_modifiers` (L2.5).
- **Cat 3 — Không phân loại được** (comment lộn xộn, thiếu context): **giữ nguyên historical row**, flag `requires_hr_review = true`, KHÔNG auto-migrate. HR duyệt thủ công sau cutover.

---

## Consequences

**Tích cực:**
- Rollback path rõ, có script và warning UI → không rủi ro "mắc kẹt" với engine mới nếu có bug lộ ra sau cutover.
- Diff taxonomy phân biệt bug fix vs regression → team không tranh cãi "diff này có ok không".
- Cat 3 legacy edit không auto-migrate → tránh silent corruption khi HR ý định không rõ.

**Tiêu cực:**
- 14 + 30 = 44 ngày dual-run tốn compute (~2× cost period).
- Rollback path bỏ modifier/exception tạo trong dual-run window → HR cần re-apply CSV, có ma sát.

**Rủi ro & mitigation:**
- Rủi ro: engine mới đi vào production trước khi Group A/B/C đủ mẫu edge case → shadow run bắt buộc bao gồm 1 chu kỳ lương (28+ ngày) để cover đủ ngày lễ + ca đêm.
- Rủi ro: HR quên window 30 ngày rồi mới báo bug → alert countdown trong admin dashboard 7 ngày, 3 ngày, 1 ngày trước khi đóng window.
- Rủi ro: Cat 3 legacy edit tồn đọng vô thời hạn → SLA 90 ngày HR phải review, sau đó auto flag `hr_review_stale`.
