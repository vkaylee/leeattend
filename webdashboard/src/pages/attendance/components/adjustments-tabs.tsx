import { Clock, ShieldCheck } from "lucide-react";
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
      id: "adjustments",
      label: t("attendance_confirm.tabs.adjustments", { defaultValue: "Danh sách điều chỉnh" }).toUpperCase(),
      icon: <Clock size={14} />,
      colorScheme: "primary",
    },
    {
      id: "behalf_assignments",
      label: t("attendance_confirm.tabs.behalf_assignments", { defaultValue: "Behalf" }).toUpperCase(),
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


