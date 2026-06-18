import { useNavigate } from "@tanstack/react-router";
import { Building2, Layers, Briefcase, ArrowUpRight } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import EmploymentPhaseManager from "@/components/people/employment-phase-manager";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { getCurrentWorkspaceId } from "@/lib/api";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

import EmployersPage from "./employers-page";


type TabType = "departments" | "employment_phases" | "employers";

export default function OrganizationSettings() {

  const { t } = useTranslation();
  const navigate = useNavigate();
  const wsId = getCurrentWorkspaceId() ?? "";
  const [activeTab, setActiveTab] = useState<TabType>("departments");
  return (

    <div className="space-y-8 animate-in fade-in duration-500">

      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">

        <div className="flex items-center gap-4">

          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50"><Building2 size={24} /></div>

          <div>
            <Typography className="text-2xl font-bold tracking-tight text-slate-850 dark:text-slate-100 normal-case mb-0.5" data-testid="organization-settings-title" variant="h3">{t("common.settings.categories.organization", { defaultValue: "Organization" })}</Typography>
            <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium normal-case tracking-normal" variant="small">{t("common.settings.categories.organization_desc", { defaultValue: "Manage departments and employment phases" })}</Typography>
          </div>

        </div>

      </div>



      <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">

        <Button className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'departments' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")} data-testid="tab-departments" onClick={() => { setActiveTab("departments"); }} variant="ghost"><Building2 size={14} /> {t("common.settings.tabs.departments")}</Button>
        <Button className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'employment_phases' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")} data-testid="tab-phases" onClick={() => { setActiveTab("employment_phases"); }} variant="ghost"><Layers size={14} /> Lộ trình & Mốc</Button>
        <Button className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'employers' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")} data-testid="tab-employers" onClick={() => { setActiveTab("employers"); }} variant="ghost"><Briefcase size={14} /> Pháp nhân</Button>
      </div>



      <div className="animate-in slide-in-from-bottom-4 duration-500">





        {activeTab === "departments" ? (
          <div className="bg-gradient-to-br from-white to-slate-50/50 dark:from-slate-900 dark:to-slate-950/50 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm p-12 flex flex-col items-center text-center max-w-2xl mx-auto my-8">
            <div className="w-16 h-16 rounded-[1.5rem] bg-primary-50 dark:bg-primary-950/50 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50 mb-6">
              <Building2 size={28} />
            </div>
            <Typography className="text-2xl font-bold tracking-tight text-slate-800 dark:text-slate-100" variant="h2">
              {t("common.departments", { defaultValue: "Departments" })}
            </Typography>
            <Typography className="text-slate-500 dark:text-slate-400 mt-3 mb-8 text-sm font-medium leading-relaxed max-w-md" variant="p">
              {t("settings.organization.departments_redirect_desc", { defaultValue: "Quản lý cơ cấu phòng ban, bổ nhiệm Trưởng bộ phận (Manager), thiết lập chính sách lương và theo dõi số lượng nhân sự thuộc từng bộ phận." })}
            </Typography>
            <Button
              className="flex items-center gap-2 group"
              onClick={() => { void navigate({ to: ROUTES.DEPARTMENTS }); }}
              variant="primary"
            >
              {t("settings.organization.go_to_departments", { defaultValue: "Đến trang quản lý phòng ban" })}
              <ArrowUpRight className="transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5" size={16} />
            </Button>
          </div>
        ) : null}

        {activeTab === "employment_phases" ? (
          <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm p-4">
            <EmploymentPhaseManager wsId={wsId} />
          </div>
        ) : null}

        {activeTab === "employers" ? (
          <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm p-4">
            <EmployersPage />
          </div>
        ) : null}

      </div>





    </div>

  );

}
