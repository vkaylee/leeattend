# ADR-003: Time-Sliced Evaluation & Per-Rule Anchoring Matrix

**Status:** Accepted (V5 Final)
**Date:** 2026-07-27
**Context:** Attendance Policy Engine Refactor — xử lý ca xuyên ngày và xuyên loại ngày
**Related:** [[ADR-001]], [[ADR-004]]

---

## Context

Ca đêm 22:00 Chủ nhật → 06:00 Thứ Hai (Ngày Lễ) đặt ra câu hỏi:
- Grace period tính theo policy CN hay Lễ?
- Late/early tính so với 22:00 CN hay 06:00 Lễ?
- OT multiplier: 22:00–24:00 hệ số CN, 00:00–06:00 hệ số Lễ, hay chọn một?
- Geofence yêu cầu: theo policy CN (tap in tại 22:05) hay policy Lễ (tap out tại 05:55)?

Các engine attendance đơn giản chọn "single anchoring" — hoặc theo ngày bắt đầu ca, hoặc theo ngày kết thúc — dẫn tới sai payroll khi cross day type.

---

## Decision

### Scope Phase 1

**Chỉ Việt Nam (UTC+7), không cross-timezone, không DST.**
Slice boundary tại 00:00 theo timezone của `employment.location`. Cross-timezone và DST đẩy ra roadmap Phase 2 khi expand quốc tế.

### Decomposition

Ca 22:00 CN → 06:00 T2 Lễ được engine tự động cắt tại 00:00:

- **Slice 1:** 22:00 → 24:00 (Sunday)
- **Slice 2:** 00:00 → 06:00 (Monday Holiday)

Mỗi slice là 1 đơn vị đánh giá độc lập với `day_type` riêng, chạy qua toàn bộ cascade ([[ADR-001]]) với policy tương ứng.

### Anchoring Matrix (đóng đinh)

**KHÔNG có single anchoring rule cho tất cả loại quyết định.** Từng loại rule anchor khác nhau:

| Rule Type | Anchoring | Lý do |
|-----------|-----------|-------|
| Punch Validation (In/Out Grace, Geofence) | **Real tap timestamp slice** | Tap 22:05 → Sunday policy; tap 05:55 → Monday holiday policy |
| Late/Early Classification | **Overall shift schedule** | So với khung 22:00–06:00 gốc, không so với slice |
| OT & Pay Multipliers | **Phân rã theo slice** | 22–24h hệ số CN; 00–06h hệ số Lễ |
| Work Hour Aggregation | **Tổng + slice breakdown** | Tổng cho display, breakdown cho payroll |

---

## Consequences

**Tích cực:**
- Payroll đúng theo luật LĐ VN (mỗi phần thời gian tính đúng hệ số).
- Grace period tính theo policy của ngày mà nhân viên thực sự tap → tránh sai lệch khi tap sát boundary.
- Late/early logic ổn định — không đổi anchoring khi ca cross day type, HR không nhầm lẫn.

**Tiêu cực:**
- Engine phức tạp hơn: mỗi ca có thể tạo nhiều slice, mỗi slice cascade độc lập.
- Report writer cần join slice-level data với shift-level metadata để tổng hợp.

**Rủi ro & mitigation:**
- Rủi ro: bug cắt slice sai (ví dụ ca 23:59 → 00:01 tạo 2 slice cực ngắn) → threshold min-slice-duration = 1 phút, dưới ngưỡng gộp về slice bên phải.
- Rủi ro: DST/timezone khi expand quốc tế → contract của module time-slice viết interface-first để Phase 2 chỉ thay implementation.
- Rủi ro: performance khi ca dài (48h shift lỡ tạo 3 slice) → benchmark cascade eval mỗi slice riêng biệt, cache CompiledPolicy theo (version_id, day_type).
