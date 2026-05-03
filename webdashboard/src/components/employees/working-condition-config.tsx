import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Plus, 
  Trash2, 
  AlertCircle, 
  Clock,
  CheckCircle2,
  Calendar,
  Baby,
  ShieldAlert
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { 
  fetchWorkingConditions,
  createWorkingCondition,
  deleteWorkingCondition
} from "@/lib/api/employees";
import { WorkingCondition } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface Props {
  employeeId: string;
  wsId: string;
}

export default function WorkingConditionConfig({ wsId, employeeId }: Props) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const { formatDate } = useDateTimeFormatter();
  const { confirm, ConfirmationDialog } = useConfirm();
  
  const [isAdding, setIsAdding] = useState(false);
  const [type, setType] = useState("EARLY_LEAVE");
  const [minuteAllowance, setMinuteAllowance] = useState(60);
  const [isPaid, setIsPaid] = useState(true);
  const [startDate, setStartDate] = useState(format(new Date(), "yyyy-MM-01"));
  const [endDate, setEndDate] = useState("");

  const { data: conditions = [], isLoading } = useQuery({ 
    queryKey: ["working-conditions", employeeId], 
    queryFn: () => fetchWorkingConditions(wsId, employeeId) 
  });

  const createMutation = useMutation({
    mutationFn: (data: Partial<WorkingCondition>) => createWorkingCondition(wsId, employeeId, data),
    onSuccess: () => { 
      toast.success(t("common.success")); 
      void queryClient.invalidateQueries({ queryKey: ["working-conditions", employeeId] }); 
      setIsAdding(false); 
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteWorkingCondition(wsId, employeeId, id),
    onSuccess: () => { 
      toast.success(t("common.success")); 
      void queryClient.invalidateQueries({ queryKey: ["working-conditions", employeeId] }); 
    },
  });

  const handleDelete = async (id: string) => {
    const ok = await confirm({
      title: t("common.warning", { defaultValue: "Warning" }),
      message: t("common.confirm_delete"),
      variant: "danger"
    });
    if (ok) {
      deleteMutation.mutate(id);
    }
  };

  const handleAdd = (e: React.FormEvent) => {
    e.preventDefault();
    createMutation.mutate({
      type,
      minute_allowance: minuteAllowance,
      is_paid: isPaid,
      start_date: startDate,
      end_date: endDate || undefined,
    });
  };

  const getIcon = (type: string) => {
    switch (type) {
      case "EARLY_LEAVE": return <Clock className="text-orange-500" size={18} />;
      case "BREASTFEEDING": return <Baby className="text-pink-500" size={18} />;
      default: return <AlertCircle className="text-primary-500" size={18} />;
    }
  };

  if (isLoading) return <div className="p-10 text-center animate-pulse font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest text-[10px]">{t("common.loading")}...</div>;

  return (
    <div className="flex flex-col gap-8 animate-in fade-in duration-500">
      <div className="flex items-center justify-between px-2">
        <div className="flex items-center gap-4">
          <div className="w-10 h-10 rounded-2xl bg-amber-50 dark:bg-amber-950 flex items-center justify-center text-amber-600 dark:text-amber-300 shadow-sm border border-amber-100 dark:border-amber-800/50"><ShieldAlert size={20} /></div>
          <div><h3 className="text-lg font-bold text-slate-800 dark:text-slate-100 tracking-tight">{t("employees.details.special_policies")}</h3><p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-0.5">{t("employees.policies.subtitle")}</p></div>
        </div>
        <button className={cn("flex items-center gap-2 px-6 py-2.5 rounded-2xl font-bold text-[10px] uppercase tracking-widest transition-all", isAdding ? 'bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border border-slate-200 dark:border-slate-700' : 'bg-primary-600 text-white shadow-lg shadow-primary-100 active:scale-95')} onClick={() => { setIsAdding(!isAdding); }}>
          {isAdding ? t("common.cancel") : <><Plus size={14} /> {t("employees.policies.add_btn")}</>}
        </button>
      </div>

      {isAdding && (
        <form className="bg-white dark:bg-slate-900 border-2 border-primary-100 dark:border-primary-800 p-10 rounded-[2.5rem] shadow-xl shadow-primary-50/50 animate-in slide-in-from-top-4 duration-500 space-y-8" onSubmit={handleAdd}>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("employees.policies.type")}</label>
              <select className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl font-bold text-sm uppercase tracking-tight focus:ring-4 focus:ring-primary-500/10 outline-none transition-all" onChange={(e) => { setType(e.target.value); }} value={type}>
                <option value="EARLY_LEAVE">{t("employees.policies.types.early_leave")}</option>
                <option value="BREASTFEEDING">{t("employees.policies.types.breastfeeding")}</option>
                <option value="LATE_IN">{t("employees.policies.types.late_in")}</option>
              </select>
            </div>
            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("employees.policies.allowance_mins")}</label>
              <input className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl font-bold text-sm tracking-widest focus:ring-4 focus:ring-primary-500/10 outline-none transition-all" onChange={(e) => { setMinuteAllowance(parseInt(e.target.value)); }} type="number" value={minuteAllowance} />
            </div>
            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("employees.policies.start_date")}</label>
              <input className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl font-bold text-sm focus:ring-4 focus:ring-primary-500/10 outline-none transition-all" onChange={(e) => { setStartDate(e.target.value); }} type="date" value={startDate} />
            </div>
            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("employees.policies.end_date")} ({t("common.optional")})</label>
              <input className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl font-bold text-sm focus:ring-4 focus:ring-primary-500/10 outline-none transition-all" onChange={(e) => { setEndDate(e.target.value); }} type="date" value={endDate} />
            </div>
          </div>
          <button className="flex items-center gap-4 bg-slate-50 dark:bg-slate-800 p-6 rounded-2xl border border-slate-100 dark:border-slate-800 group cursor-pointer text-left w-full" onClick={() => { setIsPaid(!isPaid); }} type="button">
            { }
            <div className={cn("w-6 h-6 rounded-lg border-2 flex items-center justify-center transition-all", isPaid ? "bg-primary-600 border-primary-600 text-white" : "bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700")}>
              {!!isPaid && <CheckCircle2 size={16} />}
            </div>
            <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest">{t("employees.policies.is_paid")}</span>
          </button>
          <div className="pt-4 flex justify-end gap-4">
            <button className="px-10 py-4 bg-slate-900 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 dark:shadow-black/30 active:scale-95 disabled:opacity-50" disabled={createMutation.isPending} type="submit">
              {createMutation.isPending ? t("common.saving") : t("common.save")}
            </button>
          </div>
        </form>
      )}

      <div className="grid grid-cols-1 gap-4">
        {conditions.length === 0 ? (
          <div className="p-20 border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[3rem] text-center">
            <div className="w-16 h-16 bg-slate-50 dark:bg-slate-800 rounded-full flex items-center justify-center mx-auto mb-6 text-slate-300 dark:text-slate-600"><ShieldAlert size={32} /></div>
            <p className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">{t("employees.policies.no_data")}</p>
          </div>
        ) : conditions.map((cond) => (
          <div className="group bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] hover:shadow-xl hover:shadow-primary-100/10 transition-all relative overflow-hidden" key={cond.id}>
            <div className="flex justify-between items-center relative z-10">
              <div className="flex items-center gap-6">
                <div className="w-14 h-14 bg-slate-50 dark:bg-slate-800 rounded-2xl flex items-center justify-center transition-transform group-hover:scale-110 duration-500 shadow-inner">
                  {getIcon(cond.type)}
                </div>
                <div>
                  <div className="flex items-center gap-3 mb-1.5">
                    <h4 className="font-bold text-slate-700 dark:text-slate-200 text-lg tracking-tight uppercase leading-none">{t(`employees.policies.types.${cond.type.toLowerCase()}`)}</h4>
                    <span className={cn("px-2.5 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-widest border", cond.is_paid ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" : "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800")}>
                      {cond.is_paid ? t("employees.policies.paid_allowance") : t("employees.policies.unpaid_allowance")}
                    </span>
                  </div>
                  <div className="flex items-center gap-4 text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase tracking-widest opacity-70">
                    <div className="flex items-center gap-1.5"><Clock className="text-primary-400" size={12} /> {cond.minute_allowance} {t("common.minutes")}</div>
                    <div className="w-1 h-1 bg-slate-200 dark:bg-slate-600 rounded-full" />
                    <div className="flex items-center gap-1.5"><Calendar className="text-primary-400" size={12} /> {formatDate(cond.start_date)} {cond.end_date ? `→ ${formatDate(cond.end_date)}` : `(${t("employees.history.present")})`}</div>
                  </div>
                </div>
              </div>
              <button className="p-3 text-slate-200 dark:text-slate-700 hover:text-rose-500 hover:bg-rose-50 dark:bg-rose-950 rounded-xl transition-all active:scale-90 opacity-0 group-hover:opacity-100" onClick={() => handleDelete(cond.id)}>
                <Trash2 size={20} />
              </button>
            </div>
            <div className="absolute -right-8 -bottom-8 w-32 h-32 text-slate-700 dark:text-slate-200 opacity-[0.02] -rotate-12 group-hover:rotate-0 transition-transform duration-700 pointer-events-none">
              {getIcon(cond.type)}
            </div>
          </div>
        ))}
      </div>
      <ConfirmationDialog />
    </div>
  );
}
