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
import Sidebar from '../sidebar';

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

  describe('Sidebar', () => {
    it('should not contain the logout button anymore', () => {
      renderWithProviders(
        <Sidebar
          isCollapsed={false}
          isOpen={true}
          onClose={vi.fn()}
          onShowSwitcher={vi.fn()}
          openSections={{}}
          toggleSection={vi.fn()}
        />
      );

      // Search for logout text (t("common.logout") returns "common.logout" in our mock)
      expect(screen.queryByText(/common\.logout/i)).not.toBeInTheDocument();
      expect(screen.queryByTestId('user-menu-logout')).not.toBeInTheDocument();
    });
  });

  describe('Navbar', () => {
    it('should contain the user profile trigger', () => {
      renderWithProviders(
        <Navbar
          currentWorkspaceName="Test Workspace"
          isCollapsed={false}
          onOpenSidebar={vi.fn()}
          onToggleCollapsed={vi.fn()}
        />
      );

      expect(screen.getByTestId('navbar-user-menu-trigger')).toBeInTheDocument();
      expect(screen.getByText(/John Doe/i)).toBeInTheDocument();
    });

    it('should show user menu with logout when clicking the trigger', async () => {
      renderWithProviders(
        <Navbar 
          currentWorkspaceName="Test Workspace"
          isCollapsed={false}
          onOpenSidebar={vi.fn()}
          onToggleCollapsed={vi.fn()}
        />
      );

      const trigger = screen.getByTestId('navbar-user-menu-trigger');
      fireEvent.click(trigger);

      // Radix UI DropdownMenu uses Portals, but RTL screen.getBy should find it
      // Note: "common.logout" is what t("common.logout") returns in our mock
      expect(await screen.findByTestId('user-menu-logout-btn')).toBeInTheDocument();
      expect(screen.getByTestId('user-menu-profile-btn')).toBeInTheDocument();
      expect(screen.getByTestId('user-menu-settings-btn')).toBeInTheDocument();
    });
  });

  describe('Sidebar Permissions Gating', () => {
    const renderSidebar = () =>
      renderWithProviders(
        <Sidebar
          isCollapsed={false}
          isOpen={true}
          onClose={vi.fn()}
          onShowSwitcher={vi.fn()}
          openSections={{ time: true }}
          toggleSection={vi.fn()}
        />
      );

    it('shows Leave Requests menu item when user has leave:requests:read', () => {
      activePermissions.clear();
      activePermissions.add("leave:requests:read");
      renderSidebar();
      expect(screen.getByTestId('sidebar-link-leave-requests')).toBeInTheDocument();
    });

    it('shows Leave Requests menu item when user has leave:requests:submit_on_behalf', () => {
      activePermissions.clear();
      activePermissions.add("leave:requests:submit_on_behalf");
      renderSidebar();
      expect(screen.getByTestId('sidebar-link-leave-requests')).toBeInTheDocument();
    });

    it('shows Leave Requests menu item when user has leave:requests:approve', () => {
      activePermissions.clear();
      activePermissions.add("leave:requests:approve");
      renderSidebar();
      expect(screen.getByTestId('sidebar-link-leave-requests')).toBeInTheDocument();
    });

    it('hides Leave Requests menu item when user has none of these permissions', () => {
      activePermissions.clear();
      renderSidebar();
      expect(screen.queryByTestId('sidebar-link-leave-requests')).not.toBeInTheDocument();
    });
  });
});
