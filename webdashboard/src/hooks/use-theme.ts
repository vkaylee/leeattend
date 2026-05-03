import { useEffect, useState } from 'react';

import { useWorkspace } from './use-workspace';

/**
 * Helper to adjust hex color brightness
 */
function adjustBrightness(hex: string, percent: number) {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);

  const calculate = (c: number) => {
    const value = Math.round(c + (percent / 100) * (percent > 0 ? 255 - c : c));
    return Math.min(255, Math.max(0, value)).toString(16).padStart(2, '0');
  };

  return `#${calculate(r)}${calculate(g)}${calculate(b)}`;
}

export function useTheme() {
  const { workspace } = useWorkspace();
  const [userColor, setUserColor] = useState<string | null>(localStorage.getItem('user_theme_primary_color'));
  const [previewColor, setPreviewColor] = useState<string | null>(null);

  useEffect(() => {
    // Listen for storage changes (to sync across tabs or from Profile page)
    const handleStorage = () => {
      setUserColor(localStorage.getItem('user_theme_primary_color'));
    };

    const handlePreview = (e: Event) => {
      const customEvent = e as CustomEvent<string | null>;
      setPreviewColor(customEvent.detail);
    };

    window.addEventListener('storage', handleStorage);
    // Custom event for same-tab updates
    window.addEventListener('theme-change', handleStorage);
    // Live preview event
    window.addEventListener('theme-preview', handlePreview);
    
    return () => {
      window.removeEventListener('storage', handleStorage);
      window.removeEventListener('theme-change', handleStorage);
      window.removeEventListener('theme-preview', handlePreview);
    };
  }, []);

  const primaryColor = previewColor ?? (userColor ?? workspace?.theme_primary_color) ?? '#10b981';

  useEffect(() => {
    const root = document.documentElement;
    
    const shades = {
      50: adjustBrightness(primaryColor, 95),
      100: adjustBrightness(primaryColor, 85),
      200: adjustBrightness(primaryColor, 70),
      300: adjustBrightness(primaryColor, 50),
      400: adjustBrightness(primaryColor, 30),
      500: primaryColor,
      600: adjustBrightness(primaryColor, -10),
      700: adjustBrightness(primaryColor, -25),
      800: adjustBrightness(primaryColor, -40),
      900: adjustBrightness(primaryColor, -55),
      950: adjustBrightness(primaryColor, -70),
    };

    Object.entries(shades).forEach(([shade, color]) => {
      root.style.setProperty(`--primary-${shade ?? ""}`, color);
    });

    root.style.setProperty('--sidebar-primary', primaryColor);
  }, [primaryColor]);

  return {
    primaryColor,
    isOverridden: !!userColor,
    setUserTheme: (color: string | null) => {
      if (color) {
        localStorage.setItem('user_theme_primary_color', color);
      } else {
        localStorage.removeItem('user_theme_primary_color');
      }
      setUserColor(color);
      window.dispatchEvent(new Event('theme-change'));
    },
    setPreviewTheme: (color: string | null) => {
      window.dispatchEvent(new CustomEvent('theme-preview', { detail: color }));
    }
  };
}