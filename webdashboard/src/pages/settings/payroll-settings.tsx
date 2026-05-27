import { Coins, Calculator, FileText, Blocks, Sparkles } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import BonusPolicyManager from "@/components/payroll/bonus-policy-manager";
import OvertimePolicyManager from "@/components/payroll/overtime-policy-manager";
import PayrollRulesManager from "@/components/payroll/payroll-rules-manager";
import PayrollSettingsSection from "@/components/payroll/payroll-settings-section";
import SalaryBaseGroupManager from "@/components/payroll/salary-base-group-manager";
import SalaryComponentManager from "@/components/payroll/salary-component-manager";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { getCurrentWorkspaceId } from "@/lib/api";
import { cn } from "@/lib/utils";

type TabType = "policy" | "components" | "bases" | "incentives" | "overtime" | "statutory_rules";

function PayrollTabContent({ activeTab, wsId }: { activeTab: TabType; wsId: string }) {
  switch (activeTab) {
    case "policy":
      return (
        <div className="grid grid-cols-1 gap-8">
          <PayrollSettingsSection wsId={wsId} />
        </div>
      );
    case "components":
      return (
        <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden p-8">
          <SalaryComponentManager wsId={wsId} />
        </div>
      );
    case "bases":
      return (
        <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden p-8">
          <SalaryBaseGroupManager wsId={wsId} />
        </div>
      );
    case "overtime":
      return (
        <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden p-8">
          <OvertimePolicyManager wsId={wsId} />
        </div>
      );
    case "incentives":
      return (
        <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden p-8">
          <BonusPolicyManager wsId={wsId} />
        </div>
      );
    case "statutory_rules":
      return (
        <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden p-8">
          <PayrollRulesManager wsId={wsId} />
        </div>
      );
    default:
      return null;
  }
}

export default function PayrollSettings() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const [activeTab, setActiveTab] = useState<TabType>("policy");

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
    <div className="space-y-10 animate-in fade-in duration-500">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50"><Coins size={24} /></div>
          <div>
            <Typography className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200 leading-tight" variant="h1">{t("common.settings.categories.payroll")}</Typography>
            <Typography className="text-slate-500 dark:text-slate-400 mt-1 font-medium" variant="p">{t("common.settings.categories.payroll_desc")}</Typography>
          </div>
        </div>
      </div>

      <div className="flex flex-col lg:flex-row gap-8 lg:gap-12">
        {/* Navigation Sidebar */}
        <div className="w-full lg:w-64 shrink-0">
          <nav className="flex lg:flex-col gap-1.5 overflow-x-auto lg:overflow-visible pb-4 lg:pb-0 hide-scrollbar -mx-4 px-4 lg:mx-0 lg:px-0">
            {navItems.map((item) => (
              <Button className={cn(
                  "flex items-center gap-3 px-4 py-2.5 rounded-xl text-sm font-semibold transition-all text-left whitespace-nowrap lg:whitespace-normal shrink-0 lg:shrink",
                  activeTab === item.id
                    ? "bg-slate-100 dark:bg-slate-800/80 text-primary-600 dark:text-primary-400"
                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-50 dark:hover:bg-slate-800/50"
                )}
                data-testid={item.testId}
                key={item.id}
                onClick={() => { setActiveTab(item.id); }}
                variant="ghost"
              >
                <item.icon className={cn("shrink-0", activeTab === item.id ? "text-primary-600 dark:text-primary-400" : "text-slate-400")} size={18} />
                <span>{item.label}</span>
              </Button>
            ))}
          </nav>
        </div>

        {/* Content Area */}
        <div className="flex-1 min-w-0">
          <div className="animate-in slide-in-from-right-4 duration-500">
            <PayrollTabContent activeTab={activeTab} wsId={wsId} />
          </div>
        </div>
      </div>
    </div>
  );
}
