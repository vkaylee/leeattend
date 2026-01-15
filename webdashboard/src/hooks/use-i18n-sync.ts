import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useWorkspaces } from './use-workspace';
import { useProfile } from './use-profile';
import { getCurrentWorkspaceId } from '@/lib/api';

export function useI18nSync() {
  const { i18n } = useTranslation();
  const { user } = useProfile();
  const { workspaces } = useWorkspaces();
  const currentWsId = getCurrentWorkspaceId();
  const currentWorkspace = workspaces.find(ws => ws.id === currentWsId);

  useEffect(() => {
    const langToSet = user?.preferred_language || currentWorkspace?.default_language;
    
    if (langToSet && i18n.language !== langToSet) {
      i18n.changeLanguage(langToSet);
      localStorage.setItem('i18nextLng', langToSet);
    }
  }, [user?.preferred_language, currentWorkspace?.default_language, i18n]);
}
