import { useNavigate } from "@tanstack/react-router";
import { Workflow } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { SettingsHeader } from "@/components/ui/settings/settings-header";
import { SettingsSegmentedControl, type SegmentedOption } from "@/components/ui/settings/settings-segmented-control";
import { useOrganization } from "@/hooks/use-organization";
import { ROUTES } from "@/lib/routes";
import { Route } from "@/routes/_authenticated/_workspace/settings/workflows";

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
  const { workspace: ws } = useOrganization();
  const search = Route.useSearch();
  const navigate = useNavigate();

  const activeTab = search.tab === "leave" || search.tab === "attendance" ? search.tab : "overtime";

  const setActiveTab = (tab: "overtime" | "leave" | "attendance") => {
    void navigate({
      to: ROUTES.SETTINGS.WORKFLOWS,
      search: { tab },
    });
  };

  if (!ws) return null;

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <SettingsHeader
        description={t("settings.categories.workflows_desc", { defaultValue: "Cấu hình luồng duyệt nhiều cấp cho đơn từ và yêu cầu" })}
        icon={<Workflow size={20} />}
        title={t("settings.categories.workflows", { defaultValue: "Quy trình phê duyệt" })}
      />

      <div className="bg-white dark:bg-slate-900/90 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 p-6 sm:p-8 shadow-2xs space-y-6">
        <div className="overflow-x-auto max-w-full pb-1">
          <SettingsSegmentedControl
            onChange={(val) => setActiveTab(val as "overtime" | "leave" | "attendance")}
            options={[
              { label: t("settings.overtime.title", { defaultValue: "Tăng ca" }), value: "overtime" },
              { label: t("settings.leave.title", { defaultValue: "Nghỉ phép" }), value: "leave" },
              { label: t("settings.attendance.title", { defaultValue: "Điều chỉnh công" }), value: "attendance" },
            ]}
            testId="workflows-tabs"
            value={activeTab}
          />
        </div>

        <div className="animate-in fade-in slide-in-from-bottom-2 duration-500">
          <WorkflowTabContent activeTab={activeTab} />
        </div>
      </div>
    </div>
  );
}
