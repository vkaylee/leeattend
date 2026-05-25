import { AlertCircle, CreditCard, User as UserIcon, Users } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

type TabType = "personal" | "banking" | "emergency";

export function Header({
  activeTab,
  setActiveTab,
  t,
}: {
  activeTab: TabType;
  setActiveTab: (tab: TabType) => void;
  t: TFunction;
}) {
  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
      <div className="space-y-1">
        <Typography className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none" variant="h1">
          {t("ess.my_profile")}
        </Typography>
        <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60" variant="p">
          {t("ess.profile.subtitle")}
        </Typography>
      </div>

      <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
        <TabButton
          active={activeTab === "personal"}
          icon={<UserIcon size={14} />}
          label={t("ess.personal_info")}
          onClick={() => {
            setActiveTab("personal");
          }}
        />
        <TabButton
          active={activeTab === "banking"}
          icon={<CreditCard size={14} />}
          label={t("ess.bank_details")}
          onClick={() => {
            setActiveTab("banking");
          }}
        />
        <TabButton
          active={activeTab === "emergency"}
          icon={<Users size={14} />}
          label={t("ess.emergency_contacts")}
          onClick={() => {
            setActiveTab("emergency");
          }}
        />
      </div>
    </div>
  );
}

export function LoadingState() {
  return (
    <div className="p-20 text-center">
      <div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div>
    </div>
  );
}

export function ErrorState({ t }: { t: TFunction }) {
  return (
    <div className="p-8">
      <div className="bg-rose-50 dark:bg-rose-950 border-2 border-rose-100 dark:border-rose-800 rounded-[3rem] p-12 flex items-center gap-8 shadow-2xl shadow-rose-100/50">
        <div className="bg-white dark:bg-slate-900 p-5 rounded-[1.5rem] shadow-sm border border-rose-100 dark:border-rose-800 text-rose-500">
          <AlertCircle size={40} />
        </div>
        <div>
          <Typography className="text-2xl font-bold text-rose-900 uppercase tracking-tighter leading-none mb-2" variant="h3">
            {t("ess.no_employee_linked")}
          </Typography>
          <Typography className="font-bold text-rose-600 dark:text-rose-300 uppercase tracking-tight text-sm opacity-80" variant="p">
            {t("ess.contact_hr")}
          </Typography>
        </div>
      </div>
    </div>
  );
}

export function TabButton({
  active,
  icon,
  label,
  onClick,
}: {
  active: boolean;
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      className={cn(
        "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
        active
          ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
          : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
      )}
      onClick={onClick}
    >
      {icon} {label}
    </button>
  );
}
