import { createContext } from 'react';

import type { ThemeMode } from '@/hooks/use-dark-mode';

interface DarkModeContextType {
  mode: ThemeMode;
  resolved: 'light' | 'dark';
  isDark: boolean;
  setMode: (mode: ThemeMode) => void;
}

export const DarkModeContext = createContext<DarkModeContextType | undefined>(undefined);
