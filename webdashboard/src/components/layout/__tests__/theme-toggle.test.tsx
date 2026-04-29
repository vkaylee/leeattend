import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: any) => <a href={to} {...props}>{children}</a>,
}));
import { DarkModeContext } from '@/context/dark-mode-context-definition';

import Navbar from '../Navbar';

import type { ThemeMode } from '@/hooks/use-dark-mode';

// Mock dependencies that Navbar's children need
vi.mock('@/components/ui/LanguageSwitcher', () => ({
  LanguageSwitcher: () => <div data-testid="language-switcher-mock" />,
}));

const mockSetMode = vi.fn();

const createTestQueryClient = () =>
  new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } });

function renderWithContext(mode: ThemeMode = 'system') {
  const contextValue = {
    mode,
    resolved: mode === 'dark' ? 'dark' as const : 'light' as const,
    isDark: mode === 'dark',
    setMode: mockSetMode,
  };

  return render(
    <QueryClientProvider client={createTestQueryClient()}>
      <DarkModeContext.Provider value={contextValue}>
        <Navbar
          currentWorkspaceName="Test Workspace"
          onOpenSidebar={vi.fn()}
          isCollapsed={false}
          onToggleCollapsed={vi.fn()}
        />
      </DarkModeContext.Provider>
    </QueryClientProvider>
  );
}

describe('ThemeToggle (in Navbar)', () => {
  beforeEach(() => {
    mockSetMode.mockClear();
  });

  it('renders the theme toggle with 3 buttons', () => {
    renderWithContext();
    expect(screen.getByTestId('theme-toggle')).toBeInTheDocument();
    expect(screen.getByTestId('theme-toggle-light')).toBeInTheDocument();
    expect(screen.getByTestId('theme-toggle-system')).toBeInTheDocument();
    expect(screen.getByTestId('theme-toggle-dark')).toBeInTheDocument();
  });

  it('clicking dark button calls setMode("dark")', () => {
    renderWithContext('light');
    fireEvent.click(screen.getByTestId('theme-toggle-dark'));
    expect(mockSetMode).toHaveBeenCalledWith('dark');
  });

  it('clicking light button calls setMode("light")', () => {
    renderWithContext('dark');
    fireEvent.click(screen.getByTestId('theme-toggle-light'));
    expect(mockSetMode).toHaveBeenCalledWith('light');
  });

  it('clicking system button calls setMode("system")', () => {
    renderWithContext('dark');
    fireEvent.click(screen.getByTestId('theme-toggle-system'));
    expect(mockSetMode).toHaveBeenCalledWith('system');
  });

  it('active button has highlighted style', () => {
    renderWithContext('dark');
    const darkBtn = screen.getByTestId('theme-toggle-dark');
    expect(darkBtn.className).toContain('bg-white');
  });

  it('inactive buttons do not have highlighted style', () => {
    renderWithContext('dark');
    const lightBtn = screen.getByTestId('theme-toggle-light');
    expect(lightBtn.className).not.toContain('bg-white');
  });
});
