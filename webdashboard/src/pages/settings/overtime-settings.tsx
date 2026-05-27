import { Clock } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { useWorkspace } from "@/hooks/use-workspace";

import { ApprovalWorkflowsTab } from "./components/overtime/approval-workflows-tab";
import { OvertimeQuotasTab } from "./components/overtime/overtime-quotas-tab";

export default function OvertimeSettings() {
  const { t } = useTranslation();
  const { workspace: ws } = useWorkspace();
  const [activeTab, setActiveTab] = useState<"quotas" | "workflows">("quotas");

  if (!ws) return null;

  return (
    <div className="space-y-10 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="flex flex-col gap-2">
        <div className="flex items-center gap-3">
          <div className="p-3 bg-blue-50 dark:bg-blue-900/30 rounded-2xl">
            <Clock className="w-6 h-6 text-blue-600 dark:text-blue-400" />
          </div>
          <div>
            <Typography className="text-3xl font-black text-slate-800 dark:text-slate-100 tracking-tight" variant="h1">
              {t("settings.categories.overtime", { defaultValue: "Overtime Settings" })}
            </Typography>
            <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1" variant="p">
              {t("settings.categories.overtime_desc", { defaultValue: "Manage overtime quotas and approvals" })}
            </Typography>
          </div>
        </div>
      </div>

      <div className="bg-white dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 p-8 shadow-sm">
        <div className="flex gap-4 border-b border-slate-100 dark:border-slate-800 pb-6 mb-8 overflow-x-auto custom-scrollbar">
          <button
            className={`px-6 py-3 rounded-2xl font-bold text-sm tracking-tight transition-all duration-300 whitespace-nowrap ${
              activeTab === "quotas"
                ? "bg-slate-800 dark:bg-slate-100 text-white dark:text-slate-900 shadow-md scale-105"
                : "text-slate-500 hover:bg-slate-50 dark:hover:bg-slate-800/50 hover:text-slate-800 dark:hover:text-slate-200"
            }`}
            onClick={() => { setActiveTab("quotas"); }}
          >
            {t("settings.overtime.tabs.quotas", { defaultValue: "Quotas & EAV Configs" })}
          </button>
          <button
            className={`px-6 py-3 rounded-2xl font-bold text-sm tracking-tight transition-all duration-300 whitespace-nowrap ${
              activeTab === "workflows"
                ? "bg-slate-800 dark:bg-slate-100 text-white dark:text-slate-900 shadow-md scale-105"
                : "text-slate-500 hover:bg-slate-50 dark:hover:bg-slate-800/50 hover:text-slate-800 dark:hover:text-slate-200"
            }`}
            onClick={() => { setActiveTab("workflows"); }}
          >
            {t("settings.overtime.tabs.workflows", { defaultValue: "Approval Workflows" })}
          </button>
        </div>

        <div className="animate-in fade-in slide-in-from-bottom-2 duration-500">
          {activeTab === "quotas" && <OvertimeQuotasTab wsId={ws.id} />}
          {activeTab === "workflows" && <ApprovalWorkflowsTab wsId={ws.id} />}
        </div>
      </div>
    </div>
  );
}
