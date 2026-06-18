import React from "react";

import { PersonalizationTab } from "./profile/components/personalization-tab";
import { ProfileHeader } from "./profile/components/profile-header";
import { ProfileTab } from "./profile/components/profile-tab";
import { SecurityTab } from "./profile/components/security-tab";
import { useProfileTabs } from "./profile/hooks/use-profile-tabs";

export default function ProfilePage() {
  const {
    user,
    isLoading,
    isUpdating,
    activeTab,
    setActiveTab,
    formData,
    setFormData,
    handleSubmit,
  } = useProfileTabs();

  if (isLoading) {
    return (
      <div className="p-20 text-center">
        <div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto" />
      </div>
    );
  }

  return (
    <div className="space-y-6 animate-in fade-in duration-700 pb-20">
      <ProfileHeader 
        activeTab={activeTab} 
        setActiveTab={setActiveTab} 
      />

      <div className="animate-in slide-in-from-bottom-8 duration-700">
        {activeTab === "profile" && (
          <ProfileTab 
            formData={formData}
            handleSubmit={handleSubmit}
            isUpdating={isUpdating}
            setFormData={setFormData}
            user={user}
          />
        )}

        {activeTab === "personalization" && (
          <PersonalizationTab />
        )}

        {activeTab === "security" && (
          <SecurityTab />
        )}
      </div>
    </div>
  );
}
