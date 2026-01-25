import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Plus, 
  Coins, 
  Calendar, 
  Wallet, 
  Trash2, 
  ShieldCheck, 
  AlertCircle, 
  ChevronDown, 
  ChevronRight,
  History,
  ArrowUpRight
} from "lucide-react";
import { 
  fetchSalaryComponents, 
  fetchEmployeeSalaryComponents, 
  assignEmployeeSalaryComponent,
  fetchEmployeePayrollConfigs,
  setEmployeePayrollConfig
} from "@/lib/api/payroll";
import { useToast } from "@/components/Toast";
import { useTranslation } from "react-i18next";
import { EmployeeSalaryComponent, SalaryComponentDefinition } from "@/lib/api/types";
import { Combobox } from "@/components/ui/Combobox";
import { format } from "date-fns";
import TemporalConfigTable from "../common/TemporalConfigTable";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { cn } from "@/lib/utils";

interface Props {
  wsId: string;
  employeeId: string;
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
  const [expandedGroups, setExpandedGroups] = useState<Record<string, boolean>>({});

  const { data: definitions = [] } = useQuery({ queryKey: ["salary-components", wsId], queryFn: () => fetchSalaryComponents(wsId) });
  const { data: employeeComps = [], isLoading } = useQuery({ queryKey: ["employee-salary-components", employeeId], queryFn: () => fetchEmployeeSalaryComponents(wsId, employeeId) });
  const { data: individualConfigs = [], isLoading: isConfigsLoading } = useQuery({ queryKey: ["employee-payroll-configs", employeeId], queryFn: () => fetchEmployeePayrollConfigs(wsId, employeeId) });

  const groupedComps = employeeComps.reduce((acc, comp) => {
    const key = comp.component_definition_id; if (!acc[key]) acc[key] = [];
    acc[key].push(comp); return acc;
  }, {} as Record<string, EmployeeSalaryComponent[]>);

  Object.keys(groupedComps).forEach(key => { groupedComps[key].sort((a, b) => new Date(b.effective_date).getTime() - new Date(a.effective_date).getTime()); });

  const toggleGroup = (id: string) => { setExpandedGroups(prev => ({ ...prev, [id]: !prev[id] })); };

  const saveIndividualConfigMutation = useMutation({
    mutationFn: (data: { std_working_days: number; hours_per_day: number; effective_date: string }) => setEmployeePayrollConfig(wsId, employeeId, data),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["employee-payroll-configs", employeeId] }); toast.success(t("common.success")); },
  });

  const assignMutation = useMutation({
    mutationFn: (data: Partial<EmployeeSalaryComponent>) => assignEmployeeSalaryComponent(wsId, employeeId, data),
    onSuccess: () => { toast.success(t("common.success")); queryClient.invalidateQueries({ queryKey: ["employee-salary-components", employeeId] }); setIsAdding(false); setAmount(""); setSelectedCompId(""); },
  });

  const handleAssign = (e: React.FormEvent) => {
    e.preventDefault(); if (!selectedCompId || !amount) return;
    assignMutation.mutate({ component_definition_id: selectedCompId, amount: amount, effective_date: effectiveDate });
  };

  const currencyFormatter = new Intl.NumberFormat(t("common.languages." + (t("common.currency") === "VND" ? "vi" : "en")) === "Tiếng Việt" ? "vi-VN" : "en-US");

  return (
    <div className="flex flex-col gap-8 animate-in fade-in duration-500">
      <div className="flex items-center justify-between px-2">
        <div className="flex items-center gap-4">
          <div className="w-10 h-10 rounded-2xl bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50"><Wallet size={20} /></div>
          <div><h3 className="text-lg font-bold text-slate-800 tracking-tight">{t("payroll.employee_config.title")}</h3><p className="text-[10px] font-bold text-slate-500 uppercase tracking-widest mt-0.5">{t("payroll.employee_config.salary_items_subtitle")}</p></div>
        </div>
        <button onClick={() => setIsAdding(!isAdding)} className={cn("flex items-center gap-2 px-6 py-2.5 rounded-2xl font-bold text-[10px] uppercase tracking-widest transition-all", isAdding ? 'bg-slate-100 text-slate-500 border border-slate-200' : 'bg-primary-600 text-white shadow-lg shadow-primary-100 active:scale-95')}>
          {isAdding ? t("common.cancel") : <><Plus size={14} /> {t("payroll.employee_config.add_btn")}</>}
        </button>
      </div>

      {isAdding && (
        <form onSubmit={handleAssign} className="bg-white border-2 border-primary-100 p-10 rounded-[2.5rem] shadow-xl shadow-primary-50/50 animate-in slide-in-from-top-4 duration-500">
          <div className="flex items-center gap-3 mb-8 text-primary-700">
            <div className="w-8 h-8 rounded-xl bg-primary-50 flex items-center justify-center shadow-inner"><ArrowUpRight size={16} /></div>
            <h4 className="text-[10px] font-bold uppercase tracking-[0.2em]">{t("payroll.employee_config.add_new_title")}</h4>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <div className="space-y-3">
              <label htmlFor="salary-comp-select" className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("payroll.employee_config.item_type")}</label>
              <Combobox id="salary-comp-select" options={definitions.map(d => ({ value: d.id, label: d.name, description: d.code }))} value={selectedCompId} onChange={setSelectedCompId} className="bg-slate-50 border-slate-100 h-14" />
            </div>
            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("common.amount")} ({t("common.currency")})</label>
              <input required type="number" placeholder="1,000" className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-bold text-sm uppercase tracking-tight" value={amount} onChange={e => setAmount(e.target.value)} />
            </div>
            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("common.effective_date")}</label>
              <input required type="date" className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-bold text-sm" value={effectiveDate} onChange={e => setEffectiveDate(e.target.value)} />
            </div>
          </div>
          <div className="mt-10 flex justify-end">
            <button type="submit" disabled={assignMutation.isPending} className="bg-primary-600 text-white px-12 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95 disabled:opacity-50">{assignMutation.isPending ? t("common.saving").toUpperCase() : t("common.save")}</button>
          </div>
        </form>
      )}

      <div className="bg-white border border-slate-100 rounded-[2.5rem] overflow-hidden shadow-sm">
        <table className="w-full text-left border-collapse">
          <thead className="bg-slate-50/50 border-b border-slate-100">
            <tr>
              <th className="px-8 py-5 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t("payroll.employee_config.salary_item")}</th>
              <th className="px-8 py-5 text-[10px] font-bold text-slate-500 uppercase tracking-widest text-right">{t("common.amount")}</th>
              <th className="px-8 py-5 text-[10px] font-bold text-slate-500 uppercase tracking-widest text-center">{t("common.base_for_ot")}</th>
              <th className="px-8 py-5 text-[10px] font-bold text-slate-500 uppercase tracking-widest text-right">{t("common.effective_date")}</th>
              <th className="px-8 py-5 border-b border-slate-100"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50">
            {isLoading ? (
              <tr><td colSpan={5} className="px-8 py-12 text-center text-slate-500 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.loading")}</td></tr>
            ) : Object.keys(groupedComps).length === 0 ? (
              <tr><td colSpan={5} className="px-8 py-20 text-center text-slate-500 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("payroll.employee_config.no_config")}</td></tr>
            ) : (
              Object.entries(groupedComps).map(([defId, comps]) => {
                const latest = comps[0]; const hasHistory = comps.length > 1; const isExpanded = expandedGroups[defId];
                return (
                  <React.Fragment key={defId}>
                    <tr className={cn("group transition-all hover:bg-slate-50/50", isExpanded && "bg-slate-50/30")}>
                      <td className="px-8 py-6">
                        <div className="flex items-center gap-4">
                          {hasHistory ? (
                            <button onClick={() => toggleGroup(defId)} className={cn("p-1.5 rounded-lg transition-all border shadow-sm", isExpanded ? "bg-white text-primary-600 border-primary-100 rotate-90" : "bg-slate-50 text-slate-300 border-slate-100 hover:text-slate-500")}><ChevronRight size={14} /></button>
                          ) : <div className="w-8" />}
                          <div>
                            <div className="font-bold text-slate-700 text-sm tracking-tight uppercase leading-none">{latest.name}</div>
                            <div className="text-[9px] font-mono font-bold text-slate-500 uppercase tracking-widest mt-1.5 opacity-70">{latest.code}</div>
                          </div>
                        </div>
                      </td>
                      <td className="px-8 py-6 text-right font-mono font-bold text-slate-700 text-sm">{currencyFormatter.format(parseFloat(latest.amount))}</td>
                      <td className="px-8 py-6 text-center">
                        <span className={cn("inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm", latest.is_base_for_ot ? 'bg-emerald-50 text-emerald-600 border-emerald-100' : 'bg-slate-100 text-slate-500 border-slate-200')}>
                          {latest.is_base_for_ot ? <ShieldCheck size={10} /> : null} {latest.is_base_for_ot ? t("common.yes") : t("common.no")}
                        </span>
                      </td>
                      <td className="px-8 py-6 text-right">
                        <div className="flex flex-col items-end">
                          <div className="text-xs font-bold text-slate-700 uppercase tracking-tighter">{formatDate(latest.effective_date)}</div>
                          <div className="text-[8px] font-bold uppercase text-emerald-600 tracking-[0.2em] mt-1 opacity-80">{t("common.active")}</div>
                        </div>
                      </td>
                      <td className="px-8 py-6 text-right">
                        <button className="p-2.5 bg-white text-slate-300 hover:text-rose-600 rounded-xl shadow-md border border-slate-100 transition-all opacity-0 group-hover:opacity-100"><Trash2 size={16} /></button>
                      </td>
                    </tr>
                    {isExpanded && comps.slice(1).map((comp) => (
                      <tr key={comp.id} className="bg-slate-50/20 text-slate-500 animate-in slide-in-from-top-2 duration-300">
                        <td className="px-8 py-4 pl-20 relative">
                          <div className="absolute left-12 top-0 bottom-0 w-px bg-slate-200" />
                          <div className="flex items-center gap-3">
                            <div className="w-6 h-6 rounded-lg bg-white border border-slate-100 flex items-center justify-center text-slate-300 shadow-sm"><History size={12} /></div>
                            <span className="text-[9px] font-bold text-slate-500 uppercase tracking-widest opacity-60 italic">{t("common.expired", "Archive")}</span>
                          </div>
                        </td>
                        <td className="px-8 py-4 text-right font-mono text-xs font-bold opacity-60">{currencyFormatter.format(parseFloat(comp.amount))}</td>
                        <td className="px-8 py-4 text-center opacity-40 text-[9px] font-bold uppercase tracking-widest">{comp.is_base_for_ot ? t("common.yes") : t("common.no")}</td>
                        <td className="px-8 py-4 text-right text-xs font-bold font-mono opacity-50 uppercase tracking-tighter">{formatDate(comp.effective_date)}</td>
                        <td className="px-8 py-4 text-right pr-10">
                          <button className="p-1.5 text-slate-300 hover:text-rose-600 transition-all opacity-0 group-hover:opacity-100"><Trash2 size={14} /></button>
                        </td>
                      </tr>
                    ))}
                  </React.Fragment>
                );
              })
            )}
          </tbody>
        </table>
      </div>
      
      <div className="p-8 bg-amber-50/50 border border-amber-100 rounded-[2.5rem] flex gap-5 shadow-inner">
        <div className="w-12 h-12 rounded-2xl bg-white flex items-center justify-center text-amber-500 shadow-sm border border-amber-100 shrink-0"><AlertCircle size={24} /></div>
        <div className="text-xs text-amber-900 leading-relaxed font-medium pt-1">
          <p className="font-bold uppercase tracking-[0.2em] mb-2 text-amber-950 opacity-80">{t("payroll.employee_config.effective_note_title")}</p>
          <p className="opacity-70 font-bold uppercase tracking-tight">{t("payroll.employee_config.effective_note")}</p>
        </div>
      </div>

      <div className="pt-10 border-t border-slate-100">
        <TemporalConfigTable title={t("payroll.settings.std_days_override")} records={individualConfigs} isLoading={isConfigsLoading} onSave={async (data) => { await saveIndividualConfigMutation.mutateAsync(data); }} />
      </div>
    </div>
  );
}
