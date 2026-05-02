import { Building2, Layers } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import EmploymentPhaseManager from "@/components/people/EmploymentPhaseManager";
import { useWorkspaceRoles } from "@/hooks/use-workspace";
import { cn } from "@/lib/utils";
import DepartmentsPage from "@/pages/DepartmentsPage";


type TabType = "departments" | "employment_phases";

export default function OrganizationSettings() {

  const { t } = useTranslation();
  const { roles } = useWorkspaceRoles();
  const [activeTab, setActiveTab] = useState<TabType>("departments");
  return (

    <div className="space-y-8 animate-in fade-in duration-500">

      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">

        <div className="flex items-center gap-4">

          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50"><Building2 size={24} /></div>

          <div>

            <h1 className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200 leading-tight" data-testid="organization-settings-title">{t("common.settings.categories.organization", { defaultValue: "Organization" })}</h1>

            <p className="text-slate-500 dark:text-slate-400 mt-1 font-medium">{t("common.settings.categories.organization_desc", { defaultValue: "Manage departments and employment phases" })}</p>

          </div>

        </div>

      </div>



      <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">

        <button className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'departments' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")} data-testid="tab-departments" onClick={() => { setActiveTab("departments"); }}><Building2 size={14} /> {t("common.settings.tabs.departments")}</button>
        <button className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'employment_phases' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")} data-testid="tab-phases" onClick={() => { setActiveTab("employment_phases"); }}><Layers size={14} /> {t("common.settings.tabs.employment_phases")}</button>
      </div>



      <div className="animate-in slide-in-from-bottom-4 duration-500">





        {activeTab === "departments" && (

          <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm p-4"><DepartmentsPage /></div>

        )}

        {activeTab === "employment_phases" && (
          <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm p-4">
            <EmploymentPhaseManager wsId={roles[0]?.workspace_id || ""} />
          </div>
        )}

      </div>





    </div>

  );

}
