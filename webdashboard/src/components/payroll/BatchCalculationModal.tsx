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
import { calculateBatchPayslips } from "@/lib/api/payroll";
import { BatchCalculateResult } from "@/lib/api/types";
import { useToast } from "@/components/Toast";
import { MonthYearPicker } from "@/components/common/MonthYearPicker";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

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
  
  // Default to last month
  const [calcYear, setCalcYear] = useState(() => {
    const d = new Date();
    d.setMonth(d.getMonth() - 1);
    return d.getFullYear();
  });
  const [calcMonth, setCalcMonth] = useState(() => {
    const d = new Date();
    d.setMonth(d.getMonth() - 1);
    return d.getMonth() + 1;
  });
  
  const [results, setResults] = useState<BatchCalculateResult[] | null>(null);

  const batchMutation = useMutation({
    mutationFn: (data: { year: number; month: number }) => 
      calculateBatchPayslips(workspaceId, { 
        employee_ids: employeeIds, 
        year: data.year, 
        month: data.month 
      }),
    onSuccess: (data) => {
      setResults(data.results);
      queryClient.invalidateQueries({ queryKey: ["payslips"] });
      toast.success(t("payroll.messages.calculate_success"));
    },
    onError: (e: any) => toast.error(e.message, t("payroll.messages.calculate_error"))
  });

  const handleCalculate = () => {
    batchMutation.mutate({ year: calcYear, month: calcMonth });
  };

  const resetAndClose = () => {
    setResults(null);
    batchMutation.reset();
    onClose();
  };

  const successCount = results?.filter(r => r.status === "success").length || 0;
  const errorCount = (results?.length || 0) - successCount;

  return (
    <Modal
      isOpen={isOpen}
      onClose={batchMutation.isPending ? () => {} : resetAndClose}
      title={t("payroll.modal.title")}
      subtitle={t("payroll.modal.subtitle")}
      icon={<Calculator className="w-6 h-6" />}
    >
      <div className="space-y-6">
        {!results && !batchMutation.isPending && (
          <>
            <div className="bg-primary-50 p-6 rounded-3xl border border-primary-100 flex items-center gap-4">
              <div className="w-12 h-12 bg-white rounded-2xl flex items-center justify-center text-primary-600 shadow-sm border border-primary-100">
                <User className="w-7 h-7" />
              </div>
              <div>
                <p className="text-[10px] font-black text-primary-400 uppercase tracking-widest leading-none mb-1">
                  {t("common.employee")}
                </p>
                <p className="text-lg font-black text-slate-900 tracking-tight">
                  {employeeIds.length} {t("common.people")}
                </p>
              </div>
            </div>

            <div className="space-y-4">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Kỳ lương (Pay Period)</label>
              <div className="bg-slate-50 p-4 rounded-3xl border border-slate-100">
                <MonthYearPicker
                  month={calcMonth}
                  year={calcYear}
                  onChange={(m, y) => {
                    setCalcMonth(m);
                    setCalcYear(y);
                  }}
                />
              </div>
            </div>

            <div className="pt-4 grid grid-cols-2 gap-5">
              <button type="button" onClick={onClose} className="px-8 py-4 border border-slate-100 rounded-2xl font-bold text-slate-500 hover:bg-slate-50 transition-all">{t("common.cancel")}</button>
              <button 
                onClick={handleCalculate}
                className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black hover:bg-slate-800 transition-all shadow-xl shadow-slate-200 flex items-center justify-center gap-2"
              >
                {t("payroll.modal.submit")}
              </button>
            </div>
          </>
        )}

        {batchMutation.isPending && (
          <div className="py-12 flex flex-col items-center justify-center text-center">
            <div className="w-20 h-20 bg-slate-50 rounded-full flex items-center justify-center mb-6 relative">
              <Loader2 className="w-10 h-10 text-primary-600 animate-spin" />
              <div className="absolute inset-0 border-4 border-primary-100 border-t-transparent rounded-full" />
            </div>
            <h3 className="text-xl font-bold text-slate-900 mb-2">
              {t("common.processing")}
            </h3>
            <p className="text-slate-500 text-sm max-w-xs">
              Calculating payroll for {employeeIds.length} employees. This may take a moment...
            </p>
          </div>
        )}

        {results && (
          <div className="space-y-6">
            <div className="grid grid-cols-2 gap-4">
              <div className="bg-emerald-50 p-4 rounded-2xl border border-emerald-100">
                <p className="text-[10px] font-black text-emerald-600 uppercase tracking-widest mb-1">Success</p>
                <p className="text-2xl font-black text-emerald-700">{successCount}</p>
              </div>
              <div className="bg-rose-50 p-4 rounded-2xl border border-rose-100">
                <p className="text-[10px] font-black text-rose-600 uppercase tracking-widest mb-1">Errors</p>
                <p className="text-2xl font-black text-rose-700">{errorCount}</p>
              </div>
            </div>

            <div className="max-h-60 overflow-y-auto space-y-2 pr-2 custom-scrollbar">
              {results.map(res => (
                <div key={res.employee_id} className="flex items-center justify-between p-3 bg-slate-50 rounded-xl border border-slate-100">
                  <div className="flex items-center gap-3">
                    <div className={cn(
                      "w-8 h-8 rounded-lg flex items-center justify-center",
                      res.status === 'success' ? "bg-emerald-100 text-emerald-600" : "bg-rose-100 text-rose-600"
                    )}>
                      {res.status === 'success' ? <CheckCircle2 size={16} /> : <AlertCircle size={16} />}
                    </div>
                    <div>
                      <p className="text-xs font-bold text-slate-900">{employeeNames[res.employee_id] || res.employee_id}</p>
                      {res.error_message && (
                        <p className="text-[10px] text-rose-500 truncate max-w-[200px]">{res.error_message}</p>
                      )}
                    </div>
                  </div>
                  {res.status === 'success' && res.payslip && (
                    <span className="text-[10px] font-black text-slate-400">
                      {new Intl.NumberFormat('vi-VN').format(parseFloat(res.payslip.net_pay_amount))}
                    </span>
                  )}
                </div>
              ))}
            </div>

            <button 
              onClick={resetAndClose}
              className="w-full py-4 bg-slate-900 text-white rounded-2xl font-black hover:bg-slate-800 transition-all"
            >
              {t("common.close")}
            </button>
          </div>
        )}
      </div>
    </Modal>
  );
}
