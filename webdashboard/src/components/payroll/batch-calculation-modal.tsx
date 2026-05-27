import { useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Calculator, 
  CheckCircle2, 
  AlertCircle, 
  Loader2, 
  User,
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { MonthYearPicker } from "@/components/common/month-year-picker";
import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { calculateBatchPayslips } from "@/lib/api";
import { BatchCalculateResult } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface BatchCalculationModalProps {
  employeeIds: string[];
  employeeNames: Record<string, string>; // ID -> Name map
  isOpen: boolean;
  onClose: () => void;
  workspaceId: string;
}

function BatchCalculationLoading({ employeeCount }: { employeeCount: number }) {
  const { t } = useTranslation();
  return (
    <div className="py-20 flex flex-col items-center justify-center text-center animate-in zoom-in-95 duration-500">
      <div className="w-24 h-24 bg-slate-50 dark:bg-slate-800 rounded-full flex items-center justify-center mb-8 relative shadow-inner">
        <Loader2 className="w-12 h-12 text-primary-600 dark:text-primary-400 animate-spin" />
        <div className="absolute inset-0 border-4 border-primary-100 dark:border-primary-800 border-t-transparent rounded-full opacity-50" />
      </div>
      <Typography className="text-2xl font-bold text-slate-700 dark:text-slate-200 mb-2 uppercase tracking-tight" variant="h3">{t("common.processing")}</Typography>
      <Typography className="text-slate-500 dark:text-slate-400 text-sm font-medium max-w-xs leading-relaxed" variant="p">
        Calculating payroll for {employeeCount} employees. This may take a moment...
      </Typography>
    </div>
  );
}

function BatchCalculationResults({
  results,
  employeeNames,
  resetAndClose
}: {
  results: BatchCalculateResult[];
  employeeNames: Record<string, string>;
  resetAndClose: () => void;
}) {
  const { t } = useTranslation();
  const successCount = results.filter(r => r.status === "success").length;
  const errorCount = results.length - successCount;

  return (
    <div className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="grid grid-cols-2 gap-6">
        <div className="bg-emerald-50 dark:bg-emerald-950/50 p-6 rounded-[2rem] border border-emerald-100 dark:border-emerald-800 shadow-inner">
          <Typography className="text-[10px] font-bold text-emerald-600 dark:text-emerald-300 uppercase tracking-widest mb-1 opacity-70" variant="p">Success</Typography>
          <Typography className="text-3xl font-bold text-emerald-700 dark:text-emerald-300 font-mono tracking-tighter" variant="p">{successCount}</Typography>
        </div>
        <div className="bg-rose-50 dark:bg-rose-950/50 p-6 rounded-[2rem] border border-rose-100 dark:border-rose-800 shadow-inner">
          <Typography className="text-[10px] font-bold text-rose-600 dark:text-rose-300 uppercase tracking-widest mb-1 opacity-70" variant="p">Errors</Typography>
          <Typography className="text-3xl font-bold text-rose-700 dark:text-rose-300 font-mono tracking-tighter" variant="p">{errorCount}</Typography>
        </div>
      </div>

      <div className="max-h-72 overflow-y-auto space-y-3 pr-2 custom-scrollbar p-1">
        {results.map(res => (
          <div className="flex items-center justify-between p-4 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl hover:shadow-md transition-all group" key={res.employee_id}>
            <div className="flex items-center gap-4">
              <div className={cn("w-10 h-10 rounded-xl flex items-center justify-center shadow-inner border", res.status === 'success' ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" : "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800")}>
                {res.status === 'success' ? <CheckCircle2 size={18} /> : <AlertCircle size={18} />}
              </div>
              <div>
                <Typography className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase" variant="p">{employeeNames[res.employee_id] || "—"}</Typography>
                {res.error_message ? <Typography className="text-[10px] text-rose-500 font-bold uppercase tracking-widest mt-0.5 opacity-80" variant="p">{res.error_message}</Typography> : null}
              </div>
            </div>
            {res.status === 'success' && res.payslip ? (
              <span className="text-sm font-bold text-primary-600 dark:text-primary-400 font-mono tracking-tighter">
                {new Intl.NumberFormat('vi-VN').format(parseFloat(res.payslip.net_pay_amount))}
              </span>
            ) : null}
          </div>
        ))}
      </div>

      <div className="pt-4 border-t border-slate-50 dark:border-slate-800">
        <Button className="w-full py-4 bg-slate-900 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 dark:shadow-black/30 active:scale-95" onClick={resetAndClose}>{t("common.close")}</Button>
      </div>
    </div>
  );
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
      void queryClient.invalidateQueries({ queryKey: ["payslips"] });
      toast.success(t("payroll.messages.calculate_success"));
    },
  });

  const handleCalculate = () => { batchMutation.mutate({ year: calcYear, month: calcMonth }); };
  const resetAndClose = () => { setResults(null); batchMutation.reset(); onClose(); };

  return (
    <Modal 
      icon={<Calculator size={24} />} 
      isOpen={isOpen} 
      maxWidth="lg" 
      onClose={batchMutation.isPending ? () => { /* empty */ } : resetAndClose} 
      subtitle={t("payroll.modal.subtitle")} 
      testId="batch-calc-modal"
      title={t("payroll.modal.title")}
    >
      <div className="space-y-8 pt-4 animate-in fade-in duration-500">
        {!results && !batchMutation.isPending && (
          <div className="space-y-8 animate-in fade-in duration-500">
            <div className="bg-primary-50 dark:bg-primary-950/50 p-8 rounded-[2.5rem] border border-primary-100 dark:border-primary-800/50 flex items-center gap-6 shadow-inner">
              <div className="w-16 h-16 bg-white dark:bg-slate-900 rounded-3xl flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800">
                <User size={32} />
              </div>
              <div>
                <Typography className="text-[10px] font-bold text-primary-400 uppercase tracking-[0.2em] mb-1 opacity-70" variant="p">{t("common.employees")}</Typography>
                <Typography className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tight uppercase" variant="p">
                  {employeeIds.length} {t("common.people")}
                </Typography>
              </div>
            </div>

            <div className="space-y-4">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("payroll.table.period")}</label>
              <div className="bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 p-2 rounded-[2rem] shadow-inner">
                <MonthYearPicker month={calcMonth} onChange={(m, y) => { setCalcMonth(m); setCalcYear(y); }} year={calcYear} />
              </div>
            </div>

            <div className="pt-4 grid grid-cols-2 gap-5">
              <Button className="px-8 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all" onClick={onClose} type="button">{t("common.cancel")}</Button>
              <Button 
                className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 dark:shadow-black/30 active:scale-95"
                data-testid="batch-calc-submit-btn" 
                onClick={handleCalculate}
              >
                {t("payroll.modal.submit")}
              </Button>
            </div>
          </div>
        )}

        {batchMutation.isPending ? (
          <BatchCalculationLoading employeeCount={employeeIds.length} />
        ) : null}

        {results ? (
          <BatchCalculationResults 
            employeeNames={employeeNames} 
            resetAndClose={resetAndClose} 
            results={results} 
          />
        ) : null}
      </div>
    </Modal>
  );
}
