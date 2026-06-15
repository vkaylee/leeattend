import { CheckCircle, Clock, ShieldCheck, XCircle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { PageTabs, PageTab } from "@/components/ui/page-tabs";

import type { TabType } from "../hooks/use-attendance-adjustments-page";

interface AdjustmentsTabsProps {
  activeTab: TabType;
  canManageBehalf?: boolean;
  setActiveTab: (tab: TabType) => void;
}

export function AdjustmentsTabs({ activeTab, setActiveTab, canManageBehalf }: AdjustmentsTabsProps) {
  const { t } = useTranslation();

  const tabs: PageTab[] = [
    {
      id: "pending",
      label: t("common.status_pending"),
      icon: <Clock size={14} />,
      colorScheme: "amber",
    },
    {
      id: "approved",
      label: t("common.status_approved"),
      icon: <CheckCircle size={14} />,
      colorScheme: "emerald",
    },
    {
      id: "rejected",
      label: t("common.status_rejected"),
      icon: <XCircle size={14} />,
      colorScheme: "rose",
    },
    {
      id: "behalf_assignments",
      label: t("attendance_confirm.tabs.behalf_assignments", { defaultValue: "Behalf" }),
      icon: <ShieldCheck size={14} />,
      colorScheme: "violet",
      hidden: !canManageBehalf,
    },
  ];

  return (
    <PageTabs
      activeTab={activeTab}
      onTabChange={(tabId) => {
        setActiveTab(tabId as TabType);
      }}
      tabs={tabs}
    />
  );
}


