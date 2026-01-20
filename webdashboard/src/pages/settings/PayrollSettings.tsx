import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Coins, Calculator, FileText, Blocks } from "lucide-react";
import SalaryComponentManager from "@/components/payroll/SalaryComponentManager";
import PayrollSettingsSection from "@/components/payroll/PayrollSettingsSection";
import { getCurrentWorkspaceId } from "@/lib/api";

type TabType = "policy" | "components";

export default function PayrollSettings() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const [activeTab, setActiveTab] = useState<TabType>("policy");

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-slate-900 tracking-tight">
          {t("common.settings.categories.payroll")}
        </h1>
        <p className="text-slate-500 text-sm mt-1">
          {t("common.settings.categories.payroll_desc")}
        </p>
      </div>

      {/* Tabs Navigation */}
      <div className="flex items-center gap-1 bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50">
        <button
          onClick={() => setActiveTab("policy")}
          className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${
            activeTab === "policy"
              ? "bg-white text-primary-600 shadow-sm border border-slate-100"
              : "text-slate-500 hover:text-slate-700"
          }`}
        >
          <FileText size={14} /> {t("common.settings.tabs.payroll_policy", "Payroll Policy")}
        </button>
        <button
          onClick={() => setActiveTab("components")}
          className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${
            activeTab === "components"
              ? "bg-white text-primary-600 shadow-sm border border-slate-100"
              : "text-slate-500 hover:text-slate-700"
          }`}
        >
          <Blocks size={14} /> {t("common.settings.tabs.salary_components", "Salary Components")}
        </button>
      </div>

      <div className="animate-in fade-in duration-500">
        {activeTab === "policy" && (
          <div className="grid grid-cols-1 gap-8">
            <PayrollSettingsSection wsId={wsId!} />
          </div>
        )}

        {activeTab === "components" && (
          <div className="bg-white rounded-3xl border border-slate-100 shadow-sm overflow-hidden">
            <SalaryComponentManager wsId={wsId!} />
          </div>
        )}
      </div>
    </div>
  );
}