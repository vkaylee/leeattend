import { Palette, ShieldCheck, UserCircle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
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
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
      <div className="space-y-1">
        <Typography className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none" variant="h1">
          {t("common.profile")}
        </Typography>
        <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60" variant="p">
          {t("profile.subtitle")}
        </Typography>
      </div>

      <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
        <button 
          className={cn(
            "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", 
            activeTab === 'profile' 
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" 
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
          )} 
          onClick={() => { setActiveTab("profile"); }}
        >
          <UserCircle size={14} /> {t("common.profile")}
        </button>
        <button 
          className={cn(
            "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", 
            activeTab === 'personalization' 
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" 
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
          )} 
          onClick={() => { setActiveTab("personalization"); }}
        >
          <Palette size={14} /> {t("profile.personalization.title")}
        </button>
        <button 
          className={cn(
            "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", 
            activeTab === 'security' 
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" 
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
          )} 
          onClick={() => { setActiveTab("security"); }}
        >
          <ShieldCheck size={14} /> {t("profile.security.title")}
        </button>
      </div>
    </div>
  );
};
