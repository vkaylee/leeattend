import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen } from '@testing-library/react';
import React from 'react';

import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

// Mock react-i18next
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

// Mock hooks
const activePermissions = new Set<string>();
const mockHasAnyPermission = vi.fn((...perms: string[]) =>
  perms.length === 0 || perms.some(p => activePermissions.has(p))
);

vi.mock('@/hooks/use-permissions', () => ({
  useMyPermissions: () => ({
    roles: ['Admin'],
    managedDepartmentIds: null,
    hasAnyPermission: mockHasAnyPermission,
  }),
}));

vi.mock('@/hooks/use-profile', () => ({
  useProfile: () => ({
    user: { full_name: 'John Doe', email: 'john@example.com' },
  }),
}));

vi.mock('@/context/use-dark-mode-context', () => ({
  useDarkModeContext: () => ({
    mode: 'light',
    setMode: vi.fn(),
  }),
}));

vi.mock('@/hooks/use-notifications', () => ({
  useNotifications: () => ({
    notifications: [],
    unreadCount: 0,
    isLoading: false,
    markAsRead: vi.fn(),
    markAllAsRead: vi.fn(),
    fetchNextPage: vi.fn(),
    hasNextPage: false,
    isFetchingNextPage: false,
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

// Mock DropdownMenu to avoid Radix UI + Portals issues in JSDOM
vi.mock('@/components/ui/dropdown-menu', () => ({
  DropdownMenu: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  DropdownMenuTrigger: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  DropdownMenuContent: ({ children, "data-testid": testId }: { children: React.ReactNode; "data-testid"?: string }) => <div data-testid={testId}>{children}</div>,
  DropdownMenuItem: ({ children, onClick, "data-testid": testId }: { children: React.ReactNode; onClick?: () => void; "data-testid"?: string }) => (
    <div data-testid={testId} onClick={onClick} onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') onClick?.(); }} role="button" tabIndex={0}>{children}</div>
  ),
  DropdownMenuLabel: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  DropdownMenuSeparator: () => <hr />,
}));

// Mock components that might cause issues
vi.mock('@/components/ui/language-switcher', () => ({
  LanguageSwitcher: () => <div data-testid="language-switcher-mock" />,
}));

vi.mock('../notifications-popover', () => ({
  NotificationsPopover: () => <div data-testid="notifications-popover-mock" />,
}));

import Navbar from '../navbar';

const createTestQueryClient = () =>
  new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } });

const renderWithProviders = (ui: React.ReactElement) => {
  return render(
    <QueryClientProvider client={createTestQueryClient()}>
      {ui}
    </QueryClientProvider>
  );
};

describe('Navigation Layout Consistency', () => {
  beforeEach(() => {
    activePermissions.clear();
    activePermissions.add("employees:read");
    activePermissions.add("members:read");
    activePermissions.add("departments:read");
    activePermissions.add("attendance:logs:read");
    activePermissions.add("leave:requests:read");
    activePermissions.add("leave:balances:read");
    activePermissions.add("attendance:adjustments:approve");
    activePermissions.add("shift_swaps:manage");
    activePermissions.add("reports:read");
    activePermissions.add("overtime:approve");
    activePermissions.add("shifts:read");
    activePermissions.add("payroll:read");
  });

  describe('Navbar', () => {
    it('renders organization name in breadcrumbs', () => {
      renderWithProviders(
        <Navbar
          organizationName="Test Organization"
        />
      );

      expect(screen.getByText('Test Organization')).toBeInTheDocument();
    });

    it('renders page name from current path', () => {
      renderWithProviders(
        <Navbar
          organizationName="Test Organization"
        />
      );

      // Path is "/" which maps to "common.dashboard" → resolves to "common.dashboard" in mock
      const pageName = screen.getByText('COMMON.DASHBOARD');
      expect(pageName).toBeInTheDocument();
    });

    it('renders header at 56px height', () => {
      const { container } = renderWithProviders(
        <Navbar
          organizationName="Test Organization"
        />
      );

      const header = container.querySelector('header');
      expect(header).toBeInTheDocument();
      expect(header!.className).toContain('h-14');
    });
  });
});
