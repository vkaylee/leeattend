import { useMutation, useQueryClient, useQuery } from "@tanstack/react-query";
import { Loader2, Plus, Minus, History, Calendar, MessageSquare, Info } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { adjustLeaveBalance, fetchLeaveHistory, fetchLeaveTypes } from "@/lib/api";
import { cn } from "@/lib/utils";

import Modal from "../ui/modal";
import { useToast } from "../ui/toast";



interface LeaveHistoryModalProps {
  employeeId: string;
  employeeName: string;
  isOpen: boolean;
  leaveTypeId?: string;
  leaveTypeName?: string;
  onClose: () => void;
  wsId: string;
}

export function LeaveHistoryModal({ wsId, employeeId, employeeName, leaveTypeId, leaveTypeName, isOpen, onClose }: LeaveHistoryModalProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const { formatDate } = useDateTimeFormatter();
  const [activeTab, setActiveTab] = useState("history");

  const { data: history = [], isLoading } = useQuery({ queryKey: ["leave-history", wsId, employeeId, leaveTypeId], queryFn: () => fetchLeaveHistory(wsId, employeeId, leaveTypeId), enabled: isOpen });
  const { data: leaveTypes = [] } = useQuery({ queryKey: ["leave-types", wsId], queryFn: () => fetchLeaveTypes(wsId), enabled: isOpen });

  const adjustMutation = useMutation({
    mutationFn: (data: { leave_type_id: string; amount: number; reason: string; transaction_date: string }) => adjustLeaveBalance(wsId, employeeId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-history"] });
      queryClient.invalidateQueries({ queryKey: ["workspace-leave-balances"] });
      toast.success(t("leave_balances.history.messages.adjust_success")); setActiveTab("history");
    },
  });

  const [adjustForm, setAdjustForm] = useState({ type: "add", leave_type_id: leaveTypeId || "", amount: "", reason: "", date: format(new Date(), 'yyyy-MM-dd') });

  const handleAdjustSubmit = (e: React.FormEvent) => {
    e.preventDefault(); const amount = parseFloat(adjustForm.amount);
    if (isNaN(amount) || amount <= 0) return;
    const typeId = adjustForm.leave_type_id || leaveTypes.find(t => t.code === "vacation")?.id;
    if (!typeId) { toast.error("Please select a leave type"); return; }
    adjustMutation.mutate({ leave_type_id: typeId, amount: adjustForm.type === "add" ? amount : -amount, reason: adjustForm.reason, transaction_date: adjustForm.date });
  };

  const getTransactionColor = (type: string) => {
    switch (type) {
      case "accrual": return "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800";
      case "usage": return "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800";
      case "adjustment": return "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800";
      case "carry_over": return "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-100 dark:border-blue-800";
      default: return "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800";
    }
  };

  return (
    <Modal icon={<History size={24} />} isOpen={isOpen} maxWidth="2xl" onClose={onClose} subtitle={t("leave_balances.history.subtitle")} title={`${t("leave_balances.history.title")}: ${employeeName ?? ""}${leaveTypeName ? ` — ${leaveTypeName ?? ""}` : ''}`}>
      <div className="space-y-8 pt-4 animate-in fade-in duration-500">
        <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
          <button className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'history' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")} onClick={() => { setActiveTab("history"); }}>{t("leave_balances.history.tab_history")}</button>
          <button className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'adjust' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")} onClick={() => { setActiveTab("adjust"); }}>{t("leave_balances.history.tab_adjust")}</button>
        </div>

        {activeTab === "history" ? (
          <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
            {isLoading ? (
              <div className="p-20 text-center"><Loader2 className="animate-spin w-10 h-10 text-primary-600 dark:text-primary-400 mx-auto" /></div>
            ) : history.length > 0 ? (
              <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2rem] overflow-hidden shadow-sm">
                <table className="w-full text-left border-collapse">
                  <thead>
                    <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
                      <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.date")}</th>
                      <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-center">{t("common.type")}</th>
                      <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.total")}</th>
                      <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.reason")}</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
                    {history.map((item) => (
                      <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors group" key={item.id}>
                        <td className="px-6 py-5 text-xs font-bold text-slate-500 dark:text-slate-400 font-mono uppercase">{formatDate(item.date)}</td>
                        <td className="px-6 py-5 text-center"><span className={cn("px-2.5 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border", getTransactionColor(item.transaction_type))}>{item.transaction_type}</span></td>
                        <td className={cn("px-6 py-5 text-right font-bold text-sm font-mono tracking-tighter", item.amount > 0 ? "text-emerald-600 dark:text-emerald-300" : "text-rose-600 dark:text-rose-300")}>{item.amount > 0 ? "+" : ""}{item.amount}</td>
                        <td className="px-6 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-tighter max-w-[200px] truncate" title={item.reason || ""}>{item.reason || "-"}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <div className="py-20 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[2.5rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center"><Info className="text-slate-200 dark:text-slate-700 mb-4" size={32} /><p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">{t("leave_balances.history.no_history")}</p></div>
            )}
          </div>
        ) : (
          <form className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500 pt-2" onSubmit={handleAdjustSubmit}>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
              <div className="space-y-3">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("leave_balances.history.action_label")}</label>
                <div className="grid grid-cols-2 gap-3 p-1.5 bg-slate-100 dark:bg-slate-700/50 rounded-2xl border border-slate-200 dark:border-slate-700/50 shadow-inner">
                  <button className={cn("flex items-center justify-center gap-2 py-3 rounded-xl text-xs font-bold uppercase tracking-widest transition-all", adjustForm.type === "add" ? "bg-white dark:bg-slate-900 text-emerald-600 dark:text-emerald-300 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 dark:text-slate-600")} onClick={() => { setAdjustForm({ ...adjustForm, type: "add" }); }} type="button"><Plus size={14} /> {t("leave_balances.history.add")}</button>
                  <button className={cn("flex items-center justify-center gap-2 py-3 rounded-xl text-xs font-bold uppercase tracking-widest transition-all", adjustForm.type === "deduct" ? "bg-white dark:bg-slate-900 text-rose-600 dark:text-rose-300 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 dark:text-slate-600")} onClick={() => { setAdjustForm({ ...adjustForm, type: "deduct" }); }} type="button"><Minus size={14} /> {t("leave_balances.history.deduct")}</button>
                </div>
              </div>
              <div className="space-y-3">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("leave_requests.policy.table.type")}</label>
                <select className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm uppercase tracking-tight shadow-sm" onChange={(e) => { setAdjustForm({ ...adjustForm, leave_type_id: e.target.value }); }} value={adjustForm.leave_type_id}>
                  <option value="">{t("common.select")}</option>
                  {leaveTypes.map(lt => <option key={lt.id} value={lt.id}>{lt.name}</option>)}
                </select>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
              <div className="space-y-3">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("leave_balances.history.amount_placeholder").split('.')[0]}</label>
                <input className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm font-mono" min="0.5" onChange={(e) => { setAdjustForm({ ...adjustForm, amount: e.target.value }); }} placeholder={t("leave_balances.history.amount_placeholder")} required step="0.5" type="number" value={adjustForm.amount} />
              </div>
              <div className="space-y-3">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("common.date")}</label>
                <div className="relative group"><input className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm" onChange={(e) => { setAdjustForm({ ...adjustForm, date: e.target.value }); }} required type="date" value={adjustForm.date} /><Calendar className="absolute right-4 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" /></div>
              </div>
            </div>

            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("leave_balances.history.reason_label")}</label>
              <div className="relative group"><textarea className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none font-medium text-sm transition-all resize-none shadow-sm" onChange={(e) => { setAdjustForm({ ...adjustForm, reason: e.target.value }); }} placeholder={t("leave_balances.history.reason_placeholder")} required rows={3} value={adjustForm.reason} /><MessageSquare className="absolute right-4 top-4 w-4 h-4 text-slate-200 dark:text-slate-700 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" /></div>
            </div>

            <div className="flex gap-4 pt-4 border-t border-slate-50 dark:border-slate-800">
              <button className="flex-1 px-8 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all" onClick={onClose} type="button">{t("common.cancel")}</button>
              <button className="flex-1 bg-primary-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 dark:shadow-black/30 active:scale-95 disabled:opacity-50 flex items-center justify-center gap-3" disabled={adjustMutation.isPending} type="submit">{adjustMutation.isPending ? <Loader2 className="animate-spin w-5 h-5" /> : <Shield size={18} />} {t("leave_balances.history.confirm_btn")}</button>
            </div>
          </form>
        )}
      </div>
    </Modal>
  );
}

function Shield({ className, size = 24 }: { className?: string; size?: number }) {
  return (
    <svg className={className} fill="none" height={size} stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24" width={size} xmlns="http://www.w3.org/2000/svg">
      <path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z" />
    </svg>
  );
}