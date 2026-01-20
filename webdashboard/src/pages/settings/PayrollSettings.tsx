import { useTranslation } from "react-i18next";
import { Coins, Calculator } from "lucide-react";
import SalaryComponentManager from "@/components/payroll/SalaryComponentManager";
import PayrollSettingsSection from "@/components/payroll/PayrollSettingsSection";
import AssignmentRulesPage from "@/pages/AssignmentRulesPage";
import { getCurrentWorkspaceId } from "@/lib/api";

export default function PayrollSettings() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-bold text-slate-900 tracking-tight">
          {t("common.settings.categories.payroll")}
        </h1>
        <p className="text-slate-500 text-sm mt-1">
          {t("common.settings.categories.payroll_desc")}
        </p>
      </div>

      <div className="grid grid-cols-1 gap-8">
        {/* Payroll Base Config */}
        <PayrollSettingsSection wsId={wsId!} />

        {/* Salary Components */}
        <div className="bg-white rounded-3xl border border-slate-100 shadow-sm overflow-hidden">
          <SalaryComponentManager wsId={wsId!} />
        </div>

        {/* Auto Assignment Rules */}
        <div className="bg-white rounded-3xl border border-slate-100 shadow-sm overflow-hidden">
           <div className="p-1">
              <AssignmentRulesPage />
           </div>
        </div>
      </div>
    </div>
  );
}