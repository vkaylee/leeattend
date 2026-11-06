 
import { Coins, Calculator, FileText, Blocks, Sparkles } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import BonusPolicyManager from "@/components/payroll/bonus-policy-manager";
import OvertimePolicyManager from "@/components/payroll/overtime-policy-manager";
import PayrollRulesManager from "@/components/payroll/payroll-rules-manager";
import PayrollSettingsSection from "@/components/payroll/payroll-settings-section";
import SalaryBaseGroupManager from "@/components/payroll/salary-base-group-manager";
import SalaryComponentManager from "@/components/payroll/salary-component-manager";
import { PageTabs } from "@/components/ui/page-tabs";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Typography } from "@/components/ui/typography";
import { getStandaloneOrganizationId } from "@/lib/api";

type TabType = "policy" | "components" | "bases" | "incentives" | "overtime" | "statutory_rules";

function PayrollTabContent({ activeTab, wsId }: { activeTab: TabType; wsId: string }) {
  switch (activeTab) {
    case "policy":
      return <PayrollSettingsSection wsId={wsId} />;
    case "components":
      return <SalaryComponentManager wsId={wsId} />;
    case "bases":
      return <SalaryBaseGroupManager wsId={wsId} />;
    case "overtime":
      return <OvertimePolicyManager wsId={wsId} />;
    case "incentives":
      return <BonusPolicyManager wsId={wsId} />;
    case "statutory_rules":
      return <PayrollRulesManager wsId={wsId} />;
    default:
      return null;
  }
}

export default function PayrollSettings() {
  const { t } = useTranslation();
  const wsId = getStandaloneOrganizationId();

  type ParentGroupType = "core" | "formulas" | "benefits";

  const parentGroups = [
    { id: "core" as ParentGroupType, label: t("common.settings.tabs.rules_policies", "Rules & Policies"), icon: FileText },
    { id: "formulas" as ParentGroupType, label: t("common.settings.tabs.salary_structures", "Salary Structures"), icon: Blocks },
    { id: "benefits" as ParentGroupType, label: t("common.settings.tabs.incentives_benefits", "Incentives & Benefits"), icon: Sparkles },
  ];

  const subTabMapping: Record<ParentGroupType, TabType[]> = {
    core: ["policy", "statutory_rules"],
    formulas: ["components", "bases"],
    benefits: ["incentives", "overtime"],
  };

  const [activeTab, setActiveTab] = useState<TabType>("policy");

  // Dynamically derive active parent group from the active sub-tab
  const activeParentGroup = (Object.keys(subTabMapping) as ParentGroupType[]).find(
    (key) => subTabMapping[key].includes(activeTab)
  ) ?? "core";

  if (!wsId) {
    return <div className="p-12 text-center text-slate-500 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;
  }

  const navItems = [
    { id: "policy" as TabType, icon: FileText, label: t("common.settings.tabs.payroll_policy"), testId: "payroll-tab-policy" },
    { id: "components" as TabType, icon: Blocks, label: t("common.settings.tabs.salary_components"), testId: "payroll-tab-components" },
    { id: "bases" as TabType, icon: Coins, label: "Salary Bases", testId: "payroll-tab-bases" },
    { id: "incentives" as TabType, icon: Sparkles, label: t("common.settings.tabs.incentives"), testId: "payroll-tab-incentives" },
    { id: "overtime" as TabType, icon: Calculator, label: t("common.settings.tabs.overtime_policy"), testId: "payroll-tab-overtime" },
    { id: "statutory_rules" as TabType, icon: FileText, label: t("common.settings.tabs.statutory_rules"), testId: "payroll-tab-rules" },
  ];

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-2 border-b border-slate-200/60 dark:border-slate-800/60">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950/70 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-2xs border border-primary-100/50 dark:border-primary-900/50">
            <Coins size={20} />
          </div>
          <div>
            <Typography className="text-xl font-bold tracking-tight text-slate-800 dark:text-slate-100 normal-case" variant="h3">
              {t("common.settings.categories.payroll")}
            </Typography>
            <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium normal-case tracking-normal" variant="small">
              {t("common.settings.categories.payroll_desc")}
            </Typography>
          </div>
        </div>
      </div>

      <div className="pb-2">
        {/* Parent Group Category Tabs */}
        <PageTabs
          activeTab={activeParentGroup}
          onTabChange={(groupId) => {
            const nextGroup = groupId as ParentGroupType;
            const defaultSubTab = subTabMapping[nextGroup][0];
            setActiveTab(defaultSubTab);
          }}
          tabs={parentGroups.map((group) => ({
            id: group.id,
            label: group.label,
            icon: <group.icon className="w-3.5 h-3.5" />,
            colorScheme: "primary",
            testId: `payroll-group-${group.id}`,
          }))}
        />
      </div>

      {/* Card Container holding Level-2 tabs and Panel Content */}
      <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 shadow-2xs overflow-hidden">
        {/* Level-2 Sub-tabs header */}
        <div className="border-b border-slate-100 dark:border-slate-800/60 px-8 pt-6 bg-slate-50/50 dark:bg-slate-900/50">
          <Tabs onValueChange={(tabId) => { setActiveTab(tabId as TabType); }} value={activeTab}>
            <TabsList className="border-none pb-0">
              {navItems
                .filter((item) => subTabMapping[activeParentGroup].includes(item.id))
                .map((item) => (
                  <TabsTrigger
                    className="flex items-center gap-2 px-4 pb-3 pt-2 text-xs font-semibold transition-all border-b-2"
                    data-testid={item.testId}
                    key={item.id}
                    value={item.id}
                  >
                    <item.icon className="w-3.5 h-3.5" />
                    <span>{item.label}</span>
                  </TabsTrigger>
                ))}
            </TabsList>
          </Tabs>
        </div>

        {/* Panel content area */}
        <div className="p-8 animate-in slide-in-from-right-4 duration-500">
          <PayrollTabContent activeTab={activeTab} wsId={wsId} />
        </div>
      </div>
    </div>
  );
}
