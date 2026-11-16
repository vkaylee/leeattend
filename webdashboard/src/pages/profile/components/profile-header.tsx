import { Palette, ShieldCheck, UserCircle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { PageHeader } from "@/components/ui/page-header";
import { cn } from "@/lib/utils";

import { TabType } from "../hooks/use-profile-tabs";

interface ProfileHeaderProps {
  activeTab: TabType;
  setActiveTab: (tab: TabType) => void;
}

export const ProfileHeader: React.FC<ProfileHeaderProps> = ({
  activeTab,
  setActiveTab,
}) => {
  const { t } = useTranslation();

  return (
    <PageHeader
      actions={
        <div className="flex w-full max-w-full overflow-x-auto scrollbar-hide bg-slate-100 dark:bg-slate-700/50 p-1 rounded-xl border border-slate-200 dark:border-slate-700/50 shadow-inner sm:w-fit">
          <Button className={cn(
              "flex shrink-0 items-center gap-2 px-3 py-2 rounded-lg text-xs font-bold uppercase tracking-widest transition-all sm:px-5",
              activeTab === 'profile' 
                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" 
                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
            )} 
            onClick={() => { setActiveTab("profile"); }} 
            variant="ghost"
          >
            <UserCircle size={14} /> {t("common.profile")}
          </Button>
          <Button className={cn(
              "flex shrink-0 items-center gap-2 px-3 py-2 rounded-lg text-xs font-bold uppercase tracking-widest transition-all sm:px-5",
              activeTab === 'personalization' 
                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" 
                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
            )} 
            onClick={() => { setActiveTab("personalization"); }} 
            variant="ghost"
          >
            <Palette size={14} /> {t("profile.personalization.title")}
          </Button>
          <Button className={cn(
              "flex shrink-0 items-center gap-2 px-3 py-2 rounded-lg text-xs font-bold uppercase tracking-widest transition-all sm:px-5",
              activeTab === 'security' 
                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" 
                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
            )} 
            onClick={() => { setActiveTab("security"); }} 
            variant="ghost"
          >
            <ShieldCheck size={14} /> {t("profile.security.title")}
          </Button>
        </div>
      }
      data-testid="profile-title"
      subtitle={t("profile.subtitle")}
      title={t("common.profile")}
    />
  );
};
