import { useNavigate, Navigate, Link } from 'react-router-dom';
import { useProfile } from '@/hooks/use-profile';
import { fetchDevSuperUser, setCurrentUserId, setCurrentWorkspaceId, getCurrentUserId, setAccessToken, setRefreshToken } from '@/lib/api';
import { useState } from 'react';

export default function LoginPage() {
  const userId = getCurrentUserId();
  const [error, setError] = useState<string | null>(null);

  // Redirect if already logged in
  if (userId) {
    return <Navigate to="/" replace />;
  }

  const handleFastLogin = async () => {
    try {
      const { user_id, workspace_id, access_token, refresh_token } = await fetchDevSuperUser();
      setCurrentUserId(user_id);
      setCurrentWorkspaceId(workspace_id);
      if (access_token) setAccessToken(access_token);
      if (refresh_token) setRefreshToken(refresh_token);
      window.location.reload();
    } catch (error) {
      console.error('Login failed', error);
      setError('Đăng nhập thất bại. Vui lòng thử lại.');
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-slate-50">
      <div className="max-w-md w-full space-y-8 p-8 bg-white rounded-xl shadow-lg">
        <div className="text-center">
          <h2 className="mt-6 text-3xl font-bold text-gray-900">
            LeeAttend
          </h2>
          <p className="mt-2 text-sm text-gray-600">
            Hệ thống quản lý chấm công & thiết bị
          </p>
        </div>

        {error && (
          <div className="bg-red-50 text-red-600 p-3 rounded text-sm text-center">
            {error}
          </div>
        )}

        <div className="mt-8 space-y-6">
          <button
            onClick={handleFastLogin}
            className="group relative w-full flex justify-center py-3 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-colors"
          >
            ⚡ Đăng nhập nhanh (Dev Mode)
          </button>
          
          <div className="relative">
            <div className="absolute inset-0 flex items-center">
              <div className="w-full border-t border-gray-300" />
            </div>
            <div className="relative flex justify-center text-sm">
              <span className="px-2 bg-white text-gray-500">
                Hoặc đăng nhập bằng Email
              </span>
            </div>
          </div>

          <form className="mt-8 space-y-6" action="#" method="POST">
            <div className="rounded-md shadow-sm -space-y-px">
              <div>
                <label htmlFor="email-address" className="sr-only">Email address</label>
                <input
                  id="email-address"
                  name="email"
                  type="email"
                  autoComplete="email"
                  required
                  className="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
                  placeholder="Email address"
                />
              </div>
              <div>
                <label htmlFor="password" className="sr-only">Password</label>
                <input
                  id="password"
                  name="password"
                  type="password"
                  autoComplete="current-password"
                  required
                  className="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-b-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
                  placeholder="Password"
                />
              </div>
            </div>

            <div>
              <button
                type="submit"
                className="group relative w-full flex justify-center py-3 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-gray-600 hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500 transition-colors"
              >
                Đăng nhập (Coming Soon)
              </button>
            </div>

            <div className="text-center text-sm">
              <span className="text-gray-500">Don't have an account? </span>
              <Link to="/register" className="font-medium text-indigo-600 hover:text-indigo-500">
                Sign up
              </Link>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}