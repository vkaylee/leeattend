# ADR-001: Foundation-Up Layered Hierarchy & Explicit Priority Rank

**Status:** Accepted (V5 Final)
**Date:** 2026-07-27
**Context:** Attendance Policy Engine Refactor — Layered Explicit Policy System
**Related:** [[ADR-002]], [[ADR-003]], [[ADR-004]], [[ADR-005]]

---

## Context

Chính sách chấm công (attendance policy) hiện đang bị phân mảnh:
- Override hardcode trong `shift_default_intervals` (`early_punch_in_grace_period_minutes`, `is_break_auto_deduct`, `allow_early_ot`, `is_break_overtime`).
- Runtime struct `ShiftPolicyOverrides` (`apiserver/src/domain/policy/resolver.rs:244-250`).

Không có nơi tập trung để HR định nghĩa/xem policy. Sửa 1 rule phải sửa nhiều shift; không phân biệt được base rule vs shift-level override vs employee-level modifier.

Vòng phản biện đầu đề xuất **Generic Rule Engine** với tag matching → bị REJECT vì string coupling giữa tag và runtime, UX HR quá tải, rule explosion khi số tag tăng.

---

## Decision

Áp dụng **Layered Explicit Policy System** với 6 tầng cascade từ dưới lên. Layer trên đè layer dưới khi field trùng; các field không trùng được **union** (không loại trừ).

### Cascade order

```
L1 Base Policy               (attendance_policies)
  ↓
L2a Day Type Override        (attendance_policy_day_type_overrides)   — day_type
  ↓
L2b Shift Type Override      (attendance_policy_shift_type_overrides) — shift_type_id
  ↓
L2c Shift Instance Override  (attendance_policy_shift_overrides)      — shift_id
  ↓
L2.5 Employee Modifier       (employee_policy_modifiers)              — priority_rank
  ↓
L3 Ad-hoc Exception          (attendance_exceptions)                  — TOP (WINNER)
```

### Semantics

- **Union of non-overlapping fields:** L2a override `geofence`, L2b override `rounding` → kết quả chứa cả 2.
- **Per-field conflict:** Layer trên thắng khi trùng field.
- **L2.5 multi-modifier:** `priority_rank INT NOT NULL`, giá trị thấp hơn = ưu tiên cao hơn (được xếp sau trong cascade, đè các modifier có priority cao hơn về mặt số).

### Constraint chống overlap tại L2.5

Postgres không cho phép `UNIQUE` trên range với volatile function; dùng `EXCLUDE USING GIST`:

```sql
ALTER TABLE employee_policy_modifiers
  ADD CONSTRAINT no_overlapping_priority
  EXCLUDE USING gist (
    employee_id WITH =,
    priority_rank WITH =,
    tstzrange(effective_from, effective_to, '[)') WITH &&
  );
```

UI/API bắt buộc HR nhập `priority_rank` khi tạo modifier trùng khoảng effective; không có default ngầm để tránh silent conflict.

---

## Consequences

**Tích cực:**
- Type-safe cascade, không phụ thuộc string tag matching.
- HR dễ trace: "field X ở employee Y ngày Z đến từ layer nào".
- Mở rộng layer mới (ví dụ L2d team override) chỉ cần thêm 1 tầng, không thay đổi contract các tầng khác.

**Tiêu cực:**
- Cascade evaluation phức tạp hơn flat resolver hiện tại — cần benchmark p99 latency.
- HR cần training để hiểu concept "priority_rank thấp = ưu tiên cao" (counterintuitive).

**Rủi ro & mitigation:**
- Rủi ro: HR tạo nhiều L2.5 modifier chồng chéo → EXCLUDE GIST constraint chặn ở DB layer.
- Rủi ro: L2c shift instance override bị lạm dụng (workaround thay vì fix L2b) → dashboard cảnh báo khi % shift có L2c > ngưỡng.
