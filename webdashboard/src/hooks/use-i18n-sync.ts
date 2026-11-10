import { useEffect, useRef, type RefObject } from 'react';
import { useTranslation } from 'react-i18next';


import { useProfile } from './use-profile';
import { useOrganization } from "@/hooks/use-organization";
interface SyncLangOptions {
  currentOrganizationDefaultLang?: string;
  hasInitialized: RefObject<boolean>;
  i18n: { changeLanguage: (lang: string) => Promise<unknown>; language: string };
  userPreferredLang?: string | null;
}

function syncLanguage({ hasInitialized, userPreferredLang, currentOrganizationDefaultLang, i18n }: SyncLangOptions, user: unknown) {
  if (hasInitialized.current) return;
  const localStorageLang = localStorage.getItem('i18nextLng');
  if (localStorageLang && ['en', 'vi', 'zh'].includes(localStorageLang)) {
    if (user) hasInitialized.current = true;
    return;
  }
  const langToSet = userPreferredLang ?? currentOrganizationDefaultLang ?? undefined;
  if (langToSet && i18n.language !== langToSet) {
    void i18n.changeLanguage(langToSet);
    localStorage.setItem('i18nextLng', langToSet);
  }
  if (user) hasInitialized.current = true;
}

export function useI18nSync() {
  const { i18n } = useTranslation();
  const { user } = useProfile();
  const { organization } = useOrganization();
  const hasInitialized = useRef(false);

  useEffect(() => {
    syncLanguage({
      hasInitialized,
      userPreferredLang: user?.preferred_language,
      currentOrganizationDefaultLang: organization?.default_language,
      i18n,
    }, user);
  }, [user?.preferred_language, organization?.default_language, i18n, user]);
}

