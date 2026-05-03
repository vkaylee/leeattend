import { useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';

import { getCurrentWorkspaceId } from '@/lib/api';

import { useProfile } from './use-profile';
import { useWorkspaces } from './use-workspace';

export function useI18nSync() {
  const { i18n } = useTranslation();
  const { user } = useProfile();
  const { workspaces } = useWorkspaces();
  const currentWsId = getCurrentWorkspaceId();
  const currentWorkspace = workspaces.find(ws => ws.id === currentWsId);

  // Track if this is the initial mount to only sync once on load
  const hasInitialized = useRef(false);

  useEffect(() => {
    // Only sync language on initial load, not on every preference change
    if (hasInitialized.current) return;

    // Respect localStorage if user has manually set a language
    const localStorageLang = localStorage.getItem('i18nextLng');
    if (localStorageLang && ['en', 'vi', 'zh'].includes(localStorageLang)) {
      // User has a localStorage preference, don't override with server value
      if (user) {
        hasInitialized.current = true;
      }
      return;
    }

    // No localStorage preference, use server preference or workspace default
    const langToSet = user?.preferred_language ?? currentWorkspace?.default_language;

    if (langToSet && i18n.language !== langToSet) {
      void i18n.changeLanguage(langToSet);
      localStorage.setItem('i18nextLng', langToSet);
    }

    // Mark as initialized once we have user data
    if (user) {
      hasInitialized.current = true;
    }
  }, [user?.preferred_language, currentWorkspace?.default_language, i18n, user]);
}
