import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Loader2, Plus, Minus, History, Calendar, MessageSquare, Info } from "lucide-react";
import { useMutation, useQueryClient, useQuery } from "@tanstack/react-query";
import { adjustLeaveBalance, fetchLeaveHistory, fetchLeaveTypes } from "@/lib/api";
import { useToast } from "../Toast";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import Modal from "../ui/Modal";
import { format } from "date-fns";
import { cn } from "@/lib/utils";

interface LeaveHistoryModalProps {
  wsId: string;
  employeeId: string;
  employeeName: string;
  isOpen: boolean;
  onClose: () => void;
}

export function LeaveHistoryModal({ wsId, employeeId, employeeName, isOpen, onClose }: LeaveHistoryModalProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const { formatDate } = useDateTimeFormatter();
  const [activeTab, setActiveTab] = useState("history");

  const { data: history = [], isLoading } = useQuery({ queryKey: ["leave-history", wsId, employeeId], queryFn: () => fetchLeaveHistory(wsId, employeeId), enabled: isOpen });
  const { data: leaveTypes = [] } = useQuery({ queryKey: ["leave-types", wsId], queryFn: () => fetchLeaveTypes(wsId), enabled: isOpen });

  const adjustMutation = useMutation({
    mutationFn: (data: { leave_type_id: string; amount: number; reason: string; transaction_date: string }) => adjustLeaveBalance(wsId, employeeId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-history"] });
      queryClient.invalidateQueries({ queryKey: ["leave-balances"] });
      toast.success(t("leave_balances.history.messages.adjust_success")); setActiveTab("history");
    },
  });

  const [adjustForm, setAdjustForm] = useState({ type: "add", leave_type_id: "", amount: "", reason: "", date: format(new Date(), 'yyyy-MM-dd') });

  const handleAdjustSubmit = (e: React.FormEvent) => {
    e.preventDefault(); const amount = parseFloat(adjustForm.amount);
    if (isNaN(amount) || amount <= 0) return;
    const typeId = adjustForm.leave_type_id || leaveTypes.find(t => t.code === "vacation")?.id;
    if (!typeId) { toast.error("Please select a leave type"); return; }
    adjustMutation.mutate({ leave_type_id: typeId, amount: adjustForm.type === "add" ? amount : -amount, reason: adjustForm.reason, transaction_date: adjustForm.date });
  };

  const getTransactionColor = (type: string) => {
    switch (type) {
      case "accrual": return "bg-emerald-50 text-emerald-600 border-emerald-100";
      case "usage": return "bg-rose-50 text-rose-600 border-rose-100";
      case "adjustment": return "bg-amber-50 text-amber-600 border-amber-100";
      case "carry_over": return "bg-blue-50 text-blue-600 border-blue-100";
      default: return "bg-slate-50 text-slate-500 border-slate-100";
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={`${t("leave_balances.history.title")}: ${employeeName}`} subtitle={t("leave_balances.history.subtitle")} icon={<History size={24} />} maxWidth="2xl">
      <div className="space-y-8 pt-4 animate-in fade-in duration-500">
        <div className="flex bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50 shadow-inner">
          <button onClick={() => setActiveTab("history")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'history' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}>{t("leave_balances.history.tab_history")}</button>
          <button onClick={() => setActiveTab("adjust")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'adjust' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}>{t("leave_balances.history.tab_adjust")}</button>
        </div>

        {activeTab === "history" ? (
          <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
            {isLoading ? (
              <div className="p-20 text-center"><Loader2 className="animate-spin w-10 h-10 text-primary-600 mx-auto" /></div>
            ) : history.length > 0 ? (
              <div className="bg-white border border-slate-100 rounded-[2rem] overflow-hidden shadow-sm">
                <table className="w-full text-left border-collapse">
                  <thead>
                    <tr className="bg-slate-50/50 border-b border-slate-100">
                      <th className="px-6 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t("common.date")}</th>
                      <th className="px-6 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest text-center">{t("common.type")}</th>
                      <th className="px-6 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest text-right">{t("common.total")}</th>
                      <th className="px-6 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t("common.reason")}</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-50">
                    {history.map((item) => (
                      <tr key={item.id} className="hover:bg-slate-50/50 transition-colors group">
                        <td className="px-6 py-5 text-xs font-bold text-slate-500 font-mono uppercase">{formatDate(item.date)}</td>
                        <td className="px-6 py-5 text-center"><span className={cn("px-2.5 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border", getTransactionColor(item.transaction_type))}>{item.transaction_type}</span></td>
                        <td className={cn("px-6 py-5 text-right font-bold text-sm font-mono tracking-tighter", item.amount > 0 ? "text-emerald-600" : "text-rose-600")}>{item.amount > 0 ? "+" : ""}{item.amount}</td>
                        <td className="px-6 py-5 text-[10px] font-bold text-slate-500 uppercase tracking-tighter max-w-[200px] truncate" title={item.reason || ""}>{item.reason || "-"}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <div className="py-20 text-center border-2 border-dashed border-slate-100 rounded-[2.5rem] bg-slate-50/30 flex flex-col items-center"><Info size={32} className="text-slate-200 mb-4" /><p className="text-[10px] font-bold text-slate-300 uppercase tracking-widest">{t("leave_balances.history.no_history")}</p></div>
            )}
          </div>
        ) : (
          <form onSubmit={handleAdjustSubmit} className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500 pt-2">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
              <div className="space-y-3">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("leave_balances.history.action_label")}</label>
                <div className="grid grid-cols-2 gap-3 p-1.5 bg-slate-100/50 rounded-2xl border border-slate-200/50 shadow-inner">
                  <button type="button" onClick={() => setAdjustForm({ ...adjustForm, type: "add" })} className={cn("flex items-center justify-center gap-2 py-3 rounded-xl text-xs font-bold uppercase tracking-widest transition-all", adjustForm.type === "add" ? "bg-white text-emerald-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-600")}><Plus size={14} /> {t("leave_balances.history.add")}</button>
                  <button type="button" onClick={() => setAdjustForm({ ...adjustForm, type: "deduct" })} className={cn("flex items-center justify-center gap-2 py-3 rounded-xl text-xs font-bold uppercase tracking-widest transition-all", adjustForm.type === "deduct" ? "bg-white text-rose-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-600")}><Minus size={14} /> {t("leave_balances.history.deduct")}</button>
                </div>
              </div>
              <div className="space-y-3">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("leave_requests.policy.table.type")}</label>
                <select value={adjustForm.leave_type_id} onChange={(e) => setAdjustForm({...adjustForm, leave_type_id: e.target.value})} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-bold text-sm uppercase tracking-tight shadow-sm">
                  <option value="">{t("common.select")}</option>
                  {leaveTypes.map(lt => <option key={lt.id} value={lt.id}>{lt.name}</option>)}
                </select>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
              <div className="space-y-3">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("leave_balances.history.amount_placeholder").split('.')[0]}</label>
                <input type="number" step="0.5" min="0.5" required value={adjustForm.amount} onChange={(e) => setAdjustForm({...adjustForm, amount: e.target.value})} placeholder={t("leave_balances.history.amount_placeholder")} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-bold text-sm font-mono" />
              </div>
              <div className="space-y-3">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("common.date")}</label>
                <div className="relative group"><input type="date" required value={adjustForm.date} onChange={(e) => setAdjustForm({...adjustForm, date: e.target.value})} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-bold text-sm" /><Calendar className="absolute right-4 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-300 group-focus-within:text-primary-500 transition-colors" /></div>
              </div>
            </div>

            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("leave_balances.history.reason_label")}</label>
              <div className="relative group"><textarea required rows={3} placeholder={t("leave_balances.history.reason_placeholder")} value={adjustForm.reason} onChange={(e) => setAdjustForm({...adjustForm, reason: e.target.value})} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none font-medium text-sm transition-all resize-none shadow-sm" /><MessageSquare className="absolute right-4 top-4 w-4 h-4 text-slate-200 group-focus-within:text-primary-500 transition-colors" /></div>
            </div>

            <div className="flex gap-4 pt-4 border-t border-slate-50">
              <button type="button" onClick={onClose} className="flex-1 px-8 py-4 bg-white border border-slate-200 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all">{t("common.cancel")}</button>
              <button type="submit" disabled={adjustMutation.isPending} className="flex-1 bg-primary-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 active:scale-95 disabled:opacity-50 flex items-center justify-center gap-3">{adjustMutation.isPending ? <Loader2 className="animate-spin w-5 h-5" /> : <Shield size={18} />} {t("leave_balances.history.confirm_btn")}</button>
            </div>
          </form>
        )}
      </div>
    </Modal>
  );
}

function Shield({ className, size = 24 }: { className?: string; size?: number }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={className}>
      <path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z" />
    </svg>
  );
}