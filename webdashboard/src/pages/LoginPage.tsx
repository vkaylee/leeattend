import React from 'react';
import { Cpu } from 'lucide-react';
import { fetchDevSuperUser, setCurrentUserId, setCurrentWorkspaceId, getCurrentUserId } from '@/lib/api';
import { Navigate } from 'react-router-dom';

export default function LoginPage() {
  const userId = getCurrentUserId();

  // Redirect if already logged in
  if (userId) {
    return <Navigate to="/" replace />;
  }

  const handleFastLogin = async () => {
    try {
      const { user_id, workspace_id } = await fetchDevSuperUser();
      setCurrentUserId(user_id);
      setCurrentWorkspaceId(workspace_id);
      window.location.reload();
    } catch (error) {
      console.error('Fast login failed:', error);
      alert('Fast login failed. Make sure backend is running in debug mode and DB is seeded.');
    }
  };

  return (
    <div className="min-h-screen bg-slate-50 flex flex-col items-center justify-center p-6">
      <div className="w-full max-w-md bg-white rounded-3xl shadow-xl shadow-slate-200 border border-slate-100 p-8">
        <div className="flex flex-col items-center gap-4 mb-8">
          <div className="bg-blue-600 p-3 rounded-2xl shadow-lg shadow-blue-200">
            <Cpu className="text-white w-8 h-8" />
          </div>
          <div className="text-center">
            <h1 className="text-2xl font-bold text-slate-900">LeeAttend</h1>
            <p className="text-slate-500 font-medium">Hệ thống quản lý máy chấm công</p>
          </div>
        </div>

        <div className="space-y-4">
          <div className="p-4 bg-slate-50 rounded-2xl border border-slate-100">
            <p className="text-sm text-slate-600 text-center mb-4">
              Dự án đang trong giai đoạn phát triển. Sử dụng nút đăng nhập nhanh bên dưới để bắt đầu.
            </p>
            <button
              onClick={handleFastLogin}
              className="w-full py-3 bg-blue-600 text-white rounded-xl font-bold hover:bg-blue-700 transition-all flex items-center justify-center gap-2 shadow-lg shadow-blue-100"
            >
              ⚡ Đăng nhập nhanh (Dev Mode)
            </button>
          </div>

          <div className="text-center">
            <span className="text-xs text-slate-400 font-medium uppercase tracking-widest">Hoặc</span>
          </div>

          <div className="space-y-3">
            <input 
              type="text" 
              placeholder="User ID (UUID)"
              className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all"
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  const val = (e.target as HTMLInputElement).value;
                  if (val) {
                    setCurrentUserId(val);
                    window.location.reload();
                  }
                }
              }}
            />
            <p className="text-[10px] text-slate-400 text-center px-4">
              Nhập User ID thủ công nếu bạn muốn đăng nhập vào một tài khoản cụ thể.
            </p>
          </div>
        </div>
      </div>
      
      <p className="mt-8 text-slate-400 text-sm font-medium">
        &copy; 2026 LeeAttend Team. All rights reserved.
      </p>
    </div>
  );
}
