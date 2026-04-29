import { useState } from "react";

import { Coins, Calculator, FileText, Blocks, Sparkles } from "lucide-react";
import { useTranslation } from "react-i18next";


import BonusPolicyManager from "@/components/payroll/BonusPolicyManager";
import OvertimePolicyManager from "@/components/payroll/OvertimePolicyManager";
import PayrollRulesManager from "@/components/payroll/PayrollRulesManager";
import PayrollSettingsSection from "@/components/payroll/PayrollSettingsSection";
import SalaryBaseGroupManager from "@/components/payroll/SalaryBaseGroupManager";
import SalaryComponentManager from "@/components/payroll/SalaryComponentManager";

import { getCurrentWorkspaceId } from "@/lib/api";
import { cn } from "@/lib/utils";

type TabType = "policy" | "components" | "bases" | "incentives" | "overtime" | "statutory_rules";

export default function PayrollSettings() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const [activeTab, setActiveTab] = useState<TabType>("policy");

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50"><Coins size={24} /></div>
          <div>
            <h1 className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200 leading-tight">{t("common.settings.categories.payroll")}</h1>
            <p className="text-slate-500 dark:text-slate-400 mt-1 font-medium">{t("common.settings.categories.payroll_desc")}</p>
          </div>
        </div>
      </div>

      <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
        <button onClick={() => setActiveTab("policy")} data-testid="payroll-tab-policy" className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'policy' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><FileText size={14} /> {t("common.settings.tabs.payroll_policy")}</button>
        <button onClick={() => setActiveTab("components")} data-testid="payroll-tab-components" className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'components' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><Blocks size={14} /> {t("common.settings.tabs.salary_components")}</button>
        <button onClick={() => setActiveTab("bases")} data-testid="payroll-tab-bases" className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'bases' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><Coins size={14} /> Salary Bases</button>
        <button onClick={() => setActiveTab("incentives")} data-testid="payroll-tab-incentives" className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'incentives' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><Sparkles size={14} /> {t("common.settings.tabs.incentives")}</button>
        <button onClick={() => setActiveTab("overtime")} data-testid="payroll-tab-overtime" className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === "overtime" ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><Calculator size={14} /> {t("common.settings.tabs.overtime_policy")}</button>
        <button onClick={() => setActiveTab("statutory_rules")} data-testid="payroll-tab-rules" className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === "statutory_rules" ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><FileText size={14} /> {t("common.settings.tabs.statutory_rules")}</button>
      </div>

      <div className="animate-in slide-in-from-bottom-4 duration-500">
        {activeTab === "policy" && (
          <div className="grid grid-cols-1 gap-8">
            <PayrollSettingsSection wsId={wsId!} />
          </div>
        )}

        {activeTab === "components" && (
          <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden p-8">
            <SalaryComponentManager wsId={wsId!} />
          </div>
        )}

        {activeTab === "bases" && (
          <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden p-8">
            <SalaryBaseGroupManager wsId={wsId!} />
          </div>
        )}

        {activeTab === "overtime" && (
          <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden p-8">
            <OvertimePolicyManager wsId={wsId!} />
          </div>
        )}

        {activeTab === "incentives" && (
          <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden p-8">
            <BonusPolicyManager wsId={wsId!} />
          </div>
        )}

        {activeTab === "statutory_rules" && (
          <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden p-8">
            <PayrollRulesManager wsId={wsId!} />
          </div>
        )}
      </div>
    </div>
  );
}
