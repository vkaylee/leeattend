import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';

import { useI18nSync } from '../use-i18n-sync';

// Mock i18n
const mockChangeLanguage = vi.fn().mockResolvedValue(undefined);

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      language: 'vi',
      changeLanguage: mockChangeLanguage,
    },
  }),
}));

// Mock useProfile
vi.mock('../use-profile', () => ({
  useProfile: () => ({
    user: { id: '1', preferred_language: 'en' },
    isLoading: false,
  }),
}));

// Mock useWorkspaces
vi.mock('../use-workspace', () => ({
  useWorkspaces: () => ({
    workspaces: [{ id: 'ws1', default_language: 'vi' }],
    isLoading: false,
  }),
}));

// Mock getCurrentWorkspaceId
vi.mock('@/lib/api', () => ({
  getCurrentWorkspaceId: () => 'ws1',
}));

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};
Object.defineProperty(window, 'localStorage', { value: localStorageMock });

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
};

describe('useI18nSync', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('respects localStorage language and does not override', () => {
    localStorageMock.getItem.mockReturnValue('zh');

    renderHook(() => useI18nSync(), { wrapper: createWrapper() });

    // Should NOT call changeLanguage because localStorage has a value
    expect(mockChangeLanguage).not.toHaveBeenCalled();
  });

  it('syncs to user preference when no localStorage value', () => {
    localStorageMock.getItem.mockReturnValue(null);

    renderHook(() => useI18nSync(), { wrapper: createWrapper() });

    // Should sync to user's preferred_language (en)
    expect(mockChangeLanguage).toHaveBeenCalledWith('en');
    expect(localStorageMock.setItem).toHaveBeenCalledWith('i18nextLng', 'en');
  });

  it('does not sync if localStorage has valid language', () => {
    localStorageMock.getItem.mockReturnValue('en');

    renderHook(() => useI18nSync(), { wrapper: createWrapper() });

    // Should respect localStorage
    expect(mockChangeLanguage).not.toHaveBeenCalled();
  });

  it('ignores invalid localStorage values', () => {
    localStorageMock.getItem.mockReturnValue('invalid-lang');

    renderHook(() => useI18nSync(), { wrapper: createWrapper() });

    // Should sync because localStorage value is invalid
    expect(mockChangeLanguage).toHaveBeenCalledWith('en');
  });
});
