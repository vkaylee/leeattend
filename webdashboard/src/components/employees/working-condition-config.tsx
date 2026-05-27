import type { TFunction } from "i18next";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Plus, 
  Trash2, 
  CheckCircle2, 
  ArrowRight, 
  Clock, 
  FileSpreadsheet, 
  Calendar, 
  History, 
  Baby,
  ShieldAlert
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import type { WorkingCondition } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useConfirm } from "@/hooks/use-confirm";
import { useWorkingConditions } from "@/hooks/use-employees";
import { cn } from "@/lib/utils";

enum EmployeeWorkingConditionType {
  Regular = "REGULAR",
  Maternity = "MATERNITY",
  Sick = "SICK",
  Unpaid = "UNPAID",
}

interface Props {
  employeeId: string;
}

const getIcon = (type: string) => {
  switch (type.toUpperCase()) {
    case "MATERNITY": return <Baby size={18} />;
    case "SICK": return <ShieldAlert size={18} />;
    default: return <Clock size={18} />;
  }
};

interface CreateWorkingConditionData {
  end_date: string | null;
  is_paid: boolean;
  start_date: string;
  type: EmployeeWorkingConditionType;
}

interface WorkingConditionFormProps {
  createMutation: {
    isPending: boolean;
    mutate: (data: CreateWorkingConditionData) => void;
  };
  endDate: string;
  isPaid: boolean;
  isSubmitting: boolean;
  onCancel: () => void;
  setEndDate: (val: string) => void;
  setIsPaid: (val: boolean) => void;
  setStartDate: (val: string) => void;
  setType: (val: EmployeeWorkingConditionType) => void;
  startDate: string;
  t: TFunction;
  type: EmployeeWorkingConditionType;
}

function WorkingConditionForm({
  type,
  setType,
  startDate,
  setStartDate,
  endDate,
  setEndDate,
  isPaid,
  setIsPaid,
  isSubmitting,
  onCancel,
  createMutation,
  t
}: WorkingConditionFormProps) {
  return (
    <div className="bg-white dark:bg-slate-900 border-2 border-primary-100 dark:border-primary-800 rounded-[2.5rem] p-8 shadow-xl shadow-primary-50/50 animate-in slide-in-from-top-4 duration-500">
      <div className="flex items-center gap-3 mb-8 text-primary-700 dark:text-primary-300">
        <div className="w-8 h-8 rounded-xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center shadow-inner"><Plus size={16} /></div>
        <Typography className="text-[10px] font-bold uppercase tracking-[0.2em]" variant="h4">{t("employees.policies.add_condition")}</Typography>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mb-8">
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("employees.policies.type_label")}</label>
          <select className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all appearance-none" onChange={(e) => { setType(e.target.value as EmployeeWorkingConditionType); }} value={type}>
            {Object.values(EmployeeWorkingConditionType).map((tValue) => (
              <option key={tValue} value={tValue}>{tValue}</option>
            ))}
          </select>
        </div>
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("employees.policies.is_paid")}</label>
          <div className="flex gap-4 p-1 bg-slate-100 dark:bg-slate-800/50 rounded-2xl">
            <Button className={cn("flex-1 py-3 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", isPaid ? "bg-white dark:bg-slate-700 text-emerald-600 shadow-sm" : "text-slate-400")} onClick={() => { setIsPaid(true); }} type="button" variant="ghost">{t("common.yes")}</Button>
            <Button className={cn("flex-1 py-3 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", !isPaid ? "bg-white dark:bg-slate-700 text-rose-600 shadow-sm" : "text-slate-400")} onClick={() => { setIsPaid(false); }} type="button" variant="ghost">{t("common.no")}</Button>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mb-8">
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("employees.policies.start_date")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all" onChange={(e) => { setStartDate(e.target.value); }} type="date" value={startDate} />
        </div>
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("employees.policies.end_date")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all" onChange={(e) => { setEndDate(e.target.value); }} type="date" value={endDate} />
        </div>
      </div>

      <div className="flex justify-end gap-4">
        <Button className="px-8 py-4 rounded-2xl font-bold text-[10px] uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all" onClick={onCancel} type="button" variant="ghost">{t("common.cancel")}</Button>
        <Button className="flex items-center gap-3 bg-primary-600 text-white px-10 py-4 rounded-2xl font-bold text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50" disabled={isSubmitting} onClick={() => { createMutation.mutate({ type, start_date: startDate, end_date: endDate === "" ? null : endDate, is_paid: isPaid }); }} variant="ghost">
          <CheckCircle2 size={16} /> {t("common.save").toUpperCase()}
        </Button>
      </div>
    </div>
  );
}

function WorkingConditionHeader({ isAdding, setIsAdding, t }: { isAdding: boolean; setIsAdding: (val: boolean) => void; t: TFunction }) {
  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
      <div className="flex items-center gap-4">
        <div className="w-10 h-10 rounded-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50"><FileSpreadsheet size={20} /></div>
        <div>
          <Typography className="text-lg font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase" variant="h3">{t("employees.policies.conditions_title")}</Typography>
          <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-0.5" variant="p">{t("employees.policies.conditions_desc")}</Typography>
        </div>
      </div>
      {!isAdding && (
        <Button className="flex items-center justify-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl font-bold text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 active:scale-95" onClick={() => { setIsAdding(true); }} variant="ghost"><Plus size={14} /> {t("common.add_new").toUpperCase()}</Button>
      )}
    </div>
  );
}

function useWorkingConditionActions({
  employeeId,
  queryClient,
  confirm,
  setIsAdding,
  t,
}: {
  confirm: (args: { message: string; title: string; variant?: "danger" | "warning" | "info" }) => Promise<boolean>;
  employeeId: string;
  queryClient: ReturnType<typeof useQueryClient>;
  setIsAdding: (val: boolean) => void;
  t: TFunction;
}) {
  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      const response = await fetch(`/api/employees/${employeeId}/working-conditions/${id}`, { method: "DELETE" });
      if (!response.ok) throw new Error("Failed to delete");
    },
    onSuccess: () => { void queryClient.invalidateQueries({ queryKey: ["working-conditions", employeeId] }); },
  });

  const createMutation = useMutation({
    mutationFn: async (data: CreateWorkingConditionData) => {
      const response = await fetch(`/api/employees/${employeeId}/working-conditions`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      if (!response.ok) throw new Error("Failed to create");
      return response.json() as Promise<WorkingCondition>;
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["working-conditions", employeeId] });
      setIsAdding(false);
    },
  });

  const handleDelete = async (id: string) => {
    const ok = await confirm({
      title: t("common.delete_confirm"),
      message: t("employees.policies.delete_warning"),
      variant: "danger",
    });
    if (ok) deleteMutation.mutate(id);
  };

  return { createMutation, handleDelete };
}

export default function WorkingConditionConfig({ employeeId }: Props) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { confirm, ConfirmationDialog } = useConfirm();
  const [isAdding, setIsAdding] = useState(false);
  const [type, setType] = useState<EmployeeWorkingConditionType>(EmployeeWorkingConditionType.Regular);
  const [startDate, setStartDate] = useState(new Date().toISOString().split("T")[0]);
  const [endDate, setEndDate] = useState("");
  const [isPaid, setIsPaid] = useState(true);

  const { data: conditions = [], isLoading } = useWorkingConditions(employeeId);
  const { createMutation, handleDelete } = useWorkingConditionActions({ employeeId, queryClient, confirm, setIsAdding, t });

  if (isLoading) return <div className="p-20 text-center"><div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div></div>;

  return (
    <div className="flex flex-col gap-8 animate-in fade-in duration-500">
      <WorkingConditionHeader isAdding={isAdding} setIsAdding={setIsAdding} t={t} />

      {isAdding ? (
        <WorkingConditionForm
          createMutation={createMutation}
          endDate={endDate}
          isPaid={isPaid}
          isSubmitting={createMutation.isPending}
          onCancel={() => { setIsAdding(false); }}
          setEndDate={setEndDate}
          setIsPaid={setIsPaid}
          setStartDate={setStartDate}
          setType={setType}
          startDate={startDate}
          t={t}
          type={type}
        />
      ) : null}

      <div className="grid grid-cols-1 gap-4">
        {conditions.length === 0 ? (
          <div className="bg-slate-50 dark:bg-slate-800/50 border border-dashed border-slate-200 dark:border-slate-700 rounded-[2rem] p-16 text-center">
            <div className="w-16 h-16 bg-white dark:bg-slate-900 rounded-3xl flex items-center justify-center text-slate-300 dark:text-slate-600 mx-auto mb-6 shadow-sm border border-slate-100 dark:border-slate-800"><History size={32} /></div>
            <Typography className="text-sm font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="h4">{t("employees.policies.no_conditions")}</Typography>
          </div>
        ) : (
          conditions.map((cond) => (
            <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2rem] p-8 hover:shadow-premium-xl transition-all group relative overflow-hidden" key={cond.id}>
              <div className="flex items-start justify-between relative z-10">
                <div className="flex gap-6">
                  <div className="w-14 h-14 rounded-2xl bg-slate-50 dark:bg-slate-800 flex items-center justify-center text-slate-500 dark:text-slate-400 shadow-inner group-hover:bg-primary-50 dark:group-hover:bg-primary-950/50 group-hover:text-primary-600 transition-colors">
                    {getIcon(cond.type)}
                  </div>
                  <div>
                    <div className="flex items-center gap-3 mb-1.5">
                      <Typography className="font-bold text-slate-700 dark:text-slate-200 text-lg tracking-tight uppercase leading-none" variant="h4">{t(`employees.policies.types.${cond.type.toLowerCase()}`)}</Typography>
                      <span className={cn("px-2.5 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-widest border", cond.is_paid ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" : "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800")}>
                        {cond.is_paid ? t("employees.policies.paid_allowance") : t("employees.policies.unpaid_allowance")}
                      </span>
                    </div>
                    <div className="flex items-center gap-4 text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase tracking-widest opacity-70">
                      <div className="flex items-center gap-1.5"><Calendar size={12} /> {cond.start_date}</div>
                      <ArrowRight className="text-slate-300" size={10} />
                      <div className="flex items-center gap-1.5">{cond.end_date ? <><Calendar size={12} /> {cond.end_date}</> : t("common.present")}</div>
                    </div>
                  </div>
                </div>
                <Button className="p-4 rounded-2xl bg-rose-50 dark:bg-rose-950/30 text-rose-600 dark:text-rose-400 hover:bg-rose-600 hover:text-white transition-all shadow-sm opacity-0 group-hover:opacity-100 -translate-y-2 group-hover:translate-y-0" onClick={() => handleDelete(cond.id)} variant="ghost"><Trash2 size={20} /></Button>
              </div>
              <div className="absolute top-0 right-0 w-24 h-24 bg-slate-50 dark:bg-slate-800/50 -rotate-45 translate-x-12 -translate-y-12 opacity-50 transition-all group-hover:bg-primary-50 dark:group-hover:bg-primary-950/20" />
            </div>
          ))
        )}
      </div>
      <ConfirmationDialog />
    </div>
  );
}
