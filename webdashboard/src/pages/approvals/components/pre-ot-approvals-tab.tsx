// @ts-nocheck
import { CheckCircle, ArrowRight, Check, X } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useProcessApproval } from "@/hooks/use-approval-workflows";
import { useOvertimeRequests } from "@/hooks/use-overtime-requests";

export function PreOtApprovalsTab({ wsId: _wsId }: { wsId: string }) {
  const { t } = useTranslation();
  const toast = useToast();
  // Fetch pending requests specifically for Pre-OT approval
  const { data: allRequests, isLoading } = useOvertimeRequests({ limit: 20 });
  const { mutate: processApproval, isPending: isProcessing } = useProcessApproval();

  // Filter for pending status
  const pendingRequests = (allRequests || []).filter(req => req.status === "pending_approval" || req.status === "draft");

  const handleAction = (id: string, action: "approve" | "reject") => {
    processApproval(
      {
        entity_type: "ot_request",
        entity_id: id,
        action,
        note: action === "approve" ? "Approved via Manager Portal" : "Rejected via Manager Portal",
      },
      {
        onSuccess: () => {
          toast.success(action === "approve" ? "Request approved" : "Request rejected");
        },
        onError: () => {
          toast.error("Failed to process approval");
        }
      }
    );
  };

  if (isLoading) {
    return (
      <div className="p-32 flex flex-col items-center gap-6 animate-pulse">
        <div className="w-12 h-12 border-4 border-indigo-600 border-t-transparent rounded-full animate-spin"></div>
        <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em]">
          {t("common.loading")}
        </span>
      </div>
    );
  }

  if (pendingRequests.length === 0) {
    return (
      <div className="py-40 text-center flex flex-col items-center border-4 border-dashed border-slate-50 dark:border-slate-800 m-8 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30">
        <div className="w-24 h-24 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-xl border border-slate-100 dark:border-slate-800 mb-8">
          <CheckCircle className="text-emerald-200" size={40} />
        </div>
        <Typography className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h3">
          No Pending Requests
        </Typography>
        <Typography className="text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-widest mt-2 opacity-50 italic" variant="p">
          You have no Pre-OT requests waiting for your approval.
        </Typography>
      </div>
    );
  }

  return (
    <div className="space-y-4 m-8">
      {pendingRequests.map((req) => (
        <div className="bg-white dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 p-6 flex flex-col md:flex-row gap-6 justify-between shadow-sm hover:shadow-md transition-shadow" key={req.id}>
          <div className="space-y-3 flex-1">
            <div className="flex items-center gap-3">
              <span className="px-3 py-1 rounded-lg text-[10px] font-bold uppercase tracking-widest bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400">
                Pending Approval
              </span>
              <Typography className="font-bold text-lg text-slate-800 dark:text-slate-100">
                {(req).employee_names ? (req).employee_names.join(", ") : "Employees"}
              </Typography>
            </div>
 
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 bg-slate-50 dark:bg-slate-800/50 p-4 rounded-2xl">
              <div>
                <span className="block text-[10px] font-bold text-slate-500 uppercase">Date</span>
                <span className="text-sm font-bold text-slate-700 dark:text-slate-300">{new Date(req.planned_date).toLocaleDateString()}</span>
              </div>
              <div>
                <span className="block text-[10px] font-bold text-slate-500 uppercase">Time</span>
                <span className="text-sm font-bold text-slate-700 dark:text-slate-300">{req.planned_start_time} - {req.planned_end_time}</span>
              </div>
              <div>
                <span className="block text-[10px] font-bold text-slate-500 uppercase">Duration</span>
                <span className="text-sm font-bold text-slate-700 dark:text-slate-300">{Math.round(req.planned_minutes / 60 * 10) / 10} hrs</span>
              </div>
              <div>
                <span className="block text-[10px] font-bold text-slate-500 uppercase">Reason</span>
                <span className="text-sm font-bold text-slate-700 dark:text-slate-300 truncate max-w-[150px] inline-block" title={req.reason || ""}>
                  {req.reason || "No reason"}
                </span>
              </div>
            </div>
          </div>
          
          <div className="flex flex-col items-end justify-center gap-2 shrink-0">
            <div className="flex gap-2">
              <Button
                className="normal-case tracking-normal px-4 py-2 bg-emerald-50 text-emerald-600 hover:bg-emerald-100 hover:text-emerald-700 dark:bg-emerald-950/20 dark:text-emerald-300 dark:hover:bg-emerald-900/40 dark:hover:text-emerald-200 border-none flex items-center gap-1.5 rounded-xl font-bold text-xs shadow-none h-auto transition-colors"
                disabled={isProcessing}
                onClick={() => { handleAction(req.id, "approve"); }}
                variant="ghost"
              >
                <Check className="w-3.5 h-3.5" />
                {t("common.approve") || "Approve"}
              </Button>
              <Button
                className="normal-case tracking-normal px-4 py-2 bg-rose-50 text-rose-600 hover:bg-rose-100 hover:text-rose-700 dark:bg-rose-950/20 dark:text-rose-300 dark:hover:bg-rose-900/40 dark:hover:text-rose-200 border-none flex items-center gap-1.5 rounded-xl font-bold text-xs shadow-none h-auto transition-colors"
                disabled={isProcessing}
                onClick={() => { handleAction(req.id, "reject"); }}
                variant="ghost"
              >
                <X className="w-3.5 h-3.5" />
                {t("common.reject") || "Reject"}
              </Button>
            </div>
            {!!req.workflow_id && (
              <span className="text-[10px] text-slate-400 font-medium flex items-center gap-1">
                Workflow Configured <ArrowRight className="w-3 h-3" />
              </span>
            )}
          </div>
        </div>
      ))}


    </div>
  );
}
