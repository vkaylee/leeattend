import { useNavigate } from "@tanstack/react-router";
import { Building2, Layers, Briefcase, ArrowUpRight } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import EmploymentPhaseManager from "@/components/people/employment-phase-manager";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { getStandaloneOrganizationId } from "@/lib/api";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

import EmployersPage from "./employers-page";

type TabType = "departments" | "employment_phases" | "employers";

export default function OrganizationSettings() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const wsId = getStandaloneOrganizationId();
  const [activeTab, setActiveTab] = useState<TabType>("departments");

  return (
    <div className="space-y-6 animate-in fade-in duration-300">
      {/* Apple HIG Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-2 border-b border-slate-200/60 dark:border-slate-800/60">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950/70 flex items-center justify-center text-primary-600 dark:text-primary-400 border border-primary-100/50 dark:border-primary-900/50 shadow-2xs">
            <Building2 size={20} />
          </div>
          <div>
            <Typography className="text-xl font-bold tracking-tight text-slate-800 dark:text-slate-100" data-testid="organization-settings-title" variant="h3">
              {t("common.settings.categories.organization", { defaultValue: "Cơ cấu tổ chức" })}
            </Typography>
            <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium" variant="small">
              {t("common.settings.categories.organization_desc", { defaultValue: "Quản lý phòng ban, lộ trình nhân sự và pháp nhân" })}
            </Typography>
          </div>
        </div>
      </div>

      {/* Apple HIG Segmented Control */}
      <div className="flex items-center">
        <div className="inline-flex items-center gap-1 rounded-xl bg-slate-100/90 dark:bg-slate-800/80 p-1 border border-slate-200/60 dark:border-slate-700/60" role="tablist">
          <button
            aria-selected={activeTab === "departments"}
            className={cn(
              "flex items-center gap-2 px-4 py-1.5 rounded-lg text-xs font-semibold transition-all duration-150 select-none",
              activeTab === "departments"
                ? "bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-2xs font-bold"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200"
            )}
            data-testid="tab-departments"
            onClick={() => setActiveTab("departments")}
            role="tab"
            type="button"
          >
            <Building2 size={14} />
            <span>{t("common.settings.tabs.departments", { defaultValue: "Phòng ban" })}</span>
          </button>
          <button
            aria-selected={activeTab === "employment_phases"}
            className={cn(
              "flex items-center gap-2 px-4 py-1.5 rounded-lg text-xs font-semibold transition-all duration-150 select-none",
              activeTab === "employment_phases"
                ? "bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-2xs font-bold"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200"
            )}
            data-testid="tab-phases"
            onClick={() => setActiveTab("employment_phases")}
            role="tab"
            type="button"
          >
            <Layers size={14} />
            <span>Lộ trình & Mốc</span>
          </button>
          <button
            aria-selected={activeTab === "employers"}
            className={cn(
              "flex items-center gap-2 px-4 py-1.5 rounded-lg text-xs font-semibold transition-all duration-150 select-none",
              activeTab === "employers"
                ? "bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-2xs font-bold"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200"
            )}
            data-testid="tab-employers"
            onClick={() => setActiveTab("employers")}
            role="tab"
            type="button"
          >
            <Briefcase size={14} />
            <span>Pháp nhân</span>
          </button>
        </div>
      </div>

      <div className="animate-in fade-in duration-300">
        {activeTab === "departments" ? (
          <div className="bg-white dark:bg-slate-900/90 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 overflow-hidden shadow-2xs p-8 flex flex-col items-center text-center max-w-xl mx-auto my-6">
            <div className="w-12 h-12 rounded-xl bg-primary-50 dark:bg-primary-950/70 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-2xs border border-primary-100/50 dark:border-primary-900/50 mb-4">
              <Building2 size={24} />
            </div>
            <Typography className="text-xl font-bold tracking-tight text-slate-800 dark:text-slate-100" variant="h3">
              {t("common.departments", { defaultValue: "Cơ cấu phòng ban" })}
            </Typography>
            <Typography className="text-slate-500 dark:text-slate-400 mt-2 mb-6 text-xs leading-relaxed max-w-md" variant="p">
              {t("settings.organization.departments_redirect_desc", { defaultValue: "Quản lý cơ cấu phòng ban, bổ nhiệm Trưởng bộ phận (Manager), thiết lập chính sách lương và theo dõi số lượng nhân sự thuộc từng bộ phận." })}
            </Typography>
            <Button
              className="flex items-center gap-2 rounded-xl px-5 py-2.5 text-xs font-semibold shadow-xs group"
              onClick={() => { void navigate({ to: ROUTES.DEPARTMENTS }); }}
              variant="primary"
            >
              {t("settings.organization.go_to_departments", { defaultValue: "Đến trang quản lý phòng ban" })}
              <ArrowUpRight className="transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5" size={15} />
            </Button>
          </div>
        ) : null}

        {activeTab === "employment_phases" ? (
          <div className="bg-white dark:bg-slate-900/90 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 overflow-hidden shadow-2xs p-6">
            <EmploymentPhaseManager wsId={wsId} />
          </div>
        ) : null}

        {activeTab === "employers" ? (
          <div className="bg-white dark:bg-slate-900/90 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 overflow-hidden shadow-2xs p-6">
            <EmployersPage />
          </div>
        ) : null}
      </div>
    </div>
  );
}
