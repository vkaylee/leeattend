# ADR-004: Hybrid Immutable Snapshot, Cascade Trace & 3-State Lifecycle

**Status:** Accepted (V5 Final)
**Date:** 2026-07-27
**Context:** Attendance Policy Engine Refactor — audit trail immutability
**Related:** [[ADR-001]], [[ADR-002]], [[ADR-005]]

---

## Context

HR có thể edit `employee_policy_modifiers` retroactive (ví dụ ngày 20 mới nhớ ra thai sản của Alice bắt đầu ngày 1). Nếu attendance engine re-evaluate on-the-fly khi HR mở lại tháng cũ, lịch sử bị rewrite âm thầm — không phù hợp yêu cầu audit LĐ VN.

Đồng thời cần cho phép admin sửa lỗi thực sự (không phải rewrite lịch sử). Cần lifecycle rõ ràng cho policy version và snapshot mechanism cho từng dòng attendance.

Vòng phản biện: "cache CompiledPolicy đủ chưa?" → KHÔNG. Cache mất khi restart; cần persist snapshot vào chính `daily_attendance` để bất kể policy có edit gì về sau, historical row vẫn phản ánh đúng lúc chấm công.

---

## Decision

### 1. Policy Version Lifecycle — 3-state machine

```
DRAFT ──activate──▶ ACTIVE ──first_reference──▶ FROZEN
  │                    │                          │
  Editable            Metadata only               100% immutable
  Deletable          (name/description)          (terminal)
```

**Field split:**
- **Metadata (editable trong ACTIVE):** `name`, `description`, `updated_by`, `updated_at`.
- **Rules (locked trong ACTIVE & FROZEN):** `base_rules`, `conditional_overrides`, `delta_specs`.

### 2. `rules_hash` — DB-computed generated column

Không tin app tính hash — Postgres tự tính từ JSONB text để đảm bảo deterministic:

```sql
ALTER TABLE attendance_policy_versions
  ADD COLUMN rules_hash TEXT GENERATED ALWAYS AS (
    md5(base_rules::text || conditional_overrides::text || delta_specs::text)
  ) STORED;
```

### 3. Trigger enforce immutability

```sql
CREATE OR REPLACE FUNCTION enforce_policy_version_immutability()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status = 'FROZEN' THEN
        RAISE EXCEPTION 'POLICY_VERSION_FROZEN: cannot modify frozen version %', OLD.id;
    END IF;
    IF OLD.status = 'ACTIVE' AND NEW.status = 'ACTIVE'
       AND OLD.rules_hash IS DISTINCT FROM NEW.rules_hash THEN
        RAISE EXCEPTION 'POLICY_VERSION_ACTIVE_LOCKED: rules of ACTIVE version immutable; create new DRAFT';
    END IF;
    -- Ensure monotonic state transition
    IF (OLD.status, NEW.status) NOT IN (
      ('DRAFT','DRAFT'), ('DRAFT','ACTIVE'),
      ('ACTIVE','ACTIVE'), ('ACTIVE','FROZEN'),
      ('FROZEN','FROZEN')
    ) THEN
        RAISE EXCEPTION 'POLICY_VERSION_INVALID_TRANSITION: % -> %', OLD.status, NEW.status;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

### 4. Cascade Trace Snapshot per Attendance Row

Mỗi `daily_attendance` lưu `effective_rules_snapshot JSONB`:

```json
{
  "final_rules": {
    "grace_period_minutes": 40,
    "require_geofence": true,
    "rounding_policy": "floor_15m"
  },
  "cascade_trace": [
    { "layer": "L1_base", "policy_version_id": "8f3b-...", "contributions": { "grace_period_minutes": 10 } },
    { "layer": "L2a_day_type", "contributions": { "grace_period_minutes": 20 } },
    { "layer": "L2b_shift_type", "contributions": {} },
    { "layer": "L2c_shift_instance", "contributions": {} },
    { "layer": "L2.5_employee",
      "modifier_id": "mod_maternity_099",
      "priority_rank": 1,
      "modifier_snapshot": { "delta_grace_minutes": 20 } },
    { "layer": "L3_exception", "exception_id": null, "contributions": {} }
  ],
  "clamp_warnings": []
}
```

Trace này freeze giá trị **modifier tại lúc đánh giá** (`modifier_snapshot`), không lookup runtime — nên HR edit `employee_policy_modifiers` về sau không thay đổi historical row.

### 5. Historical Correction — Supersede & Recompute

**KHÔNG DELETE FROZEN row.** Quy trình:

1. Tạo policy version mới (DRAFT → ACTIVE).
2. Batch job đánh `is_superseded = true` trên `daily_attendance` cũ.
3. Insert row `daily_attendance` mới với:
   - `superseded_by_id → old_row`
   - `policy_version_id → new_version`
   - `effective_rules_snapshot` = trace từ engine mới.

Row cũ vẫn tồn tại immutable cho audit; UI hiển thị mặc định row mới nhất, có link "xem phiên bản trước".

### 6. Storage estimation

- Scale: 50k nhân viên × 365 ngày ≈ 18.25M rows/năm.
- Trung bình snapshot ~2KB → ~36GB/năm raw JSONB.
- **Retention:** 7 năm hot storage (khớp luật LĐ VN), sau đó partition-by-month → S3 Glacier archive.

---

## Consequences

**Tích cực:**
- Audit trail chính xác 100% ngay cả khi HR edit modifier retroactive.
- HR có thể "sửa lỗi" bằng cách tạo policy version mới + supersede — không có cửa nào edit row cũ.
- Rules_hash DB-computed → không có bug hash mismatch giữa Rust ↔ SQL.

**Tiêu cực:**
- Storage tăng ~36GB/năm cho JSONB snapshot — chấp nhận được so với ROI audit.
- UI cần thêm view "xem phiên bản trước" cho supersede chain.

**Rủi ro & mitigation:**
- Rủi ro: snapshot size grow không kiểm soát nếu cascade_trace verbose → cap `contributions` chỉ log field khác giá trị current (không log toàn bộ struct).
- Rủi ro: batch job supersede fail giữa chừng → transaction wrap per-batch (10k rows/batch), có resume checkpoint.
- Rủi ro: FROZEN version quá nhiều gây clutter UI → filter mặc định chỉ hiển thị 12 tháng gần nhất, có search.
