import { useQueryClient } from '@tanstack/react-query';
import { useCallback, useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';

import { updateMe, type User } from "@/lib/api";
import { logError } from "@/lib/logger";

import { profileKeys, useProfile } from './use-profile';

export type SupportedLanguage = 'en' | 'vi' | 'zh';

export interface LanguageOption {
  code: SupportedLanguage;
  flag: string;
  name: string;
  nativeName: string;
}

// Supported languages with metadata
export const SUPPORTED_LANGUAGES: LanguageOption[] = [
  { code: 'vi', name: 'Vietnamese', nativeName: 'Tiếng Việt', flag: '🇻🇳' },
  { code: 'en', name: 'English', nativeName: 'English', flag: '🇺🇸' },
  { code: 'zh', name: 'Chinese', nativeName: '中文', flag: '🇨🇳' },
];

// Helper to normalize language codes (e.g. 'en-US' -> 'en')
function normalizeLanguage(lang: string | undefined): SupportedLanguage {
  if (!lang) return 'vi';

  // 1. Check for exact match
  const exactMatch = SUPPORTED_LANGUAGES.find(l => l.code === lang);
  if (exactMatch) return exactMatch.code;

  // 2. Check for prefix match (e.g. 'en-US' matches 'en')
  const prefixMatch = SUPPORTED_LANGUAGES.find(l => lang.startsWith(l.code));
  if (prefixMatch) return prefixMatch.code;

  // 3. Fallback
  return 'vi';
}

/**
 * Hook for managing language switching with API sync
 * - Changes i18n language immediately for responsive UI
 * - Syncs preference to server in background (silent failure)
 * - Returns current language and available options
 */
export function useLanguage() {
  const queryClient = useQueryClient();
  const { i18n } = useTranslation();
  const { user } = useProfile();

  // Local state to trigger re-renders when language changes
  const [currentLanguage, setCurrentLanguage] = useState<SupportedLanguage>(
    normalizeLanguage(i18n.language)
  );

  // Sync state with i18n language changes
  useEffect(() => {
    const handleLanguageChanged = (lng: string) => {
      setCurrentLanguage(normalizeLanguage(lng));
    };

    i18n.on('languageChanged', handleLanguageChanged);
    return () => {
      i18n.off('languageChanged', handleLanguageChanged);
    };
  }, [i18n]);

  const currentOption = SUPPORTED_LANGUAGES.find(l => l.code === currentLanguage) ?? SUPPORTED_LANGUAGES[0];

  const changeLanguage = useCallback(async (langCode: SupportedLanguage) => {
    await i18n.changeLanguage(langCode);
    localStorage.setItem('i18nextLng', langCode);

    // Sync to server in background without invalidating the full profile query.
    try {
      const updatedUser = await updateMe({ preferred_language: langCode });
      queryClient.setQueryData<User | undefined>(profileKeys.me, currentUser =>
        currentUser
          ? {
              ...currentUser,
              preferred_language: updatedUser.preferred_language ?? langCode,
            }
          : currentUser,
      );
    } catch (err: unknown) {
      logError(err, { component: "useLanguage" });
    }
  }, [i18n, queryClient]);

  return {
    currentLanguage,
    currentOption,
    languages: SUPPORTED_LANGUAGES,
    changeLanguage,
    userPreference: user?.preferred_language as SupportedLanguage | undefined,
  };
}
