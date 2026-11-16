# Task Plan: DSM 7 Style App & Feature Launcher

## 1. Compliance & Rule IDs
- **RULE-UI-01**: UI/UX Design System & WCAG 2.1 AA Compliance (`.agent/rules/design-rules.md`) - Focus trap, keyboard navigation (Esc, arrows, enter), color contrast, full accessibility.
- **RULE-CODE-02**: Clean Code, SOLID & Zero Dead Code (`.agent/rules/coding-standards.md`) - Strict TypeScript types, no `any`, modular feature registry.
- **RULE-SEC-01**: Access Control & Permission Gating (`.agent/rules/access-control.md`) - Strict filtering by permissions and system roles so unauthorized apps/features are never displayed.
- **RULE-TEST-01**: Testing Standards & Production Readiness (`.agent/rules/testing-standards.md`) - Comprehensive unit and integration test coverage.

## 2. Architecture & Components
1. **Feature Registry (`webdashboard/src/lib/feature-registry.ts`)**:
   - Master catalog of apps, sub-features, actions, and settings.
   - Vietnamese diacritic-agnostic keywords and aliases (e.g., "ot", "tang ca", "nghi phep", "bang luong").
   - Permission guards matching LeeAttend RBAC.
2. **DSM 7 Fullscreen Launcher Overlay (`webdashboard/src/components/features/dsm-launcher/dsm-launcher-overlay.tsx`)**:
   - Built on `@radix-ui/react-dialog` + `cmdk` + `framer-motion`.
   - **Fullscreen Desktop Mode**: Trải rộng toàn màn hình (`w-screen h-screen fixed inset-0`), hiệu ứng kính mờ sâu (`backdrop-blur-3xl bg-slate-950/85`).
   - Header toàn màn hình: Brand LeeAttend OS góc trái, Spotlight Search Bar lớn ở trung tâm (`max-w-2xl`), phím tắt `ESC` và nút đóng góc phải.
   - Default view: Lưới ứng dụng màn hình lớn chuẩn DSM 7 với squircle tiles kích thước lớn (`size="xl"`: `w-24 h-24`), gradient màu sắc rực rỡ và mục ứng dụng truy cập gần đây (`size="md"`).
   - Search view: Khung tìm kiếm ở giữa màn hình gom nhóm thông minh (Ứng dụng, Thao tác nhanh, Cài đặt hệ thống) hỗ trợ tiếng Việt không dấu.
3. **Trigger Integration**:
   - Hotkey listener for `Cmd+K` / `Ctrl+K`.
   - Launcher button in `Navbar` and `IconRail`.
4. **Testing**:
   - Unit tests covering search indexing, permission filtering, keyboard navigation, and open/close state.
