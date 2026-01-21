import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Clock, ArrowRightLeft, History, AlertCircle, User, Plus } from "lucide-react";
import { 
  fetchEmployees, 
  punchIn, 
  punchOut, 
  fetchTimeHistory, 
  getCurrentWorkspaceId,
  submitAttendanceAdjustment,
  fetchWorkspace,
  Employee,
  TimeEntry
} from "../lib/api";
import { useToast } from "../components/Toast";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { format } from "date-fns";
import { EmployeeSearch } from "@/components/common/EmployeeSearch";
import Modal from "../components/ui/Modal";
import { toUTCISO } from "../lib/datetime";
import { cn } from "@/lib/utils";

export default function TimeTrackingPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const toast = useToast();
  const { formatDate, formatTime, formatTime24h } = useDateTimeFormatter();
  const [selectedEmployeeId, setSelectedEmployeeId] = useState<string>("");
  const [isAdjustmentModalOpen, setIsAdjustmentModalOpen] = useState(false);
  const [adjustmentData, setAdjustmentData] = useState({
    date: format(new Date(), 'yyyy-MM-dd'),
    punchIn: "08:00",
    punchOut: "17:00",
    reason: "",
    isBreakOvertime: false,
    timeEntryId: undefined as string | undefined
  });

  const { data: employees = [] } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId!),
    enabled: !!wsId,
  });

  const { data: workspace } = useQuery({
    queryKey: ["workspace", wsId],
    queryFn: () => fetchWorkspace(wsId!),
    enabled: !!wsId,
  });

  const selectedEmployee = employees.find(e => e.id === selectedEmployeeId);

  const { startDate, endDate } = React.useMemo(() => {
    const now = new Date();
    const end = now.toISOString();
    const start = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000).toISOString();
    return { startDate: start, endDate: end };
  }, []);

  const { data: history = [], isLoading: isLoadingHistory } = useQuery({
    queryKey: ["time-history", wsId, selectedEmployeeId],
    queryFn: () => fetchTimeHistory(wsId!, selectedEmployeeId, startDate, endDate),
    enabled: !!wsId && !!selectedEmployeeId,
  });

  const activeEntry = history.find(entry => !entry.punch_out_time);
  const isOnBreak = activeEntry?.entry_type === "break";

  const punchInMutation = useMutation({
    mutationFn: (data: { employee_id: string; entry_type?: string; note?: string }) => punchIn(wsId!, data),
    onSuccess: (_, variables) => {
      const type = variables.entry_type === "break" ? t("time_tracking.messages.punch_in_break") : t("time_tracking.messages.punch_in_work");
      toast.success(t("time_tracking.messages.punch_success", { type }));
      queryClient.invalidateQueries({ queryKey: ["time-history"] });
    },
  });

  const punchOutMutation = useMutation({
    mutationFn: (data: { note?: string }) => punchOut(wsId!, selectedEmployeeId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["time-history"] });
      toast.success(t("common.success"));
    },
  });

  const adjustMutation = useMutation({
    mutationFn: (data: any) => submitAttendanceAdjustment(wsId!, data),
    onSuccess: () => {
      setIsAdjustmentModalOpen(false);
      toast.success(t("time_tracking.messages.adjustment_success"));
      queryClient.invalidateQueries({ queryKey: ["time-history"] });
    },
  });

  const handleAdjustSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const tz = workspace?.timezone || "UTC";
    const punchInTime = toUTCISO(adjustmentData.date, adjustmentData.punchIn, tz);
    const punchOutTime = toUTCISO(adjustmentData.date, adjustmentData.punchOut, tz);
    adjustMutation.mutate({
      employee_id: selectedEmployeeId,
      time_entry_id: adjustmentData.timeEntryId,
      requested_punch_in_time: punchInTime,
      requested_punch_out_time: punchOutTime,
      is_break_overtime: adjustmentData.isBreakOvertime,
      reason: adjustmentData.reason
    });
  };

  if (!wsId) return <div className="p-8 text-center font-bold text-slate-400 italic">{t("common.select_workspace")}</div>;

  return (
    <div className="space-y-8 animate-in fade-in duration-700">
      <header className="px-2">
        <h1 className="text-3xl font-bold tracking-tight text-slate-900">{t("time_tracking.title")}</h1>
        <p className="text-slate-500 mt-1">{t("time_tracking.subtitle")}</p>
      </header>

      <div className="bg-white border border-slate-200 rounded-[32px] p-8 shadow-sm group">
        <label className="block text-[10px] font-black text-slate-400 uppercase tracking-widest mb-4 ml-1 group-focus-within:text-primary-500 transition-colors">
          {t("time_tracking.select_employee_label")}
        </label>
        <div className="max-w-xl">
          <EmployeeSearch
            testId="employee-select"
            value={selectedEmployeeId}
            onChange={setSelectedEmployeeId}
          />
        </div>

      </div>

      {selectedEmployee ? (
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 animate-in slide-in-from-bottom-4 duration-500">
          {/* Punch Card */}
          <div className="lg:col-span-4 bg-white border border-slate-200 rounded-[40px] p-10 shadow-2xl shadow-slate-200/50 flex flex-col items-center text-center relative overflow-hidden group">
            <div className="absolute top-0 right-0 p-6 opacity-5 group-hover:opacity-10 transition-opacity">
               <Clock size={120} className="-rotate-12" />
            </div>

            <div className={`w-24 h-24 rounded-[2rem] flex items-center justify-center mb-8 transition-all duration-700 shadow-inner ${activeEntry ? (isOnBreak ? 'bg-amber-50 text-amber-600 animate-pulse' : 'bg-primary-50 text-primary-600 animate-pulse') : 'bg-emerald-50 text-emerald-600 shadow-emerald-50'}`}>
              <Clock className="w-10 h-10" />
            </div>
            
            <div className="mb-10 relative z-10">
              <h2 className="text-2xl font-black text-slate-900 tracking-tight uppercase leading-tight">{selectedEmployee.first_name} {selectedEmployee.last_name}</h2>
              <div className="mt-3 flex items-center justify-center gap-2">
                <span className={`w-2 h-2 rounded-full ${activeEntry ? 'bg-amber-500 animate-ping' : 'bg-emerald-500'}`} />
                <p className="text-slate-400 text-xs font-black uppercase tracking-widest">
                  {activeEntry ? (isOnBreak ? t("time_tracking.status.on_break") : t("time_tracking.status.working")) : t("time_tracking.status.off_duty")}
                </p>
              </div>
            </div>

            {activeEntry ? (
              <div className="w-full space-y-6 relative z-10">
                <div className="bg-slate-50 border border-slate-100 p-6 rounded-3xl shadow-inner">
                  <div className="text-slate-400 font-black uppercase text-[9px] tracking-[0.2em] mb-2">
                    {isOnBreak ? t("time_tracking.status.break_start_at") : t("time_tracking.status.shift_start_at")}
                  </div>
                  <div className="font-black text-4xl text-slate-800 tracking-tighter font-mono">{formatTime(activeEntry.punch_in_time)}</div>
                </div>
                
                <div className="flex flex-col gap-4">
                  {isOnBreak ? (
                    <button onClick={() => punchOutMutation.mutate({})} disabled={punchOutMutation.isPending} className="w-full bg-emerald-600 text-white py-5 rounded-2xl font-black uppercase tracking-widest text-[10px] hover:bg-emerald-700 hover:scale-[1.02] active:scale-95 transition-all shadow-lg shadow-emerald-100">
                      {t("time_tracking.actions.return_to_work")}
                    </button>
                  ) : (
                    <>
                      <button onClick={() => punchInMutation.mutate({ employee_id: selectedEmployeeId, entry_type: "break" })} disabled={punchInMutation.isPending} className="w-full bg-amber-500 text-white py-5 rounded-2xl font-black uppercase tracking-widest text-[10px] hover:bg-amber-600 hover:scale-[1.02] active:scale-95 transition-all shadow-lg shadow-amber-100">
                        {t("time_tracking.actions.start_break")}
                      </button>
                      <button onClick={() => punchOutMutation.mutate({})} disabled={punchOutMutation.isPending} className="w-full bg-rose-600 text-white py-5 rounded-2xl font-black uppercase tracking-widest text-[10px] hover:bg-rose-700 hover:scale-[1.02] active:scale-95 transition-all shadow-lg shadow-rose-100">
                        {t("time_tracking.actions.punch_out")}
                      </button>
                    </>
                  )}
                </div>
              </div>
            ) : (
              <button onClick={() => punchInMutation.mutate({ employee_id: selectedEmployeeId, entry_type: "work" })} disabled={punchInMutation.isPending} className="w-full bg-primary-600 text-white py-6 rounded-[1.5rem] font-black uppercase tracking-widest text-xs hover:bg-primary-700 hover:scale-[1.02] active:scale-95 transition-all shadow-2xl shadow-primary-100 relative z-10">
                {t("time_tracking.actions.start_work")}
              </button>
            )}

            <button onClick={() => { setAdjustmentData({ ...adjustmentData, timeEntryId: undefined }); setIsAdjustmentModalOpen(true); }} className="mt-8 text-primary-600 text-[10px] font-black uppercase tracking-widest hover:text-primary-800 transition-colors py-2">
              {t("time_tracking.forgot_punch")}
            </button>
          </div>

          {/* History */}
          <div className="lg:col-span-8 bg-white border border-slate-200 rounded-[40px] overflow-hidden shadow-sm flex flex-col">
            <div className="px-8 py-6 border-b border-slate-50 bg-slate-50/30 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <History className="w-5 h-5 text-primary-500" />
                <h3 className="font-black text-slate-800 uppercase tracking-widest text-xs">{t("time_tracking.recent_history")}</h3>
              </div>
            </div>
            <div className="flex-1 overflow-y-auto max-h-[600px]">
              <table className="w-full text-left border-collapse">
                <thead className="sticky top-0 bg-white/80 backdrop-blur-md z-10 border-b border-slate-100">
                  <tr>
                    <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.date")}</th>
                    <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.in")}</th>
                    <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.out")}</th>
                    <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right"></th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-50">
                  {isLoadingHistory ? (
                    <tr><td colSpan={5} className="px-8 py-12 text-center text-slate-400 font-bold italic">{t("common.loading")}</td></tr>
                  ) : history.length === 0 ? (
                    <tr><td colSpan={5} className="px-10 py-20 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("time_tracking.messages.no_data")}</td></tr>
                  ) : history.map(entry => (
                    <tr key={entry.id} className="hover:bg-slate-50/50 transition-all group">
                      <td className="px-8 py-5">
                        <div className="flex items-center gap-3">
                           <span className="text-sm font-black text-slate-900">{formatDate(entry.punch_in_time)}</span>
                           <span className={cn("px-2 py-0.5 rounded-lg text-[8px] font-black uppercase tracking-tighter border", entry.entry_type === 'break' ? 'bg-amber-50 text-amber-600 border-amber-100' : 'bg-primary-50 text-primary-600 border-primary-100')}>
                             {entry.entry_type === 'break' ? t("common.break") : t("common.work")}
                           </span>
                        </div>
                      </td>
                      <td className="px-8 py-5 text-xs font-black text-slate-500 font-mono">{formatTime(entry.punch_in_time)}</td>
                      <td className="px-8 py-5">
                        {entry.punch_out_time ? (
                          <span className="text-xs font-black text-slate-500 font-mono">{formatTime(entry.punch_out_time)}</span>
                        ) : (
                          <span className={cn("font-black px-2 py-1 rounded-lg text-[9px] uppercase tracking-tighter animate-pulse", entry.entry_type === 'break' ? 'text-amber-600 bg-amber-50' : 'text-primary-600 bg-primary-50')}>
                            {entry.entry_type === 'break' ? t("time_tracking.status.on_break") : t("time_tracking.status.working")}
                          </span>
                        )}
                      </td>
                      <td className="px-8 py-5 text-right">
                        <button 
                          onClick={() => {
                            setAdjustmentData({
                              date: format(new Date(entry.punch_in_time), 'yyyy-MM-dd'),
                              punchIn: formatTime24h(entry.punch_in_time),
                              punchOut: entry.punch_out_time ? formatTime24h(entry.punch_out_time) : "17:00",
                              reason: "",
                              isBreakOvertime: entry.is_break_overtime,
                              timeEntryId: entry.id
                            });
                            setIsAdjustmentModalOpen(true);
                          }}
                          className="text-primary-600 bg-white shadow-sm border border-slate-100 hover:bg-primary-50 px-4 py-1.5 rounded-xl transition-all font-black text-[10px] uppercase tracking-widest active:scale-90 opacity-0 group-hover:opacity-100"
                        >
                          {t("common.adjust")}
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center py-32 text-slate-400 border-2 border-dashed border-slate-200 rounded-[40px] bg-slate-50/30">
          <div className="bg-white p-6 rounded-full shadow-sm mb-6"><AlertCircle className="w-12 h-12 text-slate-200" /></div>
          <p className="font-black text-xs uppercase tracking-widest text-slate-300">{t("time_tracking.empty_state_desc")}</p>
        </div>
      )}

      {/* Adjustment Modal stays same but could use some visual polish in labels if needed... */}


      {/* Adjustment Modal */}
      <Modal
        isOpen={isAdjustmentModalOpen}
        onClose={() => setIsAdjustmentModalOpen(false)}
        title={adjustmentData.timeEntryId ? t("time_tracking.modal.edit_title") : t("time_tracking.modal.new_title")}
        subtitle={t("time_tracking.modal.subtitle")}
        icon={<History size={24} />}
        maxWidth="md"
      >
        <form onSubmit={handleAdjustSubmit} className="space-y-8 pt-4">
          <div className="space-y-2">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.work_day")}</label>
            <input 
              type="date" 
              required
              className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight"
              value={adjustmentData.date}
              onChange={e => setAdjustmentData({...adjustmentData, date: e.target.value})}
            />
          </div>

          <div className="grid grid-cols-2 gap-6">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.punch_in")}</label>
              <input 
                type="time" 
                required
                className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-mono font-black text-sm text-center"
                value={adjustmentData.punchIn}
                onChange={e => setAdjustmentData({...adjustmentData, punchIn: e.target.value})}
              />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.punch_out")}</label>
              <input 
                type="time" 
                required
                className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-mono font-black text-sm text-center"
                value={adjustmentData.punchOut}
                onChange={e => setAdjustmentData({...adjustmentData, punchOut: e.target.value})}
              />
            </div>
          </div>

          <div className="flex items-center gap-4 py-2 bg-slate-50/50 p-4 rounded-[1.5rem] border border-slate-100 shadow-inner">
            <input 
              type="checkbox" 
              id="is_break_overtime"
              className="w-6 h-6 text-primary-600 rounded-xl border-slate-200 focus:ring-primary-500 transition-all cursor-pointer"
              checked={adjustmentData.isBreakOvertime}
              onChange={e => setAdjustmentData({...adjustmentData, isBreakOvertime: e.target.checked})}
            />
            <label htmlFor="is_break_overtime" className="text-xs font-black text-slate-700 uppercase tracking-widest cursor-pointer leading-tight">{t("attendance_confirm.ot_label")}</label>
          </div>

          <div className="space-y-2">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("time_tracking.modal.reason_label")}</label>
            <textarea 
              required
              placeholder={t("time_tracking.modal.reason_placeholder")}
              className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-medium text-sm h-28 resize-none"
              value={adjustmentData.reason}
              onChange={e => setAdjustmentData({...adjustmentData, reason: e.target.value})}
            />
          </div>

          <div className="pt-6 grid grid-cols-2 gap-5">
            <button type="button" onClick={() => setIsAdjustmentModalOpen(false)} className="px-8 py-4 border border-slate-100 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all">{t("common.cancel")}</button>
            <button type="submit" disabled={adjustMutation.isPending} className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all shadow-xl shadow-slate-200 active:scale-95 disabled:opacity-50">
              {adjustMutation.isPending ? t("common.sending").toUpperCase() : t("common.submit_request")}
            </button>
          </div>
        </form>
      </Modal>
    </div>
  );
}