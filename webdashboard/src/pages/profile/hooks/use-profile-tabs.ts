import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { useProfile } from "@/hooks/use-profile";

export type TabType = "profile" | "personalization" | "security";

export function useProfileTabs() {
  const { t } = useTranslation();
  const { user, isLoading, updateProfile, isUpdating } = useProfile();
  const toast = useToast();
  const [activeTab, setActiveTab] = useState<TabType>("profile");

  const [formData, setFormData] = useState({
    full_name: "", 
    avatar_url: "", 
    bio: "", 
    preferred_language: "",
  });

  useEffect(() => {
    if (user) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- Derived state synced from props; intentional pattern
      setFormData({
        full_name: user.full_name ?? "", 
        avatar_url: user.avatar_url ?? "",
        bio: user.bio ?? "", 
        preferred_language: user.preferred_language ?? "",
      });
    }
  }, [user]);

  async function handleSubmit(e: React.SyntheticEvent) {
    e.preventDefault();
    try {
      await updateProfile(formData);
      toast.success(t("common.success"));
    } catch (error) {
      console.error(error);
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
