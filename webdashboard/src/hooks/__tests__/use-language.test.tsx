import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook, act } from '@testing-library/react';

import { describe, it, expect, vi, beforeEach } from 'vitest';

import { useLanguage, SUPPORTED_LANGUAGES } from '../use-language';

// Mock i18n
const mockChangeLanguage = vi.fn().mockResolvedValue(undefined);
const mockOn = vi.fn();
const mockOff = vi.fn();

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      language: 'vi',
      changeLanguage: mockChangeLanguage,
      on: mockOn,
      off: mockOff,
    },
  }),
}));

// Mock useProfile
const mockUpdateProfile = vi.fn().mockResolvedValue({});

vi.mock('../use-profile', () => ({
  useProfile: () => ({
    user: { id: '1', preferred_language: 'vi' },
    updateProfile: mockUpdateProfile,
    isUpdating: false,
  }),
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
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
};

describe('useLanguage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorageMock.getItem.mockReturnValue(null);
  });

  it('returns current language and options', () => {
    const { result } = renderHook(() => useLanguage(), { wrapper: createWrapper() });

    expect(result.current.currentLanguage).toBe('vi');
    expect(result.current.currentOption.code).toBe('vi');
    expect(result.current.languages).toEqual(SUPPORTED_LANGUAGES);
  });

  it('provides all supported languages', () => {
    const { result } = renderHook(() => useLanguage(), { wrapper: createWrapper() });

    expect(result.current.languages).toHaveLength(3);
    expect(result.current.languages.map(l => l.code)).toEqual(['vi', 'en', 'zh']);
  });

  it('changeLanguage calls i18n.changeLanguage', async () => {
    const { result } = renderHook(() => useLanguage(), { wrapper: createWrapper() });

    await act(async () => {
      await result.current.changeLanguage('en');
    });

    expect(mockChangeLanguage).toHaveBeenCalledWith('en');
  });

  it('changeLanguage saves to localStorage', async () => {
    const { result } = renderHook(() => useLanguage(), { wrapper: createWrapper() });

    await act(async () => {
      await result.current.changeLanguage('en');
    });

    expect(localStorageMock.setItem).toHaveBeenCalledWith('i18nextLng', 'en');
  });

  it('changeLanguage syncs to server via updateProfile', async () => {
    const { result } = renderHook(() => useLanguage(), { wrapper: createWrapper() });

    await act(async () => {
      await result.current.changeLanguage('zh');
    });

    expect(mockUpdateProfile).toHaveBeenCalledWith({ preferred_language: 'zh' });
  });

  it('changeLanguage handles API errors silently', async () => {
    mockUpdateProfile.mockRejectedValueOnce(new Error('API Error'));
    const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => { /* empty */ });

    const { result } = renderHook(() => useLanguage(), { wrapper: createWrapper() });

    await act(async () => {
      await result.current.changeLanguage('en');
    });

    // Should not throw, just warn
    expect(consoleSpy).toHaveBeenCalledWith('Failed to sync language preference to server');
    // i18n should still change
    expect(mockChangeLanguage).toHaveBeenCalledWith('en');

    consoleSpy.mockRestore();
  });

  it('registers languageChanged event listener on mount', () => {
    renderHook(() => useLanguage(), { wrapper: createWrapper() });

    expect(mockOn).toHaveBeenCalledWith('languageChanged', expect.any(Function));
  });

  it('unregisters languageChanged event listener on unmount', () => {
    const { unmount } = renderHook(() => useLanguage(), { wrapper: createWrapper() });

    unmount();

    expect(mockOff).toHaveBeenCalledWith('languageChanged', expect.any(Function));
  });
});
