import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Coins, Calculator, FileText, Blocks, Sparkles } from "lucide-react";
import { cn } from "@/lib/utils";
import SalaryComponentManager from "@/components/payroll/SalaryComponentManager";
import PayrollSettingsSection from "@/components/payroll/PayrollSettingsSection";
import BonusPolicyManager from "@/components/payroll/BonusPolicyManager";
import { getCurrentWorkspaceId } from "@/lib/api";

type TabType = "policy" | "components" | "incentives";

export default function PayrollSettings() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const [activeTab, setActiveTab] = useState<TabType>("policy");

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50"><Coins size={24} /></div>
          <div>
            <h1 className="text-3xl font-bold tracking-tight text-slate-900 leading-tight">{t("common.settings.categories.payroll")}</h1>
            <p className="text-slate-500 mt-1 font-medium">{t("common.settings.categories.payroll_desc")}</p>
          </div>
        </div>
      </div>

      <div className="flex bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50 shadow-inner">
        <button onClick={() => setActiveTab("policy")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'policy' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><FileText size={14} /> {t("common.settings.tabs.payroll_policy")}</button>
        <button onClick={() => setActiveTab("components")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'components' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><Blocks size={14} /> {t("common.settings.tabs.salary_components")}</button>
        <button onClick={() => setActiveTab("incentives")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'incentives' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><Sparkles size={14} /> {t("common.settings.tabs.incentives")}</button>
      </div>

      <div className="animate-in slide-in-from-bottom-4 duration-500">
        {activeTab === "policy" && (
          <div className="grid grid-cols-1 gap-8">
            <PayrollSettingsSection wsId={wsId!} />
          </div>
        )}

        {activeTab === "components" && (
          <div className="bg-white rounded-[2.5rem] border border-slate-100 shadow-sm overflow-hidden p-8">
            <SalaryComponentManager wsId={wsId!} />
          </div>
        )}

        {activeTab === "incentives" && (
          <div className="bg-white rounded-[2.5rem] border border-slate-100 shadow-sm overflow-hidden p-8">
            <BonusPolicyManager wsId={wsId!} />
          </div>
        )}
      </div>
    </div>
  );
}
