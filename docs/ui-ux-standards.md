# Tiêu chuẩn UI/UX LeeAttend

Tài liệu này quy định các nguyên tắc thiết kế và trải nghiệm người dùng (UX) cho hệ thống LeeAttend nhằm đảm bảo tính chuyên nghiệp, thẩm mỹ và hiệu suất vận hành cao.

---

## 1. Triết lý thiết kế (Design Philosophy)

### 1.1. Zero-Distraction (Không xao nhãng)
Giao diện phải tập trung vào dữ liệu nghiệp vụ. Loại bỏ các yếu tố trang trí dư thừa.
- **Header:** Chỉ chứa Breadcrumbs tĩnh và thông báo (nếu có). Không đặt các menu điều hướng chính tại đây.
- **Sidebar:** Sử dụng cơ chế "Single-open accordion" (chỉ mở một nhóm menu duy nhất). Nhóm menu "Hệ thống" luôn đóng mặc định.

### 1.2. Premium Aesthetics (Thẩm mỹ cao cấp)
- Sử dụng các góc bo tròn lớn (`rounded-2xl` hoặc `rounded-3xl`) cho Card và Modal.
- Hệ thống Shadow (đổ bóng) nhẹ nhàng, ưu tiên sử dụng màu Shadow cùng tông với màu chủ đạo (ví dụ: `shadow-violet-100`).
- Sử dụng font chữ không chân (Sans-serif) với khoảng cách dòng thoáng đãng.

---

## 2. Hệ thống màu sắc (Color System)

### 2.1. Dynamic Themeing
Hệ thống sử dụng các biến CSS để điều khiển màu sắc chủ đạo:
- `--primary-50` đến `--primary-950`.
- Màu mặc định: **Violet (#7c3aed)**.

### 2.2. Thứ tự ưu tiên màu sắc (Color Priority)
1. **User Override:** Màu do người dùng tự chọn trong trang Hồ sơ (Lưu tại LocalStorage).
2. **Workspace Theme:** Màu đại diện cho thương hiệu của doanh nghiệp (Lưu tại Database).
3. **System Default:** Màu Violet mặc định.

---

## 3. Quy tắc Layout & Responsive

### 3.1. Phân cấp khu vực
- **Sidebar (Trái):** Điều hướng chính và Chuyển đổi Workspace.
- **Header (Trên):** Chỉ dẫn vị trí (Breadcrumbs).
- **Main Content (Giữa):** Khu vực hiển thị dữ liệu và thao tác.

### 3.2. Responsive (Màn hình nhỏ)
- **Dưới 1024px (lg):** Sidebar chuyển thành dạng Drawer.
- **Backdrop:** Phải có lớp phủ mờ (`backdrop-blur-sm`) khi mở Sidebar trên mobile.
- **Touch Targets:** Các phần tử tương tác (Button, Input) phải có chiều cao tối thiểu 44px.

---

## 4. Tương tác & Hiệu ứng (Interactions)

### 4.1. Chuyển động (Animations)
Sử dụng Tailwind Animate cho mọi thành phần mới xuất hiện:
- Trang mới: `animate-in fade-in duration-500`.
- Modal/Dropdown: `animate-in zoom-in-95 slide-in-from-top-2`.

### 4.2. Phản hồi trạng thái (Feedback)
- **Thao tác thành công:** Luôn hiển thị Toast màu xanh lá.
- **Thao tác lỗi:** Toast màu đỏ kèm mô tả ngắn gọn nguyên nhân.
- **Loading:** Ưu tiên dùng **Skeleton Loaders** thay vì Spinner xoay tròn để giảm cảm giác chờ đợi.

---

## 5. Quy chuẩn linh kiện (Component Standards)

### 5.1. Buttons
- **Primary:** Nền màu chủ đạo, chữ trắng, shadow theo màu chủ đạo.
- **Secondary:** Nền xám nhạt hoặc trắng, viền mảnh.
- **Danger:** Chỉ dùng màu đỏ cho các hành động không thể hoàn tác (Xóa vĩnh viễn).

### 5.2. Tables
- Sử dụng nền trắng, viền cực mảnh hoặc không viền (chỉ kẻ dòng).
- Hỗ trợ trạng thái **Hover** trên từng dòng.
- Header của bảng luôn viết hoa, font size nhỏ (`text-xs`) và đậm.

---

## 6. Cá nhân hóa (Personalization)

Người dùng có quyền can thiệp vào giao diện mà không ảnh hưởng đến người khác.
- Cho phép thay đổi màu sắc chủ đạo.
- Tự động lưu và nhận diện màu sắc theo trình duyệt (`localStorage`) để đảm bảo tốc độ phản hồi ngay khi vừa tải trang.
