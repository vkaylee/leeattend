import { useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import type { ProfileSearch } from "@/pages/profile/search-schema";

import { useToast } from "@/components/ui/toast";
import { useDerivedStateFrom } from "@/hooks/use-derived-state-from";
import { useProfile } from "@/hooks/use-profile";
import { logError } from "@/lib/logger";
import { useSearchPatch } from "@/lib/state";
import { Route } from "@/routes/_authenticated/_workspace/profile";

export type TabType = ProfileSearch["tab"];

export function useProfileTabs() {
  const { t } = useTranslation();
  const { user, isLoading, updateProfile, isUpdating } = useProfile();
  const toast = useToast();

  const search = Route.useSearch();
  const patchSearch = useSearchPatch<ProfileSearch>();
  const activeTab = search.tab;
  const setActiveTab = useCallback((tab: TabType) => { patchSearch({ tab }); }, [patchSearch]);

  // Stabilize the external object so local form edits are not overwritten on rerender.
  const externalFormData = useMemo(() => ({
    full_name: user?.full_name ?? "",
    avatar_url: user?.avatar_url ?? "",
    bio: user?.bio ?? "",
    preferred_language: user?.preferred_language ?? "",
  }), [user]);
  const [formData, setFormData] = useDerivedStateFrom(externalFormData);

  async function handleSubmit(e: React.SyntheticEvent) {
    e.preventDefault();
    try {
      await updateProfile(formData);
      toast.success(t("common.success"));
    } catch (error) {
      logError(error, { component: "useProfileTabs" });
    }
  }

  return {
    t,
    user,
    isLoading,
    updateProfile,
    isUpdating,
    activeTab,
    setActiveTab,
    formData,
    setFormData,
    handleSubmit,
  };
}
