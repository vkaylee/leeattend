import { useNavigate } from "@tanstack/react-router";
import { Workflow } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useWorkspace } from "@/hooks/use-workspace";
import { Route } from "@/routes/_authenticated/_workspace/workspace/settings/workflows";

import { ApprovalWorkflowsTab } from "./components/overtime/approval-workflows-tab";

interface WorkflowTabContentProps {
  activeTab: "attendance" | "leave" | "overtime";
}

const WorkflowTabContent = ({ activeTab }: WorkflowTabContentProps) => {
  if (activeTab === "leave") {
    return <ApprovalWorkflowsTab entityType="leave_request" />;
  }
  if (activeTab === "attendance") {
    return <ApprovalWorkflowsTab entityType="attendance_adjustment" />;
  }
  return <ApprovalWorkflowsTab entityType="ot_request" />;
};

export default function WorkflowsSettings() {
  const { t } = useTranslation();
  const { workspace: ws } = useWorkspace();
  const search = Route.useSearch();
  const navigate = useNavigate();

  const activeTab = search.tab === "leave" || search.tab === "attendance" ? search.tab : "overtime";

  const setActiveTab = (tab: "overtime" | "leave" | "attendance") => {
    void navigate({
      to: "/workspace/settings/workflows",
      search: { tab },
    });
  };

  if (!ws) return null;

  return (
    <div className="space-y-10 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="flex flex-col gap-2">
        <div className="flex items-center gap-3">
          <div className="p-3 bg-blue-50 dark:bg-blue-900/30 rounded-2xl">
            <Workflow className="w-6 h-6 text-blue-600 dark:text-blue-400" />
          </div>
          <div>
            <Typography className="text-3xl font-black text-slate-800 dark:text-slate-100 tracking-tight" variant="h1">
              {t("settings.categories.workflows", { defaultValue: "Approval Workflows" })}
            </Typography>
            <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1" variant="p">
              {t("settings.categories.workflows_desc", { defaultValue: "Configure multi-step approval workflows for company requests" })}
            </Typography>
          </div>
        </div>
      </div>

      <div className="bg-white dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 p-8 shadow-sm">
        <div className="flex gap-4 border-b border-slate-100 dark:border-slate-800 pb-6 mb-8 overflow-x-auto custom-scrollbar">
          <Button
            className={`px-6 py-3 rounded-2xl font-bold text-sm tracking-tight transition-all duration-300 whitespace-nowrap ${
              activeTab === "overtime"
                ? "bg-slate-800 dark:bg-slate-100 text-white dark:text-slate-900 shadow-md scale-105"
                : "text-slate-500 hover:bg-slate-50 dark:hover:bg-slate-800/50 hover:text-slate-800 dark:hover:text-slate-200"
            }`}
            onClick={() => { setActiveTab("overtime"); }}
            variant="ghost"
          >
            {t("settings.overtime.title", { defaultValue: "Overtime Requests" })}
          </Button>
          <Button
            className={`px-6 py-3 rounded-2xl font-bold text-sm tracking-tight transition-all duration-300 whitespace-nowrap ${
              activeTab === "leave"
                ? "bg-slate-800 dark:bg-slate-100 text-white dark:text-slate-900 shadow-md scale-105"
                : "text-slate-500 hover:bg-slate-50 dark:hover:bg-slate-800/50 hover:text-slate-800 dark:hover:text-slate-200"
            }`}
            onClick={() => { setActiveTab("leave"); }}
            variant="ghost"
          >
            {t("settings.leave.title", { defaultValue: "Leave Requests" })}
          </Button>
          <Button
            className={`px-6 py-3 rounded-2xl font-bold text-sm tracking-tight transition-all duration-300 whitespace-nowrap ${
              activeTab === "attendance"
                ? "bg-slate-800 dark:bg-slate-100 text-white dark:text-slate-900 shadow-md scale-105"
                : "text-slate-500 hover:bg-slate-50 dark:hover:bg-slate-800/50 hover:text-slate-800 dark:hover:text-slate-200"
            }`}
            onClick={() => { setActiveTab("attendance"); }}
            variant="ghost"
          >
            {t("settings.attendance.title", { defaultValue: "Attendance Adjustments" })}
          </Button>
        </div>

        <div className="animate-in fade-in slide-in-from-bottom-2 duration-500">
          <WorkflowTabContent activeTab={activeTab} />
        </div>
      </div>
    </div>
  );
}
