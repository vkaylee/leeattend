// Trong môi trường Tauri hoặc Production build, chúng ta cần một địa chỉ tuyệt đối nếu không qua proxy.
// Vite sẽ thay thế import.meta.env.VITE_API_URL lúc build.
const BASE_URL_FROM_ENV = import.meta.env.VITE_API_URL?.replace(/\/$/, "");

// Nếu đang chạy web dev (có proxy) thì dùng "/api", nếu build tauri thì dùng URL tuyệt đối.
const isTauri = typeof window !== "undefined" && (window as any).__TAURI_INTERNALS__;

// LOGIC CẢI TIẾN:
// 1. Web Dev: Luôn dùng "/api" để tận dụng Vite Proxy (nơi đã xử lý dynamic port từ .api_port).
//    Tránh lấy trực tiếp BASE_URL_FROM_ENV vì nó thường hardcode port 3000 gây lỗi.
// 2. Tauri: Cần URL tuyệt đối, ưu tiên ENV -> mặc định localhost:3000.
// 3. Web Prod: Dùng ENV (nếu API khác domain) hoặc fallback về "/api".
export const API_BASE_URL = isTauri 
  ? (BASE_URL_FROM_ENV || "http://localhost:3000")
  : (import.meta.env.DEV ? "/api" : (BASE_URL_FROM_ENV || "/api"));

export const WS_BASE_URL = BASE_URL_FROM_ENV
  ? `${BASE_URL_FROM_ENV.replace(/^http/, "ws")}/ws`
  : `${window.location.protocol === "https:" ? "wss:" : "ws:"}//${isTauri ? "localhost:3000" : window.location.host}/ws`;
