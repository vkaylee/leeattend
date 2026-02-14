import { useContext } from 'react';
import { DarkModeContext } from './dark-mode-context-definition';

export const useDarkModeContext = () => {
  const context = useContext(DarkModeContext);
  if (!context) throw new Error('useDarkModeContext must be used within DarkModeProvider');
  return context;
};
