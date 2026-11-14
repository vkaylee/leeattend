import { renderHook, act } from '@testing-library/react';

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

import * as useOrganizationHook from '../use-organization';
import { useTheme } from '../use-theme';

// Mock useOrganization
vi.mock('../use-organization', () => ({
  useOrganization: vi.fn(),
}));

describe('useTheme', () => {
  const defaultColor = '#10b981';
  const workspaceColor = '#123456';
  const userColor = '#ff0000';

  beforeEach(() => {
    // Clear localStorage
    localStorage.clear();
    // Reset mocks
    vi.clearAllMocks();
    // Default organization mock
    vi.mocked(useOrganizationHook.useOrganization).mockReturnValue({
      organization: undefined,
      isLoading: false,
      isError: false,
    });
  });

  afterEach(() => {
    localStorage.clear();
  });

  it('should return default color when no workspace and no user override', () => {
    const { result } = renderHook(() => useTheme());
    expect(result.current.primaryColor).toBe(defaultColor);
    expect(result.current.isOverridden).toBe(false);
  });

  it('should return workspace theme when available and no user override', () => {
    vi.mocked(useOrganizationHook.useOrganization).mockReturnValue({
 
      organization: { theme_primary_color: workspaceColor } as unknown as never,
      isLoading: false,
      isError: false,
    });

    const { result } = renderHook(() => useTheme());
    expect(result.current.primaryColor).toBe(workspaceColor);
    expect(result.current.isOverridden).toBe(false);
  });

  it('should return user override color when set', () => {
    localStorage.setItem('user_theme_primary_color', userColor);
    
    vi.mocked(useOrganizationHook.useOrganization).mockReturnValue({
 
      organization: { theme_primary_color: workspaceColor } as unknown as never,
      isLoading: false,
      isError: false,
    });

    const { result } = renderHook(() => useTheme());
    expect(result.current.primaryColor).toBe(userColor);
    expect(result.current.isOverridden).toBe(true);
  });

  it('should update theme when setUserTheme is called', () => {
    const { result } = renderHook(() => useTheme());

    act(() => {
      result.current.setUserTheme(userColor);
    });

    expect(result.current.primaryColor).toBe(userColor);
    expect(result.current.isOverridden).toBe(true);
    expect(localStorage.getItem('user_theme_primary_color')).toBe(userColor);
  });

  it('should reset theme when setUserTheme is called with null', () => {
    localStorage.setItem('user_theme_primary_color', userColor);
    
    vi.mocked(useOrganizationHook.useOrganization).mockReturnValue({
 
      organization: { theme_primary_color: workspaceColor } as unknown as never,
      isLoading: false,
      isError: false,
    });

    const { result } = renderHook(() => useTheme());
    
    // Initial state check
    expect(result.current.primaryColor).toBe(userColor);

    act(() => {
      result.current.setUserTheme(null);
    });

    expect(result.current.primaryColor).toBe(workspaceColor);
    expect(result.current.isOverridden).toBe(false);
    expect(localStorage.getItem('user_theme_primary_color')).toBeNull();
  });

  it('should apply CSS variables to document root', () => {
    const setPropertySpy = vi.spyOn(document.documentElement.style, 'setProperty');
    
    renderHook(() => useTheme());

    // Check if setProperty was called for --primary-500 (base color)
    expect(setPropertySpy).toHaveBeenCalledWith('--primary-500', expect.any(String));
    // Check if sidebar primary color is set
    expect(setPropertySpy).toHaveBeenCalledWith('--sidebar-primary', expect.any(String));
  });

  it('should sync across tabs using storage event', () => {
    const { result } = renderHook(() => useTheme());
    
    act(() => {
        // Simulate storage event from another tab
        localStorage.setItem('user_theme_primary_color', userColor);
        window.dispatchEvent(new StorageEvent('storage', {
            key: 'user_theme_primary_color',
            newValue: userColor,
            storageArea: localStorage,
        }));
    });

    expect(result.current.primaryColor).toBe(userColor);
  });
});
