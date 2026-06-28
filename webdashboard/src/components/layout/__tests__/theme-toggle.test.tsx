import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';

import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

import type { ThemeMode } from '@/hooks/use-dark-mode';

import { DarkModeContext } from '@/context/dark-mode-context-definition';

// Mock hooks needed by IconRail
vi.mock('@/hooks/use-notifications', () => ({
  useNotifications: () => ({
    notifications: [],
    unreadCount: 0,
    isLoading: false,
  }),
}));

vi.mock('@/hooks/use-language', () => ({
  useLanguage: () => ({
    currentLanguage: 'vi',
    currentOption: { code: 'vi', flag: '🇻🇳', name: 'Vietnamese', nativeName: 'Tiếng Việt' },
    changeLanguage: vi.fn(),
    languages: [],
  }),
}));

vi.mock('@/hooks/use-permissions', () => ({
  useMyPermissions: () => ({
    hasAnyPermission: vi.fn(() => true),
    permissions: [],
    roles: [],
    managedDepartmentIds: null,
    isLoading: false,
  }),
}));

vi.mock('@/hooks/use-profile', () => ({
  useProfile: () => ({
    user: { full_name: 'Test User', email: 'test@test.com', system_roles: [] },
    isLoading: false,
  }),
}));

vi.mock('@/components/ui/language-switcher', () => ({
  LanguageSwitcher: () => <div data-testid="language-switcher-mock" />,
}));

import IconRail from '../icon-rail';

const mockSetMode = vi.fn();

const createTestQueryClient = () =>
  new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } });

const defaultProps = {
  activeGroup: null as any,
  isPinned: false,
  onGroupHover: vi.fn(),
  onGroupClick: vi.fn(),
  onTogglePin: vi.fn(),
  workspaceName: 'Test',
  onShowWorkspaceSwitcher: vi.fn(),
};

function renderWithContext(mode: ThemeMode = 'system') {
  const contextValue = {
    mode,
    resolved: (mode === 'dark' ? 'dark' : 'light') as 'light' | 'dark',
    isDark: mode === 'dark',
    setMode: mockSetMode,
  };

  return render(
    <QueryClientProvider client={createTestQueryClient()}>
      <DarkModeContext.Provider value={contextValue}>
        <IconRail {...defaultProps} />
      </DarkModeContext.Provider>
    </QueryClientProvider>
  );
}

describe('ThemeToggle (in IconRail)', () => {
  beforeEach(() => {
    mockSetMode.mockClear();
  });

  it('renders the theme toggle button in the rail', () => {
    renderWithContext();
    expect(screen.getByTestId('nav-rail-theme-toggle')).toBeInTheDocument();
  });

  it('clicking cycles to next theme (light → system)', () => {
    renderWithContext('light');
    fireEvent.click(screen.getByTestId('nav-rail-theme-toggle'));
    expect(mockSetMode).toHaveBeenCalledWith('system');
  });

  it('clicking cycles from system to dark', () => {
    renderWithContext('system');
    fireEvent.click(screen.getByTestId('nav-rail-theme-toggle'));
    expect(mockSetMode).toHaveBeenCalledWith('dark');
  });

  it('clicking cycles from dark to light', () => {
    renderWithContext('dark');
    fireEvent.click(screen.getByTestId('nav-rail-theme-toggle'));
    expect(mockSetMode).toHaveBeenCalledWith('light');
  });

  it('button has an accessible label', () => {
    renderWithContext('light');
    const btn = screen.getByTestId('nav-rail-theme-toggle');
    expect(btn.getAttribute('aria-label')).toBeTruthy();
  });
});
