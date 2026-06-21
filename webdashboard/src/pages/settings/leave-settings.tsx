import React from "react";
import { useTranslation } from "react-i18next";

import { AssignPolicyModal } from "@/components/leave/assign-policy-modal";
import { LeavePolicyManager } from "@/components/leave/leave-policy-manager";
import { LeaveTypeManager } from "@/components/leave/leave-type-manager";
import { Typography } from "@/components/ui/typography";

import { LeaveAssignmentsTab } from "./components/leave/leave-assignments-tab";
import { LeaveSettingsHeader } from "./components/leave/leave-settings-header";
import { LeaveSettingsTabs } from "./components/leave/leave-settings-tabs";
import { useLeaveSettings } from "./hooks/use-leave-settings";

export default function LeaveSettings() {
  const { t } = useTranslation();
  const {
    wsId,
    activeTab,
    setActiveTab,
    isAssignModalOpen,
    setIsAssignModalOpen,
    editingAssignment,
    setEditingAssignment,
    assignments,
    assignmentsLoading,
    employees,
    departments,
    deleteAssignment,
    isDeleting,
  } = useLeaveSettings();

  if (!wsId) return null;

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <LeaveSettingsHeader />

      <div className="bg-white dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 p-8 shadow-sm">
        <LeaveSettingsTabs activeTab={activeTab} onTabChange={setActiveTab} />

        <div className="animate-in fade-in slide-in-from-bottom-2 duration-500">
          {activeTab === "types" && (
            <div>
              <div className="mb-8 border-b border-slate-50 dark:border-slate-800 pb-6">
                <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase" variant="h3">
                  {t("settings.leave.tabs.types")}
                </Typography>
                <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 mt-1 uppercase tracking-tight" variant="p">
                  {t("settings.leave.tabs.types_desc")}
                </Typography>
              </div>
              <LeaveTypeManager wsId={wsId} />
            </div>
          )}

          {activeTab === "policies" && (
            <div>
              <div className="mb-8 border-b border-slate-50 dark:border-slate-800 pb-6">
                <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase" variant="h3">
                  {t("settings.leave.tabs.policies")}
                </Typography>
                <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 mt-1 uppercase tracking-tight" variant="p">
                  {t("settings.leave.tabs.policies_desc")}
                </Typography>
              </div>
              <LeavePolicyManager wsId={wsId} />
            </div>
          )}

          {activeTab === "assignments" && (
            <LeaveAssignmentsTab
              assignments={assignments}
              departments={departments}
              employees={employees}
              isDeleting={isDeleting}
              isLoading={assignmentsLoading}
              onAssignClick={() => { setIsAssignModalOpen(true); }}
              onDelete={deleteAssignment}
              onEditClick={(a) => {
                setEditingAssignment(a);
                setIsAssignModalOpen(true);
              }}
            />
          )}
        </div>
      </div>

      <AssignPolicyModal 
        editingAssignment={editingAssignment}
        isOpen={isAssignModalOpen}
        onClose={() => {
          setIsAssignModalOpen(false);
          setEditingAssignment(null);
        }}
        wsId={wsId}
      />
    </div>
  );
}
