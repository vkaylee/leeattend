import { Building2, Briefcase } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { SettingsHeader } from "@/components/ui/settings/settings-header";
import { SettingsSegmentedControl } from "@/components/ui/settings/settings-segmented-control";

import EmployersPage from "./employers-page";

type TabType = "employers";

export default function OrganizationSettings() {
  const { t } = useTranslation();
  
  const [activeTab, setActiveTab] = useState<TabType>("employers");

  return (
    <div className="space-y-6 animate-in fade-in duration-300">
      <SettingsHeader
        description={t("common.settings.categories.organization_desc", { defaultValue: "Quản lý pháp nhân và lộ trình công tác" })}
        icon={<Building2 size={20} />}
        title={t("common.settings.categories.organization", { defaultValue: "Pháp nhân & Mốc công tác" })}
      />

      <div className="overflow-x-auto max-w-full pb-1">
        <SettingsSegmentedControl
          onChange={(val) => { setActiveTab(val); }}
          options={[
            {
              icon: <Briefcase size={14} />,
              label: "Pháp nhân",
              testId: "tab-employers",
              value: "employers",
            },
          ]}
          testId="organization-tabs"
          value={activeTab}
        />
      </div>

      <div className="animate-in fade-in duration-300">


        <div className="bg-white dark:bg-slate-900/90 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 overflow-hidden shadow-2xs p-6">
          <EmployersPage />
        </div>
      </div>
    </div>
  );
}
