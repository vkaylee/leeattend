import React from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ArrowRight, MessageSquare, Clock, X, Calendar, CheckCircle2 } from "lucide-react";
import { fetchWorkspaceAttendanceAdjustments, approveAttendanceAdjustment, getCurrentWorkspaceId } from "@/lib/api";
import { useToast } from "../ui/Toast";
import Sheet from "../ui/Sheet";
import { useDateTimeFormatter } from "@/hooks/use-datetime";


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

  const employeeAdjustments = adjustments.filter(adj => adj.employee_id === employeeId);

  const resolveMutation = useMutation({
    mutationFn: (data: { id: string; status: "approved" | "rejected" }) => 
      approveAttendanceAdjustment(wsId, data.id, { status: data.status }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["adjustments"] });
      toast.success(t("common.success"));
      if (onAdjustmentResolved) onAdjustmentResolved();
    },
  });

  return (
    <Sheet
      isOpen={isOpen}
      onClose={onClose}
      title={t("payroll.adjustments.title")}
      subtitle={`${employeeName} • ${employeeAdjustments.length} Pending`}
    >
      <div className="space-y-6 pt-4 animate-in fade-in duration-500">
        {isLoading ? (
          <div className="p-12 text-center text-slate-500 font-bold italic uppercase tracking-widest text-[10px] opacity-50">{t("common.loading")}</div>
        ) : employeeAdjustments.length === 0 ? (
          <div className="p-16 text-center border-2 border-dashed border-slate-100 rounded-[2.5rem] bg-slate-50/30 flex flex-col items-center">
            <div className="bg-white w-16 h-16 rounded-3xl flex items-center justify-center shadow-sm mb-4 border border-slate-100"><CheckCircle2 className="w-8 h-8 text-emerald-400" /></div>
            <h3 className="text-lg font-bold text-slate-700 uppercase tracking-tight">{t("payroll.adjustments.all_clear")}</h3>
            <p className="text-slate-500 text-[10px] font-bold uppercase tracking-widest mt-1 opacity-50">No pending adjustments.</p>
          </div>
        ) : (
          employeeAdjustments.map((adj) => (
            <div key={adj.id} data-testid="adjustment-item" className="bg-white border border-slate-100 rounded-[2rem] p-6 shadow-sm hover:shadow-xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden">
              <div className="flex items-start justify-between mb-6">
                <div className="flex items-center gap-3">
                  <div className="w-8 h-8 rounded-lg bg-slate-50 border border-slate-100 flex items-center justify-center text-slate-500 shadow-inner group-hover:text-primary-600 transition-colors"><Calendar size={14} /></div>
                  <div>
                    <p className="text-[9px] font-bold text-slate-500 uppercase tracking-[0.2em] leading-none mb-1">Requested Date</p>
                    <p className="text-xs font-bold text-slate-700 uppercase tracking-tight">{formatDate(adj.requested_punch_in_time)}</p>
                  </div>
                </div>
                <span className="bg-amber-50 text-amber-600 px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border border-amber-100 shadow-sm animate-pulse">Pending</span>
              </div>

              <div className="flex items-center gap-4 mb-6">
                <div className="flex-1 bg-slate-50/50 p-4 rounded-2xl text-center border border-slate-100 shadow-inner group-hover:bg-white transition-colors">
                  <p className="text-[8px] text-slate-500 font-bold uppercase tracking-widest mb-1.5 opacity-70">Punch In</p>
                  <p className="text-sm font-bold text-slate-700 font-mono tracking-tighter">{formatTime(adj.requested_punch_in_time)}</p>
                </div>
                <div className="w-8 h-8 rounded-full bg-slate-100 flex items-center justify-center shadow-inner"><ArrowRight size={14} className="text-slate-300" /></div>
                <div className="flex-1 bg-slate-50/50 p-4 rounded-2xl text-center border border-slate-100 shadow-inner group-hover:bg-white transition-colors">
                  <p className="text-[8px] text-slate-500 font-bold uppercase tracking-widest mb-1.5 opacity-70">Punch Out</p>
                  <p className="text-sm font-bold text-slate-700 font-mono tracking-tighter">
                    {adj.requested_punch_out_time ? formatTime(adj.requested_punch_out_time) : "--:--"}
                  </p>
                </div>
              </div>

              {adj.reason && (
                <div className="bg-slate-50 p-4 rounded-xl mb-6 border border-slate-100 shadow-inner" data-testid="adjustment-reason">
                  <div className="flex gap-2 items-start"><MessageSquare size={12} className="text-slate-300 mt-0.5 shrink-0" /><p className="text-[10px] text-slate-500 font-bold uppercase tracking-tight leading-relaxed italic">"{adj.reason}"</p></div>
                </div>
              )}

              <div className="grid grid-cols-2 gap-4">
                <button data-testid="reject-adjustment-btn" onClick={() => resolveMutation.mutate({ id: adj.id, status: "rejected" })} disabled={resolveMutation.isPending} className="py-3 rounded-xl text-[10px] font-bold uppercase tracking-widest text-rose-600 bg-rose-50 border border-rose-100 hover:bg-rose-100 transition-all active:scale-95 shadow-sm">Reject</button>
                <button data-testid="approve-adjustment-btn" onClick={() => resolveMutation.mutate({ id: adj.id, status: "approved" })} disabled={resolveMutation.isPending} className="py-3 rounded-xl text-[10px] font-bold uppercase tracking-widest text-emerald-600 bg-emerald-50 border border-emerald-100 hover:bg-emerald-100 transition-all active:scale-95 shadow-sm">Approve</button>
              </div>
            </div>
          ))
        )}
      </div>
    </Sheet>
  );
}