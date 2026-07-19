import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { useDerivedStateFrom } from "@/hooks/use-derived-state-from";
import { useProfile } from "@/hooks/use-profile";
import { logError } from "@/lib/logger";

export type TabType = "profile" | "personalization" | "security";

export function useProfileTabs() {
  const { t } = useTranslation();
  const { user, isLoading, updateProfile, isUpdating } = useProfile();
  const toast = useToast();
  const [activeTab, setActiveTab] = useState<TabType>("profile");

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
