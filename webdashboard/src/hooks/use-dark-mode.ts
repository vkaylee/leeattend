import { useState, useEffect, useCallback, useMemo } from 'react';

export type ThemeMode = 'light' | 'dark' | 'system';
const STORAGE_KEY = 'theme_mode';

/** Safe localStorage wrapper for private browsing / restricted environments */
function readStorage(): ThemeMode {
  try { return (localStorage.getItem(STORAGE_KEY) as ThemeMode | null) ?? 'system'; }
  catch { return 'system'; }
}

function writeStorage(value: ThemeMode) {
  try { localStorage.setItem(STORAGE_KEY, value); }
  catch { /* private browsing — ignore */ }
}

function getSystemPreference(): 'light' | 'dark' {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function resolveTheme(mode: ThemeMode): 'light' | 'dark' {
  return mode === 'system' ? getSystemPreference() : mode;
}

function applyTheme(resolved: 'light' | 'dark') {
  document.documentElement.classList.toggle('dark', resolved === 'dark');
}

export function useDarkMode() {
  const [mode, setModeState] = useState<ThemeMode>(() => {
    return readStorage();
  });

  const resolved = resolveTheme(mode);

  const setMode = useCallback((newMode: ThemeMode) => {
    writeStorage(newMode);
    setModeState(newMode);
    applyTheme(resolveTheme(newMode));
    window.dispatchEvent(new CustomEvent('theme-mode-change', { detail: newMode }));
  }, []);

  // Apply on mount + listen for system preference changes
  useEffect(() => {
    applyTheme(resolved);
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = () => {
      if (mode === 'system') applyTheme(getSystemPreference());
    };
    mq.addEventListener('change', handleChange);
    return () => { mq.removeEventListener('change', handleChange); };
  }, [mode, resolved]);

  // Cross-tab sync
  useEffect(() => {
    const handleStorage = (e: StorageEvent) => {
      if (e.key === STORAGE_KEY) {
        const value = (e.newValue as ThemeMode | null) ?? 'system';
        setModeState(value);
        applyTheme(resolveTheme(value));
      }
    };
    window.addEventListener('storage', handleStorage);
    return () => { window.removeEventListener('storage', handleStorage); };
  }, []);

  return useMemo(() => ({ mode, resolved, setMode, isDark: resolved === 'dark' }), [mode, resolved, setMode]);
}
