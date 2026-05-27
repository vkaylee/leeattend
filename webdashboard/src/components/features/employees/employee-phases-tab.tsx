/* eslint-disable complexity */
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  AlertTriangle,
  CheckCircle2,
  Milestone,
  Pencil,
  Plus,
  XCircle,
} from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import {
  assignEmployeePhase,
  fetchEmployeePhases,
  fetchEmploymentPhases,
  updateEmployeePhaseStatus,
} from "@/lib/api/employees";
import { EmployeePhase } from "@/lib/api/types";
import { cn } from "@/lib/utils";
import { useEmployeeContracts } from "@/pages/employees/hooks/use-employee-contracts";

interface Props {
  employeeId: string;
  formatDate: (date: string) => string;
  wsId: string;
}

// eslint-disable-next-line max-lines-per-function
export default function EmployeePhasesTab({ wsId, employeeId, formatDate }: Props) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  
  const [isAssignModalOpen, setIsAssignModalOpen] = useState(false);
  const [isStatusModalOpen, setIsStatusModalOpen] = useState(false);
  const [selectedPhase, setSelectedPhase] = useState<EmployeePhase | null>(null);

  const { data: phases, isLoading: isLoadingPhases } = useQuery({
    queryKey: ["employee-phases", employeeId],
    queryFn: () => fetchEmployeePhases(wsId, employeeId),
    enabled: !!wsId && !!employeeId,
  });

  const { data: definitions } = useQuery({
    queryKey: ["employment-phases", wsId],
    queryFn: () => fetchEmploymentPhases(wsId),
    enabled: !!wsId,
  });

  const { contracts } = useEmployeeContracts(wsId, employeeId);

  const [assignData, setAssignData] = useState({
    phase_definition_id: "",
    start_date: "",
    end_date: "",
  });

  const [statusData, setStatusData] = useState({
    status: "completed",
    evaluation_notes: "",
  });

  const assignMutation = useMutation({
    mutationFn: (data: typeof assignData) => assignEmployeePhase(wsId, employeeId, data),
    onSuccess: () => {
      toast.success(t("common.success"));
      void queryClient.invalidateQueries({ queryKey: ["employee-phases", employeeId] });
      setIsAssignModalOpen(false);
      setAssignData({ phase_definition_id: "", start_date: "", end_date: "" });
    },
    onError: (err: unknown) => {
      const error = err as Error;
      toast.error(error.message || t("common.error"));
    }
  });

  const statusMutation = useMutation({
    mutationFn: (data: { phaseId: string; status: string; evaluation_notes: string }) => 
      updateEmployeePhaseStatus(wsId, employeeId, data.phaseId, { status: data.status, evaluation_notes: data.evaluation_notes }),
    onSuccess: () => {
      toast.success(t("common.success"));
      void queryClient.invalidateQueries({ queryKey: ["employee-phases", employeeId] });
      setIsStatusModalOpen(false);
      setSelectedPhase(null);
      setStatusData({ status: "completed", evaluation_notes: "" });
    },
    onError: (err: unknown) => {
      const error = err as Error;
      toast.error(error.message || t("common.error"));
    }
  });

  const isOutOfContractBounds = useMemo(() => {
    if (!assignData.start_date || !assignData.end_date || contracts.length === 0) {
      return contracts.length === 0 ? "Nhân sự này chưa có hợp đồng nào, Cột mốc này có thể không hợp lệ." : null;
    }
    const startDates = contracts.map(c => new Date(c.start_date).getTime());
    const endDates = contracts.map(c => c.end_date ? new Date(c.end_date).getTime() : Infinity);
    const minStart = Math.min(...startDates);
    const maxEnd = Math.max(...endDates);
    const phaseStart = new Date(assignData.start_date).getTime();
    const phaseEnd = new Date(assignData.end_date).getTime();
    
    if (phaseStart < minStart || phaseEnd > maxEnd) {
      return "Thời gian của cột mốc này đang vượt ra ngoài tổng thời hạn các hợp đồng hiện tại.";
    }
    return null;
  }, [assignData.start_date, assignData.end_date, contracts]);

  const handleAssignSubmit = (e: React.SyntheticEvent) => {
    e.preventDefault();
    if (!assignData.phase_definition_id || !assignData.start_date || !assignData.end_date) {
      toast.error("Please fill all required fields"); return;
    }
    assignMutation.mutate(assignData);
  };

  const handleStatusSubmit = (e: React.SyntheticEvent) => {
    e.preventDefault();
    if (!selectedPhase) return;
    statusMutation.mutate({
      phaseId: selectedPhase.id,
      ...statusData
    });
  };

  const openStatusModal = (phase: EmployeePhase) => {
    setSelectedPhase(phase);
    setStatusData({
      status: phase.status === "active" ? "completed" : phase.status,
      evaluation_notes: phase.evaluation_notes ?? "",
    });
    setIsStatusModalOpen(true);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between px-2">
        <div className="max-w-xl">
          <Typography className="text-sm font-bold text-slate-800 dark:text-slate-100 uppercase tracking-widest flex items-center gap-2" variant="h3">
            Hành trình Phát triển
            <span className="bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 px-2 py-0.5 rounded-full text-[10px] normal-case tracking-normal border border-blue-100 dark:border-blue-800/50">Phục vụ Đánh giá</span>
          </Typography>
          <Typography className="text-[11px] font-medium text-slate-500 dark:text-slate-400 mt-2 leading-relaxed" variant="p">
            Theo dõi các cột mốc trong lộ trình phát triển của nhân sự (Ví dụ: Thử việc, Lên Senior, Thăng chức). 
            Đây cũng là nơi lưu trữ đánh giá hiệu suất khi hoàn thành mỗi cột mốc.
          </Typography>
        </div>
        <PermissionGate permission="employment_phases:manage">
          <Button 
            className="flex items-center gap-2 bg-primary-50 dark:bg-primary-950/50 text-primary-600 dark:text-primary-400 px-4 py-2 rounded-xl font-bold text-[10px] uppercase tracking-widest hover:bg-primary-100 dark:hover:bg-primary-900 transition-all border border-primary-100 dark:border-primary-800 active:scale-95 h-auto shadow-none"
            onClick={() => { setIsAssignModalOpen(true); }}
            type="button"
            variant="ghost"
          >
            <Plus size={14} /> Thêm Cột mốc
          </Button>
        </PermissionGate>
      </div>

      <div className="space-y-4">
        {isLoadingPhases ? (
          <div className="p-10 text-center">
            <div className="w-6 h-6 border-2 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div>
          </div>
        ) : phases?.length === 0 ? (
          <div className="p-10 text-center bg-slate-50 dark:bg-slate-800/30 rounded-[2rem] border border-dashed border-slate-200 dark:border-slate-700">
            <Milestone className="w-10 h-10 text-slate-300 dark:text-slate-600 mx-auto mb-3" />
            <Typography className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest" variant="p">Chưa có cột mốc nào trong hành trình</Typography>
          </div>
        ) : (
          <div className="relative pl-10 space-y-6 before:absolute before:left-[19px] before:top-4 before:bottom-4 before:w-px before:bg-slate-200 dark:before:bg-slate-700">
            {phases?.map((phase, idx) => {
              const def = definitions?.find(d => d.id === phase.phase_definition_id);
              const isCompleted = phase.status === "completed";
              const isTerminated = phase.status === "terminated";
              
              return (
                <div className="relative animate-in slide-in-from-left duration-500" key={phase.id} style={{ animationDelay: `${idx * 100}ms` }}>
                  <div className={cn(
                    "absolute -left-[2.15rem] top-4 w-5 h-5 rounded-full border-4 border-white dark:border-slate-900 shadow-sm z-10",
                    isCompleted ? "bg-emerald-500" : isTerminated ? "bg-rose-500" : "bg-primary-500 ring-4 ring-primary-50 dark:ring-primary-900/30"
                  )}></div>
                  
                  <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 rounded-[2rem] shadow-sm group hover:shadow-md transition-all">
                    <div className="flex justify-between items-start mb-4">
                      <div>
                        <div className="flex items-center gap-2 mb-1">
                          <Typography className="font-bold text-slate-800 dark:text-slate-100 uppercase tracking-tight" variant="h4">{def?.name ?? "Unknown Phase"}</Typography>
                          <span className={cn(
                            "text-[8px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-full",
                            isCompleted ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-400" :
                            isTerminated ? "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-400" :
                            "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400"
                          )}>
                            {phase.status}
                          </span>
                        </div>
                        <Typography className="text-[10px] text-slate-500 dark:text-slate-400 font-mono font-bold tracking-widest" variant="p">
                          {formatDate(phase.start_date)} — {formatDate(phase.end_date)}
                        </Typography>
                      </div>
                      
                      <PermissionGate permission="employment_phases:manage">
                        <Button 
                          className="p-2 text-slate-300 dark:text-slate-600 hover:text-primary-600 dark:hover:text-primary-400 hover:bg-slate-50 dark:hover:bg-slate-800 rounded-xl transition-all h-auto w-auto shadow-none"
                          onClick={() => { openStatusModal(phase); }}
                          size="icon"
                          title="Cập nhật trạng thái"
                          type="button"
                          variant="ghost"
                        >
                          <Pencil size={14} />
                        </Button>
                      </PermissionGate>
                    </div>

                    {!!phase.evaluation_notes && (
                      <div className="mt-4 p-4 bg-slate-50 dark:bg-slate-800/50 rounded-2xl border border-slate-100 dark:border-slate-700/50">
                        <Typography className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mb-1" variant="p">Đánh giá</Typography>
                        <Typography className="text-sm text-slate-600 dark:text-slate-300" variant="p">{phase.evaluation_notes}</Typography>
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      <Modal icon={<Milestone size={24} />} isOpen={isAssignModalOpen} maxWidth="md" onClose={() => { setIsAssignModalOpen(false); }} subtitle="Ghi nhận một Cột mốc mới cho nhân viên. Các Cột mốc được theo dõi hoàn toàn độc lập với Hợp đồng." title="Thêm Cột mốc">
        <form className="space-y-6 pt-4 animate-in fade-in" onSubmit={handleAssignSubmit}>
          <div className="space-y-3">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="phase_definition_id">Loại Cột mốc</label>
            <select 
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm"
              id="phase_definition_id"
              onChange={e => {
                const newPhaseId = e.target.value;
                const def = definitions?.find(d => d.id === newPhaseId);
                let newEndDate = assignData.end_date;
                
                if (def?.rule_config?.duration_days && assignData.start_date) {
                  const startDate = new Date(assignData.start_date);
                  startDate.setDate(startDate.getDate() + def.rule_config.duration_days);
                  newEndDate = startDate.toISOString().split('T')[0];
                }
                
                setAssignData({...assignData, phase_definition_id: newPhaseId, end_date: newEndDate});
              }}
              required
              value={assignData.phase_definition_id}
            >
              <option value="">-- Chọn Cột mốc --</option>
              {definitions?.filter(d => d.is_active).map(def => (
                <option key={def.id} value={def.id}>{def.name} ({def.phase_type})</option>
              ))}
            </select>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="start_date">Từ ngày</label>
              <input 
                className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-mono font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm"
                id="start_date"
                onChange={e => {
                  const newStartDate = e.target.value;
                  const def = definitions?.find(d => d.id === assignData.phase_definition_id);
                  let newEndDate = assignData.end_date;
                  
                  if (def?.rule_config?.duration_days && newStartDate) {
                    const startDate = new Date(newStartDate);
                    startDate.setDate(startDate.getDate() + def.rule_config.duration_days);
                    newEndDate = startDate.toISOString().split('T')[0];
                  }
                  
                  setAssignData({...assignData, start_date: newStartDate, end_date: newEndDate});
                }}
                required
                type="date"
                value={assignData.start_date}
              />
            </div>
            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="end_date">Đến ngày</label>
              <input 
                className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-mono font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm"
                id="end_date"
                onChange={e => { setAssignData({...assignData, end_date: e.target.value}); }}
                required
                type="date"
                value={assignData.end_date}
              />
            </div>
          </div>

          {!!isOutOfContractBounds && (
            <div className="bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800/50 p-4 rounded-2xl flex gap-3 items-start animate-in fade-in slide-in-from-top-2">
              <AlertTriangle className="text-amber-500 shrink-0 mt-0.5" size={18} />
              <div>
                <Typography className="text-[11px] font-bold text-amber-800 dark:text-amber-500 uppercase tracking-widest mb-1" variant="p">Cảnh báo Mềm</Typography>
                <Typography className="text-xs text-amber-700 dark:text-amber-600 font-medium leading-relaxed" variant="p">{isOutOfContractBounds} Bạn vẫn có thể lưu lại nếu đây là chủ đích của mình.</Typography>
              </div>
            </div>
          )}

          <div className="flex gap-4 pt-4 border-t border-slate-50 dark:border-slate-800">
            <Button className="flex-1 px-6 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-500 dark:text-slate-400 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95 h-auto shadow-none" onClick={() => { setIsAssignModalOpen(false); }} type="button" variant="outline">Hủy</Button>
            <Button className="flex-[1.5] bg-primary-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50 h-auto" disabled={assignMutation.isPending} type="submit" variant="primary">{assignMutation.isPending ? "Đang lưu..." : "Thêm Cột mốc"}</Button>
          </div>
        </form>
      </Modal>

      <Modal icon={<CheckCircle2 size={24} />} isOpen={isStatusModalOpen} maxWidth="md" onClose={() => { setIsStatusModalOpen(false); }} subtitle="Ghi nhận kết quả đánh giá cho Cột mốc này." title="Cập nhật kết quả">
        <form className="space-y-6 pt-4 animate-in fade-in" onSubmit={handleStatusSubmit}>
          <div className="space-y-3">
            <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 block">Trạng thái mới</span>
            <div className="flex gap-4">
              <Button 
                className={cn("flex-1 py-3 px-4 rounded-xl font-bold text-xs uppercase tracking-widest border transition-all flex items-center justify-center gap-2 h-auto shadow-none border-transparent", statusData.status === "completed" ? "bg-emerald-50 text-emerald-600 border-emerald-200 hover:bg-emerald-50" : "bg-white text-slate-400 border-slate-200 hover:border-emerald-200 hover:bg-slate-50")}
                onClick={() => { setStatusData({...statusData, status: "completed"}); }}
                type="button"
                variant="ghost"
              >
                <CheckCircle2 size={16} /> Completed
              </Button>
              <Button 
                className={cn("flex-1 py-3 px-4 rounded-xl font-bold text-xs uppercase tracking-widest border transition-all flex items-center justify-center gap-2 h-auto shadow-none border-transparent", statusData.status === "terminated" ? "bg-rose-50 text-rose-600 border-rose-200 hover:bg-rose-50" : "bg-white text-slate-400 border-slate-200 hover:border-rose-200 hover:bg-slate-50")}
                onClick={() => { setStatusData({...statusData, status: "terminated"}); }}
                type="button"
                variant="ghost"
              >
                <XCircle size={16} /> Terminated
              </Button>
            </div>
          </div>

          <div className="space-y-3">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="evaluation_notes">Ghi chú Đánh giá</label>
            <textarea 
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm min-h-[120px] resize-none"
              id="evaluation_notes"
              onChange={e => { setStatusData({...statusData, evaluation_notes: e.target.value}); }}
              placeholder="Ghi chú đánh giá sau khi kết thúc cột mốc này..."
              value={statusData.evaluation_notes}
            />
          </div>

          <div className="flex gap-4 pt-4 border-t border-slate-50 dark:border-slate-800">
            <Button className="flex-1 px-6 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-500 dark:text-slate-400 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95 h-auto shadow-none" onClick={() => { setIsStatusModalOpen(false); }} type="button" variant="outline">Hủy</Button>
            <Button className="flex-[1.5] bg-primary-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50 h-auto" disabled={statusMutation.isPending} type="submit" variant="primary">{statusMutation.isPending ? "Đang lưu..." : "Cập nhật"}</Button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
