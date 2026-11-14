import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';

import { describe, it, expect, vi, beforeEach } from 'vitest';

import { ThemeSwitch } from '../theme-switch';

const mockSetMode = vi.fn();
let mockIsDark = false;

vi.mock('@/context/use-dark-mode-context', () => ({
  useDarkModeContext: () => ({
    isDark: mockIsDark,
    setMode: mockSetMode,
  }),
}));

describe('ThemeSwitch Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockIsDark = false;
  });

  it('renders with role="switch" and correct ARIA attributes in light mode', () => {
    mockIsDark = false;
    render(<ThemeSwitch />);

    const switchEl = screen.getByRole('switch');
    expect(switchEl).toBeInTheDocument();
    expect(switchEl).toHaveAttribute('aria-checked', 'false');
    expect(switchEl).toHaveAttribute('title', 'Switch to dark theme');
    expect(switchEl).toHaveAttribute('aria-label', 'Switch to dark theme');
  });

  it('renders with correct ARIA attributes in dark mode', () => {
    mockIsDark = true;
    render(<ThemeSwitch />);

    const switchEl = screen.getByRole('switch');
    expect(switchEl).toBeInTheDocument();
    expect(switchEl).toHaveAttribute('aria-checked', 'true');
    expect(switchEl).toHaveAttribute('title', 'Switch to light theme');
    expect(switchEl).toHaveAttribute('aria-label', 'Switch to light theme');
  });

  it('calls setMode("dark") when clicked in light mode', () => {
    mockIsDark = false;
    render(<ThemeSwitch />);

    const switchEl = screen.getByRole('switch');
    fireEvent.click(switchEl);
    expect(mockSetMode).toHaveBeenCalledWith('dark');
  });

  it('calls setMode("light") when clicked in dark mode', () => {
    mockIsDark = true;
    render(<ThemeSwitch />);

    const switchEl = screen.getByRole('switch');
    fireEvent.click(switchEl);
    expect(mockSetMode).toHaveBeenCalledWith('light');
  });

  it('supports custom data-testid, aria-label, and className', () => {
    render(
      <ThemeSwitch
        aria-label="Toggle display mode"
        className="custom-class"
        data-testid="custom-theme-switch"
      />
    );

    const switchEl = screen.getByTestId('custom-theme-switch');
    expect(switchEl).toBeInTheDocument();
    expect(switchEl).toHaveAttribute('aria-label', 'Toggle display mode');
    expect(switchEl.className).toContain('custom-class');
  });

  it('handles View Transitions API when supported by document', () => {
    const mockStartViewTransition = vi.fn((cb: () => void) => {
      cb();
      return {
        ready: Promise.resolve(),
      };
    });

    Object.defineProperty(document, 'startViewTransition', {
      writable: true,
      configurable: true,
      value: mockStartViewTransition,
    });

    render(<ThemeSwitch enableViewTransition />);
    const switchEl = screen.getByRole('switch');
    fireEvent.click(switchEl);

    expect(mockStartViewTransition).toHaveBeenCalled();
    expect(mockSetMode).toHaveBeenCalledWith('dark');

    // Cleanup
    Reflect.deleteProperty(document, 'startViewTransition');
  });
});
