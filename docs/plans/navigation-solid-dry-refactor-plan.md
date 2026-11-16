# Kế Hoạch Tối Ưu Hóa Menu & Cài Đặt (SOLID & DRY Architecture)

Status: Approved

## 🎯 Mục Tiêu
Tái cấu trúc luồng điều hướng (Navigation Sidebar & Settings Hub) của LeeAttend theo chuẩn Enterprise HRM:
1. **Triệt tiêu trùng lặp (DRY):** Xóa bỏ điểm khai báo phân mảnh giữa `APP_REGISTRY`, `useNavMenuItems()`, `useSettingsNav()`; hợp nhất các tính năng trùng lặp (`/approvals/ot` vào `/approvals`).
2. **Chuẩn hóa trách nhiệm (SOLID - SRP):** Phân định ranh giới rõ ràng giữa **Tác vụ vận hành hàng ngày (Main Menu)** và **Chính sách, Mẫu & Thiết lập kỹ thuật (Settings Hub)**.
3. **Mở rộng dễ dàng (SOLID - OCP & LSP):** Thống nhất Navigation Data Contract, cho phép mở rộng module mới mà không sửa tràn lan component giao diện.

---

## 📋 Rule Compliance Check (MANDATORY)
1. **Rule ID: `coding-standards.md#2-clean-code-standards`**: Áp dụng triệt để SOLID & DRY, Single Responsibility Principle; loại bỏ Dead Code và các đường dẫn điều hướng rác.
2. **Rule ID: `project-structure.md#1-directory-structure`**: Duy trì Single Source of Truth cho route constants (`routes.ts`) và app definitions (`app-registry.ts`).
3. **Rule ID: `design-rules.md#2-accessibility--wcag-21-aa-standards`**: Đảm bảo keyboard navigation (focus-visible rings), aria-labels, semantic HTML (`<nav>`, `<aside>`) và test IDs chuẩn mực.
4. **Rule ID: `coding-standards.md#6-production-readiness`**: Hoàn thiện 100% i18n (tiếng Việt & tiếng Anh), đồng bộ trạng thái loading và quyền RBAC cho từng mục.

---

## 🏛️ Thiết Kế Kiến Trúc: SOLID & DRY Navigation Contract

### 1. Phân Tách Trách Nhiệm (Single Responsibility Principle - SRP)
- **Main Menu (Sidebar):** Dành riêng cho **Nghiệp vụ vận hành thường nhật (Day-to-day Operations)** theo 4 nhóm vai trò chính:
  - *HR Director / C&B:* Nhân sự, Hợp đồng, Bảng lương, Chốt công, Báo cáo.
  - *Shift Manager:* Lịch làm việc & Xếp ca (`/shifts`), Đổi ca.
  - *Team Lead / Manager:* Hộp thư phê duyệt tập trung (`/approvals`).
  - *Employee:* Tự phục vụ (ESS).
- **Settings Hub (Cài đặt):** Dành riêng cho **Thiết lập tham số, Quy định chính sách & Cấu hình kỹ thuật**:
  - *Access & Accounts:* Phân quyền, Vai trò, Tài khoản thành viên hệ thống (`/workspace/settings/members`).
  - *Policies & Rules:* Quy định tăng ca (`overtime`), Quy tắc chấm công (`time`), Mẫu ca & Quy tắc phân ca tự động (`shifts-rules`), Chính sách phép (`leave`), Quy trình duyệt đa cấp (`workflows`).
  - *System & Integrations:* SMTP Server, Storage S3 MinIO, Cấu hình Chữ ký số E-Contracts.

### 2. Triệt Tiêu Trùng Lặp (DRY Principle)
- **Hợp nhất Phê duyệt:** Gộp hoàn toàn `/approvals/ot` vào `/approvals` (Unified Inbox với tab filter `type="overtime"`). Xóa bỏ liên kết thừa cấp 1 trên Sidebar.
- **Xóa tab redirect rỗng:** Xóa tab "Phòng ban" trong `organization-settings.tsx` (tab chỉ có 1 nút bấm quay về `/departments`). Đổi tên danh mục cài đặt thành "Pháp nhân & Mốc công tác" (`Employers & Employment Phases`).
- **Thống nhất Single Source of Truth:** `APP_REGISTRY` trở thành master catalogue định nghĩa route, permissions, testId, labelKey. `useNavMenuItems` và `useSettingsNav` derive trực tiếp từ master metadata hoặc chia sẻ typed schema chung thay vì hardcode duplicate.

---

## 🔄 Ma Trận Điều Chuyển Navigation (Re-allocation Matrix)

### A. Từ Menu Chính -> Đưa Vào Cài Đặt (Main Menu -> Settings)
| STT | Mục | Đường dẫn hiện tại | Lý do nghiệp vụ | Đích đến mới trong Settings |
|:---:|:---|:---|:---|:---|
| 1 | **Cấu hình SMTP** | `/system/smtp` | Cấu hình kỹ thuật 1 lần của Admin IT; không phải nghiệp vụ HR. | `Settings -> System -> Máy chủ Email (SMTP)` (`ROUTES.SETTINGS.SMTP`) |
| 2 | **Quy tắc phân ca (Shift Rules)** | `/shifts/rules` | Chính sách và mẫu ca tuần hoàn (Policy/Template) cùng bản chất với OT Rules, Time Rules. | `Settings -> Operations -> Quy tắc phân ca & Mẫu ca` (`ROUTES.SETTINGS.SHIFTS_RULES`) |
| 3 | **Thành viên hệ thống (People)** | `/workspace/people` | Quản trị tài khoản đăng nhập/mời người dùng; phân biệt với Hồ sơ nhân viên (`/employees`). | `Settings -> Access -> Tài khoản người dùng` (`ROUTES.SETTINGS.MEMBERS`) |
| 4 | **Duyệt OT riêng** | `/approvals/ot` | Trùng lặp với Hộp thư phê duyệt hợp nhất. | Gộp vào `/approvals?type=overtime`, gỡ link cấp 1 trên Sidebar. |

### B. Từ Cài Đặt -> Đưa Ra Menu Chính (Settings -> Main Menu)
| STT | Mục | Đường dẫn hiện tại | Lý do nghiệp vụ | Vị trí mới trên Menu chính |
|:---:|:---|:---|:---|:---|
| 1 | **Lịch ngày nghỉ lễ** | `/workspace/settings/holidays` | Lịch biểu chung công ty; HR và Quản lý ca cần đối soát thường xuyên khi xếp lịch và chốt công. | Menu chính nhóm **Thời gian & Chấm công** (`time`): `ROUTES.HOLIDAYS` |
| 2 | **Hợp đồng lao động (E-Contracts)** | `/econtracts` (bị ẩn bởi flag) | Nghiệp vụ cốt lõi của C&B (theo dõi hạn HĐ, phụ lục, thử việc lên chính thức). | Kích hoạt vào Menu chính nhóm **Nhân sự** (`org`). |

---

## 📋 Danh Sách Tác Vụ Cụ Thể (Actionable Tasks)

### Phase 1: Chuẩn Hóa Routing & Master Registry (DRY Foundation)
- [x] **Task 1: Cập nhật `ROUTES` constants**
  - File: `webdashboard/src/lib/routes.ts`
  - Bổ sung `SETTINGS.SMTP: "/workspace/settings/smtp"`, `SETTINGS.SHIFTS_RULES: "/workspace/settings/shifts-rules"`, `HOLIDAYS: "/holidays"`.
  - Giữ compatibility redirect nếu người dùng bookmark đường dẫn cũ.
  - Verify: `npx tsc --noEmit` không có lỗi cú pháp.

- [x] **Task 2: Đồng bộ `APP_REGISTRY` & Metadata**
  - File: `webdashboard/src/lib/app-registry.ts` & `webdashboard/src/lib/route-metadata.ts`
  - Cập nhật nhóm (`group`) cho các module được điều chuyển.
  - Loại bỏ entry rác `/approvals/ot`, liên kết trực tiếp vào unified inbox.
  - Verify: Drift-detection tests phản ánh cấu trúc mới.

### Phase 2: Tái Cấu Trúc Main Menu Hook & Sidebar (SRP Execution)
- [x] **Task 3: Refactor `useNavMenuItems()`**
  - File: `webdashboard/src/hooks/use-nav-menu-items.ts`
  - Nhóm `org`: Giữ `employees`, `departments`, `org-chart`, kích hoạt `econtracts`. Chuyển `people` sang Settings.
  - Nhóm `time`: Bổ sung `holidays`, giữ `attendance-logs`, `leave-requests`, `leave-balances`, `attendance-adjustments`, `shift-swaps`, `reports`, `approvals`. Xóa `approvals-ot`.
  - Nhóm `shifts`: Chỉ giữ `shifts` (Roster/Xếp ca). Chuyển `shifts-rules` sang Settings.
  - Nhóm `system`: Xóa `system-smtp`. Chỉ giữ `infra-monitor` và `audit-logs` (được bảo vệ nghiêm ngặt bởi quyền `admin`).
  - Verify: Hook trả về đúng danh sách items tinh gọn, không còn mục rác kỹ thuật.

- [x] **Task 4: Tinh gọn Unified Inbox Header & Linkings**
  - File: `webdashboard/src/pages/approvals/unified-inbox-page.tsx`
  - Đảm bảo query param `?type=overtime` tự động active filter tab Overtime khi người dùng truy cập từ notification hoặc liên kết nhanh.
  - Verify: Truy cập `/approvals?type=overtime` hiển thị ngay danh sách đơn tăng ca cần duyệt.

### Phase 3: Tái Cấu Trúc Settings Hub (Configuration Consolidation)
- [x] **Task 5: Refactor `use-settings-nav.tsx` & Sidebar**
  - File: `webdashboard/src/pages/settings/hooks/use-settings-nav.tsx`
  - Thêm `shifts-rules` vào `operationsGroup` (icon `Zap`, nhãn "Quy tắc phân ca & Mẫu ca").
  - Thêm `smtp` vào `systemGroup` (icon `Mail`, nhãn "Máy chủ Email SMTP").
  - Chuyển `holidays` từ Settings sang menu chính (hoặc giữ shortcut liên kết nhanh).
  - Tối ưu `organization`: Đổi nhãn thành "Pháp nhân & Mốc công tác", gỡ tab rỗng redirect phòng ban.
  - Verify: Settings sidebar hiển thị trực quan, đúng 4 nhóm chức năng rõ ràng.

- [x] **Task 6: Tạo Route Settings cho SMTP & Shifts Rules**
  - File: `webdashboard/src/routes/_authenticated/_workspace/workspace/settings/`
  - Tạo route lazy load cho SMTP và Shifts Rules bên trong khung Layout của Settings Hub.
  - Tái sử dụng components đã có (`SmtpSettingsPage`, `ShiftsRulesPage`) theo chuẩn DRY.
  - Verify: Mở `/workspace/settings/smtp` và `/workspace/settings/shifts-rules` nằm trọn vẹn trong Settings Layout.

### Phase 4: Cập Nhật Test Suite & Drift-Detection (Verification Gate)
- [x] **Task 7: Cập nhật Navigation Unit Tests**
  - File: `webdashboard/src/hooks/__tests__/use-nav-menu-items.test.tsx`
  - File: `webdashboard/src/pages/settings/hooks/__tests__/use-settings-nav.test.tsx`
  - Cập nhật assertions khớp với danh sách menu mới sau điều chuyển.
  - Verify: Chạy unit tests qua vitest pass 100%.

- [x] **Task 8: Kiểm thử tích hợp & E2E smoke check**
  - Kiểm tra 5 vai trò mẫu (`admin`, `hr_manager`, `shift_manager`, `team_lead`, `employee`):
    - `admin`: Thấy Worker Monitor, Audit Logs, Settings đầy đủ.
    - `hr_manager`: Thấy Hồ sơ nhân viên, Phòng ban, Ngày nghỉ lễ, Bảng lương, Không bị loãng bởi SMTP.
    - `shift_manager`: Thấy Xếp ca (`/shifts`), Đổi ca, Ngày nghỉ lễ.
    - `team_lead`: Thấy Hộp thư phê duyệt (`/approvals`).
    - `employee`: Thấy phân hệ tự phục vụ ESS.
  - Verify: Không có console error, không gãy layout, chuyển trang mượt mà.

---

## 🎯 Tiêu Chí Hoàn Thành (Done When)
1. Không còn bất kỳ cấu hình kỹ thuật hạ tầng nào (như SMTP) nằm lẫn trên Menu chính.
2. Không còn trùng lặp giữa `/approvals` và `/approvals/ot`; toàn bộ phê duyệt được quy tụ về một mối.
3. Lịch nghỉ lễ (`holidays`) được đưa ra ngoài phục vụ trực tiếp cho chấm công và xếp ca.
4. Cấu trúc Menu chính giảm tải từ các mục rác, phân định rõ ràng giữa Nghiệp vụ vận hành và Thiết lập cấu hình.
5. Toàn bộ Unit tests và Typecheck pass mà không phát sinh lỗi hồi quy (regression).
