import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import { LayoutDashboard, Cpu, History, Settings } from 'lucide-react';
import Link from 'next/link';

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "LeeAttend Central",
  description: "Hệ thống quản lý chấm công tập trung",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="vi">
      <body className={`${inter.className} bg-slate-50 text-slate-900`}>
        <div className="flex min-h-screen">
          {/* Sidebar */}
          <aside className="w-64 bg-white border-r border-slate-200 flex flex-col sticky top-0 h-screen">
            <div className="p-6">
              <div className="flex items-center gap-3 px-2">
                <div className="bg-blue-600 p-2 rounded-xl shadow-lg shadow-blue-200">
                  <Cpu className="text-white w-6 h-6" />
                </div>
                <span className="font-bold text-xl tracking-tight">LeeAttend</span>
              </div>
            </div>

            <nav className="flex-1 px-4 space-y-1">
              <Link href="/" className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group">
                <LayoutDashboard className="w-5 h-5 group-hover:scale-110 transition-transform" />
                <span className="font-semibold">Bảng điều khiển</span>
              </Link>
              <Link href="/devices" className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group">
                <Cpu className="w-5 h-5 group-hover:scale-110 transition-transform" />
                <span className="font-semibold">Thiết bị</span>
              </Link>
              <Link href="/logs" className="flex items-center gap-3 px-4 py-3 text-slate-600 hover:bg-slate-50 hover:text-blue-600 rounded-2xl transition-all group">
                <History className="w-5 h-5 group-hover:scale-110 transition-transform" />
                <span className="font-semibold">Nhật ký</span>
              </Link>
            </nav>

            <div className="p-6 border-t border-slate-100">
              <Link href="/settings" className="flex items-center gap-3 px-4 py-2 text-slate-400 hover:text-slate-600 transition-colors">
                <Settings className="w-5 h-5" />
                <span className="text-sm font-medium">Cài đặt</span>
              </Link>
            </div>
          </aside>

          {/* Main Content */}
          <main className="flex-1 overflow-y-auto">
            {children}
          </main>
        </div>
      </body>
    </html>
  );
}