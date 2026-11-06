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
    <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-2 border-b border-slate-200/60 dark:border-slate-800/60">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-blue-50 dark:bg-blue-950/70 flex items-center justify-center text-blue-600 dark:text-blue-400 shadow-2xs border border-blue-100/50 dark:border-blue-900/50">
            <Workflow size={20} />
          </div>
          <div>
            <Typography className="text-xl font-bold tracking-tight text-slate-800 dark:text-slate-100 normal-case" variant="h3">
              {t("settings.categories.workflows", { defaultValue: "Quy trình phê duyệt" })}
            </Typography>
            <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium normal-case tracking-normal" variant="small">
              {t("settings.categories.workflows_desc", { defaultValue: "Cấu hình luồng duyệt nhiều cấp cho đơn từ và yêu cầu" })}
            </Typography>
          </div>
        </div>
      </div>

      <div className="bg-white dark:bg-slate-900/90 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 p-6 sm:p-8 shadow-2xs space-y-6">
        <div className="inline-flex items-center gap-1 rounded-xl bg-slate-100/90 dark:bg-slate-800/80 p-1 border border-slate-200/60 dark:border-slate-700/60" role="tablist">
          {[
            { id: "overtime", label: t("settings.overtime.title", { defaultValue: "Tăng ca" }) },
            { id: "leave", label: t("settings.leave.title", { defaultValue: "Nghỉ phép" }) },
            { id: "attendance", label: t("settings.attendance.title", { defaultValue: "Điều chỉnh công" }) },
          ].map((tab) => (
            <Button
              aria-selected={activeTab === tab.id}
              className={`px-4 py-1.5 rounded-lg text-xs font-semibold transition-all select-none ${
                activeTab === tab.id
                  ? "bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-2xs font-bold"
                  : "text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200"
              }`}
              key={tab.id}
              onClick={() => { setActiveTab(tab.id as "overtime" | "leave" | "attendance"); }}
              role="tab"
              type="button"
            >
              {tab.label}
            </Button>
          ))}
        </div>

        <div className="animate-in fade-in slide-in-from-bottom-2 duration-500">
          <WorkflowTabContent activeTab={activeTab} />
        </div>
      </div>
    </div>
  );
}
