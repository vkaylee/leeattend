import React from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import Sheet from "../ui/Sheet";
import { 
  fetchWorkspaceAttendanceAdjustments, 
  approveAttendanceAdjustment, 
  getCurrentWorkspaceId,
  AttendanceAdjustmentRequest
} from "@/lib/api";
import { useToast } from "../Toast";
import { Check, X, Clock, Calendar, ArrowRight } from "lucide-react";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

interface AdjustmentDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  employeeId: string;
  employeeName: string;
  onAdjustmentResolved?: () => void;
}

export default function AdjustmentDrawer({
  isOpen,
  onClose,
  employeeId,
  employeeName,
  onAdjustmentResolved
}: AdjustmentDrawerProps) {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const toast = useToast();
  const { formatDate, formatTime } = useDateTimeFormatter();

  const { data: adjustments = [], isLoading } = useQuery({
    queryKey: ["adjustments", wsId, "pending"],
    queryFn: () => fetchWorkspaceAttendanceAdjustments(wsId, "pending"),
    enabled: isOpen && !!wsId,
  });

  // Client-side filter for the specific employee
  const employeeAdjustments = adjustments.filter(adj => adj.employee_id === employeeId);

  const resolveMutation = useMutation({
    mutationFn: (data: { id: string; status: "approved" | "rejected" }) => 
      approveAttendanceAdjustment(wsId, data.id, { status: data.status }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["adjustments"] });
      toast.success(t("payroll.adjustments.resolved_success", "Adjustment resolved successfully"));
      if (onAdjustmentResolved) onAdjustmentResolved();
    },
    onError: () => toast.error(t("payroll.adjustments.resolved_error", "Failed to resolve adjustment"))
  });

  return (
    <Sheet
      isOpen={isOpen}
      onClose={onClose}
      title={t("payroll.adjustments.title", "Pending Adjustments")}
      subtitle={`${employeeName} • ${employeeAdjustments.length} Pending`}
    >
      <div className="space-y-4">
        {isLoading ? (
          <div className="text-center py-8 text-slate-400 italic">
            {t("common.loading", "Loading...")}
          </div>
        ) : employeeAdjustments.length === 0 ? (
          <div className="text-center py-12 bg-slate-50 rounded-2xl border border-slate-100 border-dashed">
            <Check className="w-8 h-8 text-emerald-300 mx-auto mb-2" />
            <p className="text-sm font-bold text-slate-400">
              {t("payroll.adjustments.all_clear", "All caught up! No pending adjustments.")}
            </p>
          </div>
        ) : (
          employeeAdjustments.map((adj) => (
            <div 
              key={adj.id} 
              data-testid="adjustment-item"
              className="bg-white border border-slate-100 rounded-2xl p-4 shadow-sm hover:shadow-md transition-shadow"
            >
              <div className="flex items-start justify-between mb-3">
                <div className="flex items-center gap-2 text-slate-500 text-xs font-bold uppercase tracking-wide">
                  <Calendar className="w-3 h-3" />
                  {formatDate(adj.requested_punch_in_time)}
                </div>
                <div className="bg-amber-100 text-amber-700 px-2 py-0.5 rounded-full text-[10px] font-black uppercase">
                  Pending
                </div>
              </div>

              <div className="flex items-center gap-3 mb-4">
                <div className="flex-1 bg-slate-50 p-2 rounded-lg text-center">
                  <p className="text-[10px] text-slate-400 font-bold uppercase">In</p>
                  <p className="text-sm font-black text-slate-900">{formatTime(adj.requested_punch_in_time)}</p>
                </div>
                <ArrowRight className="w-4 h-4 text-slate-300" />
                <div className="flex-1 bg-slate-50 p-2 rounded-lg text-center">
                  <p className="text-[10px] text-slate-400 font-bold uppercase">Out</p>
                  <p className="text-sm font-black text-slate-900">
                    {adj.requested_punch_out_time ? formatTime(adj.requested_punch_out_time) : "--:--"}
                  </p>
                </div>
              </div>

              {adj.reason && (
                <div className="bg-slate-50 p-3 rounded-xl mb-4 text-xs text-slate-600 italic border border-slate-100" data-testid="adjustment-reason">
                  "{adj.reason}"
                </div>
              )}

              <div className="grid grid-cols-2 gap-3">
                <button
                  data-testid="reject-adjustment-btn"
                  onClick={() => resolveMutation.mutate({ id: adj.id, status: "rejected" })}
                  disabled={resolveMutation.isPending}
                  className="py-2 rounded-xl text-xs font-bold text-rose-600 bg-rose-50 hover:bg-rose-100 transition-colors flex items-center justify-center gap-2"
                >
                  <X className="w-3 h-3" /> Reject
                </button>
                <button
                  data-testid="approve-adjustment-btn"
                  onClick={() => resolveMutation.mutate({ id: adj.id, status: "approved" })}
                  disabled={resolveMutation.isPending}
                  className="py-2 rounded-xl text-xs font-bold text-emerald-600 bg-emerald-50 hover:bg-emerald-100 transition-colors flex items-center justify-center gap-2"
                >
                  <Check className="w-3 h-3" /> Approve
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </Sheet>
  );
}
