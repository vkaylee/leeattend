import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { 
  Calculator, 
  CheckCircle2, 
  AlertCircle, 
  Loader2, 
  User,
} from "lucide-react";
import Modal from "@/components/ui/Modal";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { calculateBatchPayslips } from "@/lib/api";
import { BatchCalculateResult } from "@/lib/api/types";
import { useToast } from "@/components/ui/Toast";
import { MonthYearPicker } from "@/components/common/MonthYearPicker";
import { cn } from "@/lib/utils";

interface BatchCalculationModalProps {
  isOpen: boolean;
  onClose: () => void;
  workspaceId: string;
  employeeIds: string[];
  employeeNames: Record<string, string>; // ID -> Name map
}

export default function BatchCalculationModal({ 
  isOpen, 
  onClose, 
  workspaceId, 
  employeeIds,
  employeeNames
}: BatchCalculationModalProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  
  const [calcYear, setCalcYear] = useState(() => {
    const d = new Date(); d.setMonth(d.getMonth() - 1);
    return d.getFullYear();
  });
  const [calcMonth, setCalcMonth] = useState(() => {
    const d = new Date(); d.setMonth(d.getMonth() - 1);
    return d.getMonth() + 1;
  });
  
  const [results, setResults] = useState<BatchCalculateResult[] | null>(null);

  const batchMutation = useMutation({
    mutationFn: (data: { year: number; month: number }) => 
      calculateBatchPayslips(workspaceId, { employee_ids: employeeIds, year: data.year, month: data.month }),
    onSuccess: (data) => {
      setResults(data.results);
      queryClient.invalidateQueries({ queryKey: ["payslips"] });
      toast.success(t("payroll.messages.calculate_success"));
    },
  });

  const handleCalculate = () => { batchMutation.mutate({ year: calcYear, month: calcMonth }); };
  const resetAndClose = () => { setResults(null); batchMutation.reset(); onClose(); };

  const successCount = results?.filter(r => r.status === "success").length || 0;
  const errorCount = (results?.length || 0) - successCount;

  return (
    <Modal 
      isOpen={isOpen} 
      onClose={batchMutation.isPending ? () => {} : resetAndClose} 
      title={t("payroll.modal.title")} 
      subtitle={t("payroll.modal.subtitle")} 
      icon={<Calculator size={24} />} 
      maxWidth="lg"
      testId="batch-calc-modal"
    >
      <div className="space-y-8 pt-4 animate-in fade-in duration-500">
        {!results && !batchMutation.isPending && (
          <div className="space-y-8 animate-in fade-in duration-500">
            <div className="bg-primary-50 dark:bg-primary-950/50 p-8 rounded-[2.5rem] border border-primary-100 dark:border-primary-800/50 flex items-center gap-6 shadow-inner">
              <div className="w-16 h-16 bg-white dark:bg-slate-900 rounded-3xl flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800">
                <User size={32} />
              </div>
              <div>
                <p className="text-[10px] font-bold text-primary-400 uppercase tracking-[0.2em] mb-1 opacity-70">{t("common.employees")}</p>
                <p className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tight uppercase">
                  {employeeIds.length} {t("common.people")}
                </p>
              </div>
            </div>

            <div className="space-y-4">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("payroll.table.period")}</label>
              <div className="bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 p-2 rounded-[2rem] shadow-inner">
                <MonthYearPicker month={calcMonth} year={calcYear} onChange={(m, y) => { setCalcMonth(m); setCalcYear(y); }} />
              </div>
            </div>

            <div className="pt-4 grid grid-cols-2 gap-5">
              <button type="button" onClick={onClose} className="px-8 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all">{t("common.cancel")}</button>
              <button 
                data-testid="batch-calc-submit-btn"
                onClick={handleCalculate} 
                className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 dark:shadow-black/30 active:scale-95"
              >
                {t("payroll.modal.submit")}
              </button>
            </div>
          </div>
        )}

        {batchMutation.isPending && (
          <div className="py-20 flex flex-col items-center justify-center text-center animate-in zoom-in-95 duration-500">
            <div className="w-24 h-24 bg-slate-50 dark:bg-slate-800 rounded-full flex items-center justify-center mb-8 relative shadow-inner">
              <Loader2 className="w-12 h-12 text-primary-600 dark:text-primary-400 animate-spin" />
              <div className="absolute inset-0 border-4 border-primary-100 dark:border-primary-800 border-t-transparent rounded-full opacity-50" />
            </div>
            <h3 className="text-2xl font-bold text-slate-700 dark:text-slate-200 mb-2 uppercase tracking-tight">{t("common.processing")}</h3>
            <p className="text-slate-500 dark:text-slate-400 text-sm font-medium max-w-xs leading-relaxed">
              Calculating payroll for {employeeIds.length} employees. This may take a moment...
            </p>
          </div>
        )}

        {results && (
          <div className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div className="grid grid-cols-2 gap-6">
              <div className="bg-emerald-50 dark:bg-emerald-950/50 p-6 rounded-[2rem] border border-emerald-100 dark:border-emerald-800 shadow-inner">
                <p className="text-[10px] font-bold text-emerald-600 dark:text-emerald-300 uppercase tracking-widest mb-1 opacity-70">Success</p>
                <p className="text-3xl font-bold text-emerald-700 dark:text-emerald-300 font-mono tracking-tighter">{successCount}</p>
              </div>
              <div className="bg-rose-50 dark:bg-rose-950/50 p-6 rounded-[2rem] border border-rose-100 dark:border-rose-800 shadow-inner">
                <p className="text-[10px] font-bold text-rose-600 dark:text-rose-300 uppercase tracking-widest mb-1 opacity-70">Errors</p>
                <p className="text-3xl font-bold text-rose-700 dark:text-rose-300 font-mono tracking-tighter">{errorCount}</p>
              </div>
            </div>

            <div className="max-h-72 overflow-y-auto space-y-3 pr-2 custom-scrollbar p-1">
              {results.map(res => (
                <div key={res.employee_id} className="flex items-center justify-between p-4 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl hover:shadow-md transition-all group">
                  <div className="flex items-center gap-4">
                    <div className={cn("w-10 h-10 rounded-xl flex items-center justify-center shadow-inner border", res.status === 'success' ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" : "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800")}>
                      {res.status === 'success' ? <CheckCircle2 size={18} /> : <AlertCircle size={18} />}
                    </div>
                    <div>
                      <p className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase">{employeeNames[res.employee_id] || "—"}</p>
                      {res.error_message && <p className="text-[10px] text-rose-500 font-bold uppercase tracking-widest mt-0.5 opacity-80">{res.error_message}</p>}
                    </div>
                  </div>
                  {res.status === 'success' && res.payslip && (
                    <span className="text-sm font-bold text-primary-600 dark:text-primary-400 font-mono tracking-tighter">
                      {new Intl.NumberFormat('vi-VN').format(parseFloat(res.payslip.net_pay_amount))}
                    </span>
                  )}
                </div>
              ))}
            </div>

            <div className="pt-4 border-t border-slate-50 dark:border-slate-800">
              <button onClick={resetAndClose} className="w-full py-4 bg-slate-900 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 dark:shadow-black/30 active:scale-95">{t("common.close")}</button>
            </div>
          </div>
        )}
      </div>
    </Modal>
  );
}
