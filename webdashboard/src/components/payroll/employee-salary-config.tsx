import type { TFunction } from "i18next";

import { UseMutationResult, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  AlertCircle,
  ArrowUpRight,
  ChevronRight,
  History,
  Plus,
  Trash2,
  Wallet,
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import { Combobox } from "@/components/ui/combobox";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import {
  assignEmployeeSalaryComponent,
  fetchEmployeePayrollConfigs,
  fetchEmployeeSalaryComponents,
  fetchSalaryComponents,
  setEmployeePayrollConfig,
} from "@/lib/api/payroll";
import { EmployeeSalaryComponent } from "@/lib/api/types";
import { cn } from "@/lib/utils";

import TemporalConfigTable from "../common/temporal-config-table";

interface SalaryCompDefinition {
  code: string;
  id: string;
  name: string;
}

interface AddSalaryFormProps {
  amount: string;
  assignMutation: UseMutationResult<unknown, Error, Partial<EmployeeSalaryComponent>>;
  definitions: SalaryCompDefinition[];
  effectiveDate: string;
  onSubmit: (e: React.SyntheticEvent) => void;
  selectedCompId: string;
  setAmount: (v: string) => void;
  setEffectiveDate: (v: string) => void;
  setSelectedCompId: (v: string) => void;
  t: TFunction;
}

function AddSalaryForm({
  amount, assignMutation, definitions, effectiveDate, onSubmit,
  selectedCompId, setAmount, setEffectiveDate, setSelectedCompId, t,
}: AddSalaryFormProps) {
  return (
    <form className="bg-white dark:bg-slate-900 border-2 border-primary-100 dark:border-primary-800 p-10 rounded-[2.5rem] shadow-xl shadow-primary-50/50 animate-in slide-in-from-top-4 duration-500" onSubmit={onSubmit}>
      <div className="flex items-center gap-3 mb-8 text-primary-700 dark:text-primary-300">
        <div className="w-8 h-8 rounded-xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center shadow-inner"><ArrowUpRight size={16} /></div>
        <Typography className="text-[10px] font-bold uppercase tracking-[0.2em]" variant="h4">{t("payroll.employee_config.add_new_title")}</Typography>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="salary-comp-select">{t("payroll.employee_config.item_type")}</label>
          <Combobox className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" id="salary-comp-select" onChange={setSelectedCompId} options={definitions.map(d => ({ value: d.id, label: d.name, description: d.code }))} value={selectedCompId} />
        </div>
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("common.amount")} ({t("common.currency")})</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight" onChange={e => { setAmount(e.target.value); }} placeholder="1,000" required type="number" value={amount} />
        </div>
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("common.effective_date")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm" onChange={e => { setEffectiveDate(e.target.value); }} required type="date" value={effectiveDate} />
        </div>
      </div>
      <div className="mt-10 flex justify-end">
        <button className="bg-primary-600 text-white px-12 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50" disabled={assignMutation.isPending} type="submit">{assignMutation.isPending ? t("common.saving").toUpperCase() : t("common.save")}</button>
      </div>
    </form>
  );
}

interface SalaryTableBodyProps {
  currencyFormatter: Intl.NumberFormat;
  expandedGroups: Partial<Record<string, boolean>>;
  formatDate: (d: string) => string;
  groupedComps: Partial<Record<string, EmployeeSalaryComponent[]>>;
  isLoading: boolean;
  t: TFunction;
  toggleGroup: (id: string) => void;
}

function SalaryTableBody({ isLoading, groupedComps, expandedGroups, currencyFormatter, formatDate, t, toggleGroup }: SalaryTableBodyProps) {
  if (isLoading) {
    return <tr><td className="px-8 py-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50" colSpan={4}>{t("common.loading")}</td></tr>;
  }
  if (Object.keys(groupedComps).length === 0) {
    return <tr><td className="px-8 py-20 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50" colSpan={4}>{t("payroll.employee_config.no_config")}</td></tr>;
  }
  return (
    <>
      {Object.entries(groupedComps).map(([defId, comps]) => {
        if (!comps) return null;
        const latest = comps[0]; const hasHistory = comps.length > 1; const isExpanded = expandedGroups[defId];
        return (
          <React.Fragment key={defId}>
            <tr className={cn("group transition-all hover:bg-slate-50 dark:hover:bg-slate-800/50", isExpanded && "bg-slate-50 dark:bg-slate-800/30")}>
              <td className="px-8 py-6">
                <div className="flex items-center gap-4">
                  {hasHistory ? (
                    <button className={cn("p-1.5 rounded-lg transition-all border shadow-sm", isExpanded ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 border-primary-100 dark:border-primary-800 rotate-90" : "bg-slate-50 dark:bg-slate-800 text-slate-300 dark:text-slate-600 border-slate-100 dark:border-slate-800 hover:text-slate-500 dark:text-slate-400")} onClick={() => { toggleGroup(defId); }}><ChevronRight size={14} /></button>
                  ) : <div className="w-8" />}
                  <div>
                    <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none">{latest.name}</div>
                    <div className="text-[9px] font-mono font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1.5 opacity-70">{latest.code}</div>
                  </div>
                </div>
              </td>
              <td className="px-8 py-6 text-right font-mono font-bold text-slate-700 dark:text-slate-200 text-sm">{currencyFormatter.format(parseFloat(latest.amount))}</td>
              <td className="px-8 py-6 text-right">
                <div className="flex flex-col items-end">
                  <div className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tighter">{formatDate(latest.effective_date)}</div>
                  <div className="text-[8px] font-bold uppercase text-emerald-600 dark:text-emerald-300 tracking-[0.2em] mt-1 opacity-80">{t("common.active")}</div>
                </div>
              </td>
              <td className="px-8 py-6 text-right"><button className="p-2.5 bg-white dark:bg-slate-900 text-slate-300 dark:text-slate-600 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all opacity-0 group-hover:opacity-100"><Trash2 size={16} /></button></td>
            </tr>
            {isExpanded ? comps.slice(1).map((comp) => (
              <tr className="bg-slate-50 dark:bg-slate-800/20 text-slate-500 dark:text-slate-400 animate-in slide-in-from-top-2 duration-300" key={comp.id}>
                <td className="px-8 py-4 pl-20 relative">
                  <div className="absolute left-12 top-0 bottom-0 w-px bg-slate-200 dark:bg-slate-600" />
                  <div className="flex items-center gap-3">
                    <div className="w-6 h-6 rounded-lg bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-300 dark:text-slate-600 shadow-sm"><History size={12} /></div>
                    <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest opacity-60 italic">{t("common.expired", "Archive")}</span>
                  </div>
                </td>
                <td className="px-8 py-4 text-right font-mono text-xs font-bold opacity-60">{currencyFormatter.format(parseFloat(comp.amount))}</td>
                <td className="px-8 py-4 text-right text-xs font-bold font-mono opacity-50 uppercase tracking-tighter">{formatDate(comp.effective_date)}</td>
                <td className="px-8 py-4 text-right pr-10"><button className="p-1.5 text-slate-300 dark:text-slate-600 hover:text-rose-600 dark:text-rose-300 transition-all opacity-0 group-hover:opacity-100"><Trash2 size={14} /></button></td>
              </tr>
            )) : null}
          </React.Fragment>
        );
      })}
    </>
  );
}

interface Props {
  employeeId: string;
  wsId: string;
}

export default function EmployeeSalaryConfig({ wsId, employeeId }: Props) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const { formatDate } = useDateTimeFormatter();

  const [isAdding, setIsAdding] = useState(false);
  const [selectedCompId, setSelectedCompId] = useState("");
  const [amount, setAmount] = useState("");
  const [effectiveDate, setEffectiveDate] = useState(format(new Date(), "yyyy-MM-01"));
  const [expandedGroups, setExpandedGroups] = useState<Partial<Record<string, boolean>>>({});

  const { data: definitions = [] } = useQuery({ queryKey: ["salary-components", wsId], queryFn: () => fetchSalaryComponents(wsId) });
  const { data: employeeComps = [], isLoading } = useQuery({ queryKey: ["employee-salary-components", employeeId], queryFn: () => fetchEmployeeSalaryComponents(wsId, employeeId) });
  const { data: individualConfigs = [], isLoading: isConfigsLoading } = useQuery({ queryKey: ["employee-payroll-configs", employeeId], queryFn: () => fetchEmployeePayrollConfigs(wsId, employeeId) });

  const groupedComps = employeeComps.reduce<Partial<Record<string, EmployeeSalaryComponent[]>>>((acc, comp) => {
    const key = comp.component_definition_id;
    acc[key] ??= [];
    acc[key].push(comp);
    return acc;
  }, {});
  Object.keys(groupedComps).forEach(key => { groupedComps[key]?.sort((a, b) => new Date(b.effective_date).getTime() - new Date(a.effective_date).getTime()); });

  const toggleGroup = (id: string) => { setExpandedGroups(prev => ({ ...prev, [id]: !prev[id] })); };

  const saveIndividualConfigMutation = useMutation({
    mutationFn: (data: { effective_date: string; hours_per_day: number; std_working_days: number }) => setEmployeePayrollConfig(wsId, employeeId, data),
    onSuccess: () => { void queryClient.invalidateQueries({ queryKey: ["employee-payroll-configs", employeeId] }); toast.success(t("common.success")); },
  });

  const assignMutation = useMutation({
    mutationFn: (data: Partial<EmployeeSalaryComponent>) => assignEmployeeSalaryComponent(wsId, employeeId, data),
    onSuccess: () => { toast.success(t("common.success")); void queryClient.invalidateQueries({ queryKey: ["employee-salary-components", employeeId] }); setIsAdding(false); setAmount(""); setSelectedCompId(""); },
  });

  const handleAssign = (e: React.SyntheticEvent) => {
    e.preventDefault();
    if (!selectedCompId || !amount) return;
    assignMutation.mutate({ component_definition_id: selectedCompId, amount, effective_date: effectiveDate });
  };

  const currencyFormatter = new Intl.NumberFormat(t("common.languages." + (t("common.currency") === "VND" ? "vi" : "en")) === "Tiếng Việt" ? "vi-VN" : "en-US");

  return (
    <div className="flex flex-col gap-8 animate-in fade-in duration-500">
      <div className="flex items-center justify-between px-2">
        <div className="flex items-center gap-4">
          <div className="w-10 h-10 rounded-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50"><Wallet size={20} /></div>
          <div><Typography className="text-lg font-bold text-slate-800 dark:text-slate-100 tracking-tight" variant="h3">{t("payroll.employee_config.title")}</Typography><Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-0.5" variant="p">{t("payroll.employee_config.salary_items_subtitle")}</Typography></div>
        </div>
        <button className={cn("flex items-center gap-2 px-6 py-2.5 rounded-2xl font-bold text-[10px] uppercase tracking-widest transition-all", isAdding ? 'bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border border-slate-200 dark:border-slate-700' : 'bg-primary-600 text-white shadow-lg shadow-primary-100 active:scale-95')} onClick={() => { setIsAdding(!isAdding); }}>
          {isAdding ? t("common.cancel") : <><Plus size={14} /> {t("payroll.employee_config.add_btn")}</>}
        </button>
      </div>

      {isAdding ? (
        <AddSalaryForm amount={amount} assignMutation={assignMutation} definitions={definitions} effectiveDate={effectiveDate} onSubmit={handleAssign} selectedCompId={selectedCompId} setAmount={setAmount} setEffectiveDate={setEffectiveDate} setSelectedCompId={setSelectedCompId} t={t} />
      ) : null}

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm">
        <table className="w-full text-left border-collapse">
          <thead className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
            <tr>
              <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("payroll.employee_config.salary_item")}</th>
              <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.amount")}</th>
              <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.effective_date")}</th>
              <th className="px-8 py-5 border-b border-slate-100 dark:border-slate-800"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
            <SalaryTableBody currencyFormatter={currencyFormatter} expandedGroups={expandedGroups} formatDate={formatDate} groupedComps={groupedComps} isLoading={isLoading} t={t} toggleGroup={toggleGroup} />
          </tbody>
        </table>
      </div>

      <div className="p-8 bg-amber-50 dark:bg-amber-950/50 border border-amber-100 dark:border-amber-800 rounded-[2.5rem] flex gap-5 shadow-inner">
        <div className="w-12 h-12 rounded-2xl bg-white dark:bg-slate-900 flex items-center justify-center text-amber-500 shadow-sm border border-amber-100 dark:border-amber-800 shrink-0"><AlertCircle size={24} /></div>
        <div className="text-xs text-amber-900 leading-relaxed font-medium pt-1">
          <Typography className="font-bold uppercase tracking-[0.2em] mb-2 text-amber-950 opacity-80" variant="p">{t("payroll.employee_config.effective_note_title")}</Typography>
          <Typography className="opacity-70 font-bold uppercase tracking-tight" variant="p">{t("payroll.employee_config.effective_note")}</Typography>
        </div>
      </div>

      <div className="pt-10 border-t border-slate-100 dark:border-slate-800">
        <TemporalConfigTable isLoading={isConfigsLoading} onSave={async (data) => { await saveIndividualConfigMutation.mutateAsync(data); }} records={individualConfigs} title={t("payroll.settings.std_days_override")} />
      </div>
    </div>
  );
}
