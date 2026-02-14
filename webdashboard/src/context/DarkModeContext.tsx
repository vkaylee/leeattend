import React from 'react';
import { useDarkMode } from '@/hooks/use-dark-mode';
import { DarkModeContext } from './dark-mode-context-definition';

export const DarkModeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const darkMode = useDarkMode();
  return (
    <DarkModeContext.Provider value={darkMode}>
      {children}
    </DarkModeContext.Provider>
  );
};
