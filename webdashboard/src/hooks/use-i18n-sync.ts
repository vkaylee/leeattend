import { useEffect, useRef, type MutableRefObject } from 'react';
import { useTranslation } from 'react-i18next';

import { getCurrentWorkspaceId } from '@/lib/api';

import { useProfile } from './use-profile';
import { useWorkspaces } from './use-workspace';

interface SyncLangOptions {
  currentWorkspaceDefaultLang?: string;
  hasInitialized: MutableRefObject<boolean>;
  i18n: { changeLanguage: (lang: string) => Promise<unknown>; language: string };
  userPreferredLang?: string;
}

function syncLanguage({ hasInitialized, userPreferredLang, currentWorkspaceDefaultLang, i18n }: SyncLangOptions, user: unknown) {
  if (hasInitialized.current) return;
  const localStorageLang = localStorage.getItem('i18nextLng');
  if (localStorageLang && ['en', 'vi', 'zh'].includes(localStorageLang)) {
    if (user) hasInitialized.current = true;
    return;
  }
  const langToSet = userPreferredLang ?? currentWorkspaceDefaultLang;
  if (langToSet && i18n.language !== langToSet) {
    void i18n.changeLanguage(langToSet);
    localStorage.setItem('i18nextLng', langToSet);
  }
  if (user) hasInitialized.current = true;
}

export function useI18nSync() {
  const { i18n } = useTranslation();
  const { user } = useProfile();
  const { workspaces } = useWorkspaces();
  const currentWsId = getCurrentWorkspaceId();
  const currentWorkspace = workspaces.find(ws => ws.id === currentWsId);
  const hasInitialized = useRef(false);

  useEffect(() => {
    syncLanguage({
      hasInitialized,
      userPreferredLang: user?.preferred_language,
      currentWorkspaceDefaultLang: currentWorkspace?.default_language,
      i18n,
    }, user);
  }, [user?.preferred_language, currentWorkspace?.default_language, i18n, user]);
}

