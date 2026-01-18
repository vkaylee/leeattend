import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Loader2, Plus, Minus, History, Calendar, MessageSquare, Info } from "lucide-react";
import { useMutation, useQueryClient, useQuery } from "@tanstack/react-query";
import { adjustLeaveBalance, fetchLeaveHistory } from "@/lib/api";
import { useToast } from "../Toast";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import Modal from "../ui/Modal";
import { format } from "date-fns";

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

  // Fetch History
  const { data: history = [], isLoading } = useQuery({
    queryKey: ["leave-history", wsId, employeeId],
    queryFn: () => fetchLeaveHistory(wsId, employeeId),
    enabled: isOpen,
  });

  // Adjust Mutation
  const adjustMutation = useMutation({
    mutationFn: (data: { leave_type: string; amount: number; reason: string; transaction_date: string }) =>
      adjustLeaveBalance(wsId, employeeId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-history", wsId, employeeId] });
      queryClient.invalidateQueries({ queryKey: ["leave-balances", wsId, employeeId] });
      toast.success(t("leave_balances.history.messages.adjust_success"));
      setActiveTab("history");
    },
    onError: (error: any) => {
      toast.error(t("leave_balances.history.messages.adjust_error") + ": " + (error.message || "Unknown error"));
    }
  });

  const [adjustForm, setAdjustForm] = useState({
    type: "add", // add or deduct
    leave_type: "vacation",
    amount: "",
    reason: "",
    date: format(new Date(), 'yyyy-MM-dd')
  });

  const handleAdjustSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const amount = parseFloat(adjustForm.amount);
    if (isNaN(amount) || amount <= 0) {
      toast.error(t("leave_balances.history.validation.invalid_amount"));
      return;
    }

    const finalAmount = adjustForm.type === "add" ? amount : -amount;
    
    adjustMutation.mutate({
      leave_type: adjustForm.leave_type,
      amount: finalAmount,
      reason: adjustForm.reason,
      transaction_date: adjustForm.date
    });
  };

  const getTransactionColor = (type: string) => {
    switch (type) {
      case "accrual": return "bg-green-100 text-green-700";
      case "usage": return "bg-red-100 text-red-700";
      case "adjustment": return "bg-amber-100 text-amber-700";
      case "carry_over": return "bg-blue-100 text-blue-700";
      default: return "bg-slate-100 text-slate-700";
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={`${t("leave_balances.history.title")}: ${employeeName}`}
      subtitle={t("leave_balances.history.subtitle")}
      icon={<History className="w-6 h-6 text-primary-600" />}
      maxWidth="2xl"
    >
      <div className="space-y-6">
        {/* Tabs */}
        <div className="flex gap-2 p-1 bg-slate-100 w-full rounded-2xl">
          <button
            onClick={() => setActiveTab("history")}
            className={`flex-1 py-2 rounded-xl font-bold text-sm transition-all ${activeTab === "history" ? 'bg-white text-primary-600 shadow-sm' : 'text-slate-500 hover:text-slate-700'}`}
          >
            {t("leave_balances.history.tab_history")}
          </button>
          <button
            onClick={() => setActiveTab("adjust")}
            className={`flex-1 py-2 rounded-xl font-bold text-sm transition-all ${activeTab === "adjust" ? 'bg-white text-primary-600 shadow-sm' : 'text-slate-500 hover:text-slate-700'}`}
          >
            {t("leave_balances.history.tab_adjust")}
          </button>
        </div>

        {activeTab === "history" ? (
          <div className="space-y-4">
            {isLoading ? (
              <div className="flex flex-col items-center justify-center p-12 gap-3">
                <Loader2 className="animate-spin w-8 h-8 text-primary-500" />
                <p className="text-slate-400 font-medium italic">{t("common.loading")}</p>
              </div>
            ) : history.length > 0 ? (
              <div className="bg-white border border-slate-100 rounded-2xl overflow-hidden shadow-sm">
                <table className="w-full text-left">
                  <thead>
                    <tr className="bg-slate-50 border-b border-slate-100">
                      <th className="px-4 py-3 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.date")}</th>
                      <th className="px-4 py-3 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.type")}</th>
                      <th className="px-4 py-3 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.total")}</th>
                      <th className="px-4 py-3 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.reason")}</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-50">
                    {history.map((item) => (
                      <tr key={item.id} className="hover:bg-slate-50/50 transition-colors">
                        <td className="px-4 py-3 text-sm text-slate-600 font-medium whitespace-nowrap">
                          {formatDate(item.date)}
                        </td>
                        <td className="px-4 py-3">
                          <span className={`px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider ${getTransactionColor(item.transaction_type)}`}>
                            {item.transaction_type}
                          </span>
                        </td>
                        <td className={`px-4 py-3 text-right font-black text-sm ${item.amount > 0 ? "text-emerald-600" : "text-rose-600"}`}>
                          {item.amount > 0 ? "+" : ""}{item.amount}
                        </td>
                        <td className="px-4 py-3 text-xs text-slate-500 max-w-[180px] truncate" title={item.reason || ""}>
                          {item.reason || "-"}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <div className="text-center py-12 bg-slate-50 rounded-2xl border border-dashed border-slate-200">
                <Info className="w-8 h-8 text-slate-300 mx-auto mb-2" />
                <p className="text-slate-400 font-medium italic">{t("leave_balances.history.no_history")}</p>
              </div>
            )}
          </div>
        ) : (
          <form onSubmit={handleAdjustSubmit} className="space-y-6 animate-in fade-in slide-in-from-top-2 duration-300">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("leave_balances.history.action_label")}</label>
                <div className="grid grid-cols-2 gap-2 p-1 bg-slate-100 rounded-xl">
                  <button
                    type="button"
                    onClick={() => setAdjustForm({ ...adjustForm, type: "add" })}
                    className={`flex items-center justify-center gap-2 py-2 rounded-lg text-xs font-bold transition-all ${adjustForm.type === "add" ? "bg-white text-emerald-600 shadow-sm" : "text-slate-500 hover:text-slate-700"}`}
                  >
                    <Plus className="w-3 h-3" /> {t("leave_balances.history.add")}
                  </button>
                  <button
                    type="button"
                    onClick={() => setAdjustForm({ ...adjustForm, type: "deduct" })}
                    className={`flex items-center justify-center gap-2 py-2 rounded-lg text-xs font-bold transition-all ${adjustForm.type === "deduct" ? "bg-white text-rose-600 shadow-sm" : "text-slate-500 hover:text-slate-700"}`}
                  >
                    <Minus className="w-3 h-3" /> {t("leave_balances.history.deduct")}
                  </button>
                </div>
              </div>
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("leave_requests.policy.table.type")}</label>
                <select 
                  value={adjustForm.leave_type} 
                  onChange={(e) => setAdjustForm({...adjustForm, leave_type: e.target.value})}
                  className="w-full px-4 py-2.5 bg-slate-50 border border-slate-200 rounded-xl outline-none focus:ring-2 focus:ring-primary-500 font-bold text-sm transition-all"
                >
                  <option value="vacation">{t("leave_requests.types.vacation_full")}</option>
                  <option value="sick">{t("leave_requests.types.sick_full")}</option>
                </select>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("leave_balances.history.amount_placeholder").split('.')[0]}</label>
                <input 
                  type="number" 
                  step="0.5" 
                  min="0.5"
                  required
                  value={adjustForm.amount} 
                  onChange={(e) => setAdjustForm({...adjustForm, amount: e.target.value})} 
                  placeholder={t("leave_balances.history.amount_placeholder")}
                  className="w-full px-4 py-2.5 bg-slate-50 border border-slate-200 rounded-xl outline-none focus:ring-2 focus:ring-primary-500 font-bold transition-all"
                />
              </div>
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.date")}</label>
                <div className="relative">
                  <input 
                    type="date" 
                    required
                    value={adjustForm.date} 
                    onChange={(e) => setAdjustForm({...adjustForm, date: e.target.value})} 
                    className="w-full px-4 py-2.5 bg-slate-50 border border-slate-200 rounded-xl outline-none focus:ring-2 focus:ring-primary-500 font-bold transition-all"
                  />
                  <Calendar className="absolute right-3 top-2.5 w-4 h-4 text-slate-300 pointer-events-none" />
                </div>
              </div>
            </div>

            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("leave_balances.history.reason_label")}</label>
              <div className="relative">
                <textarea 
                  required
                  rows={3}
                  placeholder={t("leave_balances.history.reason_placeholder")}
                  value={adjustForm.reason}
                  onChange={(e) => setAdjustForm({...adjustForm, reason: e.target.value})}
                  className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-xl outline-none focus:ring-2 focus:ring-primary-500 font-medium transition-all resize-none"
                />
                <MessageSquare className="absolute right-3 top-3 w-4 h-4 text-slate-200" />
              </div>
            </div>

            <div className="flex gap-3 pt-4">
              <button 
                type="button" 
                onClick={onClose}
                className="flex-1 px-6 py-3 border border-slate-200 rounded-xl font-bold text-slate-500 hover:bg-slate-50 transition-all"
              >
                {t("common.cancel")}
              </button>
              <button 
                type="submit" 
                disabled={adjustMutation.isPending}
                className="flex-1 px-6 py-3 bg-primary-900 text-white rounded-xl font-bold hover:bg-black transition-all shadow-lg shadow-slate-200 flex items-center justify-center gap-2 disabled:opacity-50"
              >
                {adjustMutation.isPending ? <Loader2 className="animate-spin w-4 h-4" /> : <Shield className="w-4 h-4" />}
                {t("leave_balances.history.confirm_btn")}
              </button>
            </div>
          </form>
        )}
      </div>
    </Modal>
  );
}

function Shield({ className }: { className?: string }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={className}>
      <path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z" />
    </svg>
  );
}