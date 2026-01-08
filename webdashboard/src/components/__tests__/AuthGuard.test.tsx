import React from 'react';
import { render, screen } from '@testing-library/react';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import AuthGuard from '../AuthGuard';
import * as api from '@/lib/api';
import { useProfile } from '@/hooks/use-profile';
import { vi, describe, it, expect, beforeEach } from 'vitest';

vi.mock('@/lib/api', () => ({
  getCurrentUserId: vi.fn(),
  clearAuthSession: vi.fn(),
}));

vi.mock('@/hooks/use-profile', () => ({
  useProfile: vi.fn(),
}));

const MockLogin = () => <div>Login Page</div>;
const MockChild = () => <div>Dashboard</div>;

describe('AuthGuard', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('redirects to /login if no user id is found', () => {
    (api.getCurrentUserId as any).mockReturnValue(null);
    (useProfile as any).mockReturnValue({ user: null, isLoading: false, isError: false });

    render(
      <MemoryRouter initialEntries={['/']}>
        <Routes>
          <Route element={<AuthGuard />}>
            <Route path="/" element={<MockChild />} />
          </Route>
          <Route path="/login" element={<MockLogin />} />
        </Routes>
      </MemoryRouter>
    );

    expect(screen.getByText(/Không tìm thấy thông tin người dùng/i)).toBeInTheDocument();
  });

  it('shows loading spinner when fetching profile', () => {
    (api.getCurrentUserId as any).mockReturnValue('user-123');
    (useProfile as any).mockReturnValue({ user: null, isLoading: true, isError: false });

    render(
      <MemoryRouter initialEntries={['/']}>
        <AuthGuard />
      </MemoryRouter>
    );

    expect(screen.getByText(/Đang kiểm tra bảo mật/i)).toBeInTheDocument();
  });

  it('renders children when authenticated and profile exists', () => {
    (api.getCurrentUserId as any).mockReturnValue('user-123');
    (useProfile as any).mockReturnValue({ 
      user: { id: 'user-123', email: 'test@example.com' }, 
      isLoading: false, 
      isError: false 
    });

    render(
      <MemoryRouter initialEntries={['/']}>
        <Routes>
          <Route element={<AuthGuard />}>
            <Route path="/" element={<MockChild />} />
          </Route>
        </Routes>
      </MemoryRouter>
    );

    expect(screen.getByText('Dashboard')).toBeInTheDocument();
  });

  it('redirects to /login if profile fetch fails', () => {
    (api.getCurrentUserId as any).mockReturnValue('user-invalid');
    (useProfile as any).mockReturnValue({ user: null, isLoading: false, isError: true });

    const { container } = render(
      <MemoryRouter initialEntries={['/']}>
        <Routes>
          <Route element={<AuthGuard />}>
            <Route path="/" element={<MockChild />} />
          </Route>
          <Route path="/login" element={<MockLogin />} />
        </Routes>
      </MemoryRouter>
    );

    // Khi isError: true, AuthGuard hiện thông báo chuyển hướng
    expect(screen.getByText(/Phiên làm việc hết hạn/i)).toBeInTheDocument();
  });
});