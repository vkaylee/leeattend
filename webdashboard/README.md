# LeeAttend Web Dashboard

Bảng điều khiển quản lý tập trung cho hệ thống LeeAttend, được xây dựng bằng React + Vite + Tailwind CSS và tích hợp với Tauri để đóng gói ứng dụng desktop.

## Công nghệ sử dụng

- **React 19**: Thư viện UI chính.
- **Vite**: Công cụ build và dev server cực nhanh.
- **React Router 7**: Quản lý điều hướng trong ứng dụng SPA.
- **Tailwind CSS 4**: Framework CSS utility-first.
- **Lucide React**: Bộ icon đẹp mắt.
- **TanStack Query**: Quản lý state server và caching.
- **Tauri**: Đóng gói ứng dụng desktop bằng Rust.

## Hướng dẫn phát triển

### 1. Cài đặt

```bash
npm install
```

### 2. Cấu hình

Sao chép file `.env.example` thành `.env.local` và điều chỉnh `VITE_API_URL` trỏ tới địa chỉ của API Server (mặc định là `http://localhost:3000`).

### 3. Chạy môi trường phát triển

```bash
# Chạy web dashboard trong trình duyệt (mặc định port 1420)
npm run dev

# Chạy ứng dụng Desktop (Tauri)
npm run tauri dev
```

### 4. Xây dựng ứng dụng

```bash
# Build web tĩnh
npm run build

# Build ứng dụng Desktop (Tauri)
npm run tauri build
```

## Cấu trúc thư mục

- `src/app.tsx`: Cấu hình Router và Layout chính.
- `src/pages/`: Các trang (Dashboard, Agents, Devices, v.v.).
- `src/components/`: Các component dùng chung.
- `src/lib/api.ts`: Các hàm gọi API tới apiserver.
- `src-tauri/`: Mã nguồn Rust cho ứng dụng Desktop.
