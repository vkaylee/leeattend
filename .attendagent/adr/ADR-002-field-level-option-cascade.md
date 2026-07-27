# ADR-002: Field-Level `Option<T>` Cascade & Absolute Delta Caps

**Status:** Accepted (V5 Final)
**Date:** 2026-07-27
**Context:** Attendance Policy Engine Refactor — merge formula giữa các layer
**Related:** [[ADR-001]], [[ADR-004]]

---

## Context

Mỗi layer trong cascade ([[ADR-001]]) có thể override một phần field, hoặc chỉ đóng góp một delta (± một số phút). Cần merge formula:
- Deterministic (cùng input → cùng output).
- Cho phép override tuyệt đối và cộng dồn delta cùng lúc.
- Có thể trace từng contribution ở layer nào ([[ADR-004]] cascade_trace).
- Ngăn giá trị bay ra ngoài giới hạn nghiệp vụ (ví dụ grace_period không âm, không quá 120 phút).

Giải pháp phổ biến "override thắng tuyệt đối" mất đi khả năng cộng dồn delta cho trường hợp thai sản (+30 phút grace, không cần biết grace base là bao nhiêu).

---

## Decision

**Field-level `Option<T>` merge với absolute + delta cùng struct:**

```rust
pub struct PartialPolicyRules {
    pub grace_period_minutes: Option<u32>,
    pub delta_grace_minutes: Option<i32>,
    pub rounding_policy: Option<RoundingPolicy>,
    pub require_geofence: Option<bool>,
    // ... các field khác cùng pattern
}
```

**Merge formula tại mỗi layer (áp dụng từ L2a → L3):**

```
value_new = override.unwrap_or(value_current) + delta.unwrap_or(0)
```

- `override = Some(x)`: reset toàn bộ delta cộng dồn trước đó, đặt lại về `x`.
- `delta = Some(±y)`: cộng dồn lên giá trị hiện tại.
- Cả hai `Some`: override thắng, sau đó cộng delta lên trên override.
- Cả hai `None`: không đổi giá trị hiện tại.

### Ví dụ dòng chảy (grace_period_minutes)

| Layer   | Input             | Kết quả |
|---------|-------------------|---------|
| L1 Base | 10                | 10 |
| L2a     | override=20       | 20 |
| L2b     | (None)            | 20 |
| L2.5    | delta=+30         | 50 |
| L3      | override=15       | **15** (override reset toàn bộ delta trước đó) |

### Absolute caps tại Base Policy

```rust
pub struct BaseFieldSpec<T> {
    pub default_value: T,
    pub absolute_min: Option<T>,
    pub absolute_max: Option<T>,
}
```

Sau khi cascade kết thúc (sau L3), clamp giá trị vào `[absolute_min, absolute_max]`. Nếu bị clamp → log warning vào `cascade_trace.clamp_warnings` (xem [[ADR-004]]) để HR biết modifier/exception vượt policy nghiệp vụ.

---

## Consequences

**Tích cực:**
- Delta và override có thể cùng tồn tại trong 1 layer — L3 vẫn có thể `override=15, delta=+2` = 17.
- Base policy là **single source of truth** về giới hạn nghiệp vụ; không HR nào có thể set grace = 999 phút qua modifier.
- Trace log rõ ràng: HR nhìn cascade_trace hiểu ngay giá trị cuối đến từ đâu, có bị clamp không.

**Tiêu cực:**
- Số field trong `PartialPolicyRules` tăng gấp đôi (mỗi field có bản `Option<T>` + `Option<delta>`). Cần macro hoặc code generation để giảm boilerplate.
- HR có thể set `delta=+30` không biết base là bao nhiêu → cần UI preview "grace hiện tại sẽ thành X phút" trước khi save.

**Rủi ro & mitigation:**
- Rủi ro: clamp âm thầm → warning bắt buộc vào cascade_trace + notification khi cascade bị clamp lần đầu cho một employee.
- Rủi ro: dev quên implement merge cho field mới → trait `Mergeable` + derive macro enforce contract.
