# Kế hoạch Tối ưu UI/UX Launcher (Phương án A - Clean Enterprise / Raycast Style)

## 1. Compliance & Rule IDs
- **RULE-UI-01**: Tuân thủ Design Rules & WCAG 2.1 AA (`.agent/rules/design-rules.md`): Tương phản màu sắc >= 4.5:1, visual focus indicator rõ ràng, nhãn trợ năng không đổi.
- **RULE-CODE-01**: Tiêu chuẩn Clean Code (`.agent/rules/coding-standards.md`): Không over-engineering, không dead code, giữ vững TypeScript type safety, giữ các testID nguyên vẹn.
- **RULE-TEST-01**: Quy chuẩn kiểm thử (`.agent/rules/testing-standards.md`): Toàn bộ test suite liên quan (`dsm-launcher-overlay.test.tsx`, `use-dsm-launcher.test.ts`, `app-launcher-page.test.tsx`) phải pass 100%.

---

## 2. Mục tiêu tối ưu (Design Goals)
Chuyển đổi giao diện DSM Launcher từ phong cách "Skeuomorphic bóng bẩy / Kích thước quá khổ" sang phong cách "Clean Enterprise / Raycast Spotlight":
1. **Tinh giản thị giác (Visual De-cluttering):**
   - Loại bỏ lớp highlight bóng bẩy (`gloss sheen from-white/25 to-transparent`).
   - Viền mỏng, tinh tế (`border-white/10 dark:border-white/5`), bóng đổ mềm nhẹ thay vì shadow gắt.
2. **Kích thước công thái học (Ergonomic Sizing):**
   - Đưa icon tile từ kích thước quá khổ `96px` (`size="xl"`) về chuẩn Clean `64px` (`w-16 h-16 sm:w-16 sm:h-16`).
   - Icon vector bên trong giảm từ `48px` về `28px - 32px` (`w-7 h-7 sm:w-8 sm:h-8`), cân đối thị giác ~50% diện tích tile.
   - Khoảng cách lưới (`gap`): Giảm từ `gap-8 lg:gap-10` (32px-40px) xuống `gap-4 sm:gap-6` (16px-24px).
3. **Cố định đường chân chữ (Uniform Baseline Alignment):**
   - Text nhãn đặt chiều cao cố định (line-clamp-2 với min-h phù hợp), font `text-xs font-medium text-slate-300`, giới hạn `max-w-[96px]`.
   - Tránh việc tên ngắn 1 dòng và tên dài 2 dòng làm lệch hàng icon ngang.
4. **Spotlight Search Bar làm tâm điểm:**
   - Giữ thanh tìm kiếm nổi bật ở vị trí trung tâm, thao tác phím tắt mượt mà.

---

## 3. Danh sách tệp can thiệp
1. `webdashboard/src/components/features/dsm-launcher/dsm-squircle-icon.tsx`:
   - Cập nhật `sizeClasses` (`md`, `lg`, `xl`).
   - Bỏ gloss overlay `absolute inset-x-0 top-0 h-1/2 bg-gradient-to-b from-white/25...`.
   - Chuẩn hóa style text label với `line-clamp-2`, `h-[32px]` để đều baseline.
2. `webdashboard/src/components/features/dsm-launcher/dsm-launcher-overlay.tsx`:
   - Chuyển `size="xl"` ở All Apps section sang `size="lg"`.
   - Cập nhật class grid từ `gap-4 sm:gap-8 lg:gap-10` thành `gap-4 sm:gap-6 lg:gap-6`.
   - Cập nhật số cột hiển thị cho phù hợp với kích thước tile gọn hơn (`grid-cols-4 sm:grid-cols-5 md:grid-cols-7 lg:grid-cols-8 xl:grid-cols-10`).
3. Unit test:
   - Chạy `webdashboard/src/components/features/dsm-launcher/__tests__/dsm-launcher-overlay.test.tsx`.
