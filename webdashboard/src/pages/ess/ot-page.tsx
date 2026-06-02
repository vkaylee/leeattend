import { Plus, Clock } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { useMyProfile } from "@/hooks/use-ess";
import { useMyOtAssignments, useEmployeeQuota } from "@/hooks/use-overtime-requests";
import { useWorkspace } from "@/hooks/use-workspace";

import { CreateOtRequestModal } from "./components/create-ot-request-modal";
import { MyOtAssignmentsList } from "./components/my-ot-assignments-list";
import { OtQuotaWidget } from "./components/ot-quota-widget";

export default function EssOtPage() {
  const { t } = useTranslation();
  const { workspace: ws } = useWorkspace();
  const { employee } = useMyProfile();
  const wsId = ws?.id;
  const employeeId = employee?.id;

  const [createModalOpen, setCreateModalOpen] = useState(false);

  const { data: assignments, isLoading: loadingAssignments } = useMyOtAssignments(employeeId);
  const { data: quota, isLoading: loadingQuota } = useEmployeeQuota(employeeId || "", new Date().toISOString().split('T')[0]);

  if (!wsId || !employeeId) return null;

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20 p-8">
      <div className="flex flex-col gap-2">
        <div className="flex justify-between items-start">
          <div className="flex items-center gap-3">
            <div className="p-3 bg-indigo-50 dark:bg-indigo-900/30 rounded-2xl">
              <Clock className="w-6 h-6 text-indigo-600 dark:text-indigo-400" />
            </div>
            <div>
              <Typography className="text-3xl font-black text-slate-800 dark:text-slate-100 tracking-tight" variant="h1">
                {t("ess.ot.title", { defaultValue: "My Overtime" })}
              </Typography>
              <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1" variant="p">
                {t("ess.ot.subtitle", { defaultValue: "Request overtime or respond to assignments" })}
              </Typography>
            </div>
          </div>
          <button
            className="flex items-center gap-2 bg-indigo-600 hover:bg-indigo-700 text-white px-6 py-3 rounded-2xl font-bold text-sm tracking-tight transition-all shadow-sm"
            onClick={() => { setCreateModalOpen(true); }}
          >
            <Plus className="w-4 h-4" />
            {t("ess.ot.request", { defaultValue: "Request OT" })}
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <div className="lg:col-span-2 space-y-8">
          <MyOtAssignmentsList assignments={assignments || []} isLoading={loadingAssignments} wsId={wsId} />
        </div>
        <div className="space-y-8">
          <OtQuotaWidget isLoading={loadingQuota} quota={quota} />
        </div>
      </div>

      {!!createModalOpen && (
        <CreateOtRequestModal
          employeeId={employeeId}
          isOpen={createModalOpen}
          onClose={() => { setCreateModalOpen(false); }}
          wsId={wsId}
        />
      )}
    </div>
  );
}
