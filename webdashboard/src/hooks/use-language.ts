import { useCallback, useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useProfile } from './use-profile';

export type SupportedLanguage = 'en' | 'vi' | 'zh';

export interface LanguageOption {
  code: SupportedLanguage;
  name: string;
  nativeName: string;
  flag: string;
}

// Supported languages with metadata
export const SUPPORTED_LANGUAGES: LanguageOption[] = [
  { code: 'vi', name: 'Vietnamese', nativeName: 'Tiếng Việt', flag: '🇻🇳' },
  { code: 'en', name: 'English', nativeName: 'English', flag: '🇺🇸' },
  { code: 'zh', name: 'Chinese', nativeName: '中文', flag: '🇨🇳' },
];

/**
 * Hook for managing language switching with API sync
 * - Changes i18n language immediately for responsive UI
 * - Syncs preference to server in background (silent failure)
 * - Returns current language and available options
 */
export function useLanguage() {
  const { i18n } = useTranslation();
  const { user, updateProfile } = useProfile();

  // Local state to trigger re-renders when language changes
  const [currentLanguage, setCurrentLanguage] = useState<SupportedLanguage>(
    (i18n.language || 'vi') as SupportedLanguage
  );

  // Sync state with i18n language changes
  useEffect(() => {
    const handleLanguageChanged = (lng: string) => {
      setCurrentLanguage(lng as SupportedLanguage);
    };

    i18n.on('languageChanged', handleLanguageChanged);
    return () => {
      i18n.off('languageChanged', handleLanguageChanged);
    };
  }, [i18n]);

  const currentOption = SUPPORTED_LANGUAGES.find(l => l.code === currentLanguage) || SUPPORTED_LANGUAGES[0];

  const changeLanguage = useCallback(async (langCode: SupportedLanguage) => {
    await i18n.changeLanguage(langCode);
    localStorage.setItem('i18nextLng', langCode);

    // Sync to server in background (silent failure)
    try {
      await updateProfile({ preferred_language: langCode });
    } catch {
      console.warn('Failed to sync language preference to server');
    }
  }, [i18n, updateProfile]);

  return {
    currentLanguage,
    currentOption,
    languages: SUPPORTED_LANGUAGES,
    changeLanguage,
    userPreference: user?.preferred_language as SupportedLanguage | undefined,
  };
}
