import { createContext } from 'react';

import type { ThemeMode } from '@/hooks/use-dark-mode';

interface DarkModeContextType {
  isDark: boolean;
  mode: ThemeMode;
  resolved: 'light' | 'dark';
  setMode: (mode: ThemeMode) => void;
}

export const DarkModeContext = createContext<DarkModeContextType | undefined>(undefined);
