import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useDarkMode } from '../use-dark-mode';

describe('useDarkMode', () => {
  let mockMatchMedia: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    localStorage.clear();
    document.documentElement.classList.remove('dark');

    // Mock matchMedia
    mockMatchMedia = vi.fn().mockReturnValue({
      matches: false,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    });
    window.matchMedia = mockMatchMedia;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('defaults to system mode when no stored preference', () => {
    const { result } = renderHook(() => useDarkMode());
    expect(result.current.mode).toBe('system');
  });

  it('reads stored preference from localStorage', () => {
    localStorage.setItem('theme_mode', 'dark');
    const { result } = renderHook(() => useDarkMode());
    expect(result.current.mode).toBe('dark');
    expect(result.current.isDark).toBe(true);
  });

  it('setMode("dark") applies dark class and persists', () => {
    const { result } = renderHook(() => useDarkMode());

    act(() => { result.current.setMode('dark'); });

    expect(result.current.mode).toBe('dark');
    expect(result.current.isDark).toBe(true);
    expect(document.documentElement.classList.contains('dark')).toBe(true);
    expect(localStorage.getItem('theme_mode')).toBe('dark');
  });

  it('setMode("light") removes dark class', () => {
    document.documentElement.classList.add('dark');
    const { result } = renderHook(() => useDarkMode());

    act(() => { result.current.setMode('light'); });

    expect(result.current.mode).toBe('light');
    expect(result.current.isDark).toBe(false);
    expect(document.documentElement.classList.contains('dark')).toBe(false);
    expect(localStorage.getItem('theme_mode')).toBe('light');
  });

  it('system mode resolves based on matchMedia', () => {
    mockMatchMedia.mockReturnValue({
      matches: true, // system prefers dark
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    });

    const { result } = renderHook(() => useDarkMode());
    expect(result.current.mode).toBe('system');
    expect(result.current.resolved).toBe('dark');
    expect(result.current.isDark).toBe(true);
  });

  it('system mode with light system preference resolves to light', () => {
    mockMatchMedia.mockReturnValue({
      matches: false, // system prefers light
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    });

    const { result } = renderHook(() => useDarkMode());
    expect(result.current.mode).toBe('system');
    expect(result.current.resolved).toBe('light');
    expect(result.current.isDark).toBe(false);
  });

  it('dispatches theme-mode-change event on setMode', () => {
    const handler = vi.fn();
    window.addEventListener('theme-mode-change', handler);

    const { result } = renderHook(() => useDarkMode());
    act(() => { result.current.setMode('dark'); });

    expect(handler).toHaveBeenCalledTimes(1);
    expect((handler.mock.calls[0][0] as CustomEvent).detail).toBe('dark');

    window.removeEventListener('theme-mode-change', handler);
  });

  it('cycling through modes works correctly', () => {
    const { result } = renderHook(() => useDarkMode());

    act(() => { result.current.setMode('dark'); });
    expect(result.current.isDark).toBe(true);

    act(() => { result.current.setMode('light'); });
    expect(result.current.isDark).toBe(false);

    act(() => { result.current.setMode('system'); });
    expect(result.current.mode).toBe('system');
  });
});
