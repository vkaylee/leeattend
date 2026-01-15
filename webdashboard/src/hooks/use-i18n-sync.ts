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
    if (user?.preferred_language) {
      if (i18n.language !== user.preferred_language) {
        i18n.changeLanguage(user.preferred_language);
      }
    } else if (currentWorkspace?.default_language) {
      if (i18n.language !== currentWorkspace.default_language) {
        i18n.changeLanguage(currentWorkspace.default_language);
      }
    }
  }, [user?.preferred_language, currentWorkspace?.default_language, i18n]);
}
