import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: 'export',
  images: {
    unoptimized: true,
  },
  /* Thêm các cấu hình tối ưu hóa khác nếu cần */
};

export default nextConfig;