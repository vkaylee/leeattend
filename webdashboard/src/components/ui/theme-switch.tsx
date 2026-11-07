import { Sun, Moon } from 'lucide-react';
import React, { memo, useCallback } from 'react';

import { useDarkModeContext } from '@/context/use-dark-mode-context';
import { cn } from '@/lib/utils';

export interface ThemeSwitchProps {
  className?: string;
  /** Test identifier, defaults to "theme-switch" */
  'data-testid'?: string;
  /** Accessible label override */
  'aria-label'?: string;
  /** Whether to trigger Circular Wipe View Transition if supported (default: true) */
  enableViewTransition?: boolean;
}

/**
 * Triggers modern Circular Wipe animation via View Transitions API
 * Falls back safely to direct state update when unsupported or reduced motion is requested
 */
function triggerCircularWipe(
  event: React.MouseEvent<HTMLButtonElement>,
  updateTheme: () => void,
  isDark: boolean
) {
  const doc = typeof document !== 'undefined' ? document : null;
  const isSupported =
    doc !== null &&
    typeof doc.startViewTransition === 'function' &&
    typeof window !== 'undefined' &&
    !window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  if (!isSupported) {
    updateTheme();
    return;
  }

  const x = event.clientX;
  const y = event.clientY;
  const endRadius = Math.hypot(
    Math.max(x, window.innerWidth - x),
    Math.max(y, window.innerHeight - y)
  );

  try {
    const transition = doc.startViewTransition(() => {
      updateTheme();
    });

    transition.ready
      .then(() => {
        const clipPath = [
          `circle(0px at ${x}px ${y}px)`,
          `circle(${endRadius}px at ${x}px ${y}px)`,
        ];
        doc.documentElement.animate(
          {
            clipPath: isDark ? [...clipPath].reverse() : clipPath,
          },
          {
            duration: 350,
            easing: 'ease-in-out',
            pseudoElement: isDark
              ? '::view-transition-old(root)'
              : '::view-transition-new(root)',
          }
        );
      })
      .catch(() => {
        // Fallback gracefully if animation fails
      });
  } catch {
    updateTheme();
  }
}

/**
 * VitePress-inspired dark/light appearance switch component
 * Based on nginxproxymanager.com (VPSwitchAppearance)
 */
export const ThemeSwitch = memo(function ThemeSwitch({
  className,
  'data-testid': testId = 'theme-switch',
  'aria-label': ariaLabel,
  enableViewTransition = true,
}: ThemeSwitchProps) {
  const { isDark, setMode } = useDarkModeContext();

  const handleToggle = useCallback(
    (event: React.MouseEvent<HTMLButtonElement>) => {
      const nextMode = isDark ? 'light' : 'dark';
      if (enableViewTransition) {
        triggerCircularWipe(event, () => setMode(nextMode), isDark);
      } else {
        setMode(nextMode);
      }
    },
    [enableViewTransition, isDark, setMode]
  );

  const label = ariaLabel ?? (isDark ? 'Switch to light theme' : 'Switch to dark theme');

  return (
    <button
      aria-checked={isDark}
      aria-label={label}
      className={cn(
        'relative inline-flex h-[22px] w-10 shrink-0 cursor-pointer items-center rounded-full border border-slate-300 dark:border-slate-700 bg-slate-200/90 dark:bg-zinc-800 transition-colors duration-200 ease-in-out hover:border-primary-500 dark:hover:border-primary-400 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2',
        className
      )}
      data-testid={testId}
      onClick={handleToggle}
      role="switch"
      title={label}
      type="button"
    >
      <span
        aria-hidden="true"
        className={cn(
          'pointer-events-none absolute top-[1px] left-[1px] h-[18px] w-[18px] rounded-full bg-white dark:bg-zinc-950 shadow-sm transition-transform duration-200 ease-out flex items-center justify-center overflow-hidden',
          isDark ? 'translate-x-[18px]' : 'translate-x-0'
        )}
      >
        <Sun
          className={cn(
            'absolute h-3 w-3 text-amber-500 transition-all duration-200',
            isDark
              ? 'opacity-0 scale-50 -rotate-90 pointer-events-none'
              : 'opacity-100 scale-100 rotate-0'
          )}
          strokeWidth={2.2}
        />
        <Moon
          className={cn(
            'absolute h-3 w-3 text-slate-100 transition-all duration-200',
            isDark
              ? 'opacity-100 scale-100 rotate-0'
              : 'opacity-0 scale-50 rotate-90 pointer-events-none'
          )}
          strokeWidth={2.2}
        />
      </span>
    </button>
  );
});
