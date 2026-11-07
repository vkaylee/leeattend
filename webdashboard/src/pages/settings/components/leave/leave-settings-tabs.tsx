import { CalendarDays, ShieldCheck, UserPlus } from "lucide-react";
import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";

import {
  SettingsSegmentedControl,
  type SegmentedOption,
} from "@/components/ui/settings/settings-segmented-control";

import type { TabType } from "../../hooks/use-leave-settings";

interface LeaveSettingsTabsProps {
  activeTab: TabType;
  onTabChange: (tab: TabType) => void;
}

export function LeaveSettingsTabs({
  activeTab,
  onTabChange,
}: LeaveSettingsTabsProps) {
  const { t } = useTranslation();

  const options: SegmentedOption<TabType>[] = useMemo(
    () => [
      {
        icon: <CalendarDays className="w-3.5 h-3.5" />,
        label: t("settings.leave.tabs.types"),
        testId: "leave-tab-types",
        value: "types",
      },
      {
        icon: <ShieldCheck className="w-3.5 h-3.5" />,
        label: t("settings.leave.tabs.policies"),
        testId: "leave-tab-policies",
        value: "policies",
      },
      {
        icon: <UserPlus className="w-3.5 h-3.5" />,
        label: t("settings.leave.tabs.assignments"),
        testId: "leave-tab-assignments",
        value: "assignments",
      },
    ],
    [t]
  );

  return (
    <div className="mb-6 overflow-x-auto max-w-full pb-1">
      <SettingsSegmentedControl
        onChange={onTabChange}
        options={options}
        testId="leave-settings-tabs"
        value={activeTab}
      />
    </div>
  );
}

export default LeaveSettingsTabs;
