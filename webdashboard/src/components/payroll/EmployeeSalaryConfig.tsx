import { useState } from "react";
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

  const { data: definitions = [] } = useQuery({
    queryKey: ["salary-components", wsId],
    queryFn: () => fetchSalaryComponents(wsId),
  });

  const { data: employeeComps = [], isLoading } = useQuery({
    queryKey: ["employee-salary-components", employeeId],
    queryFn: () => fetchEmployeeSalaryComponents(wsId, employeeId),
  });

  const { data: individualConfigs = [], isLoading: isConfigsLoading } = useQuery({
    queryKey: ["employee-payroll-configs", employeeId],
    queryFn: () => fetchEmployeePayrollConfigs(wsId, employeeId),
  });

  // Group components by definition ID
  const groupedComps = employeeComps.reduce((acc, comp) => {
    const key = comp.component_definition_id;
    if (!acc[key]) acc[key] = [];
    acc[key].push(comp);
    return acc;
  }, {} as Record<string, EmployeeSalaryComponent[]>);

  // Sort each group by effective date DESC
  Object.keys(groupedComps).forEach(key => {
    groupedComps[key].sort((a, b) => new Date(b.effective_date).getTime() - new Date(a.effective_date).getTime());
  });

  const toggleGroup = (id: string) => {
    setExpandedGroups(prev => ({ ...prev, [id]: !prev[id] }));
  };

  const saveIndividualConfigMutation = useMutation({
    mutationFn: (data: { std_working_days: number; hours_per_day: number; effective_date: string }) => 
      setEmployeePayrollConfig(wsId, employeeId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["employee-payroll-configs", employeeId] });
      toast.success(t("common.success"));
    },
  });

  const assignMutation = useMutation({
    mutationFn: (data: Partial<EmployeeSalaryComponent>) => 
      assignEmployeeSalaryComponent(wsId, employeeId, data),
    onSuccess: () => {
      toast.success(t("common.success"));
      queryClient.invalidateQueries({ queryKey: ["employee-salary-components", employeeId] });
      setIsAdding(false);
      setAmount("");
      setSelectedCompId("");
    },
    onError: (e: any) => toast.error(e.message)
  });

  const handleAssign = (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedCompId || !amount) return;
    assignMutation.mutate({
      component_definition_id: selectedCompId,
      amount: amount,
      effective_date: effectiveDate
    });
  };

  const currencyFormatter = new Intl.NumberFormat(t("common.languages." + (t("common.currency") === "VND" ? "vi" : "en")) === "Tiếng Việt" ? "vi-VN" : "en-US");

  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-center justify-between px-2">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 rounded-xl bg-primary-50 flex items-center justify-center text-primary-600">
            <Wallet size={18} />
          </div>
          <div>
            <h3 className="font-bold text-slate-800 tracking-tight">
              {t("payroll.employee_config.title")}
            </h3>
            <p className="text-[10px] text-slate-400 font-bold uppercase tracking-widest">{t("payroll.employee_config.salary_items_subtitle", "Cấu hình thu nhập hàng tháng")}</p>
          </div>
        </div>
        <button
          onClick={() => setIsAdding(!isAdding)}
          data-testid="add-salary-item-btn"
          className={`flex items-center gap-2 px-5 py-2.5 rounded-2xl font-black text-xs uppercase tracking-widest transition-all ${
            isAdding ? 'bg-slate-100 text-slate-500' : 'bg-primary-600 text-white shadow-lg shadow-primary-100 active:scale-95'
          }`}
        >
          {isAdding ? t("common.cancel") : <><Plus className="w-4 h-4" /> {t("payroll.employee_config.add_btn")}</>}
        </button>
      </div>

      {isAdding && (
        <form onSubmit={handleAssign} className="bg-white border-2 border-primary-100 p-8 rounded-[32px] shadow-xl shadow-primary-50/50 animate-in slide-in-from-top-4 duration-300">
          <div className="flex items-center gap-2 mb-6 text-primary-700">
            <ArrowUpRight className="w-4 h-4" />
            <h4 className="text-xs font-black uppercase tracking-widest">{t("payroll.employee_config.add_new_title", "Gán khoản lương mới")}</h4>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("payroll.employee_config.item_type")}</label>
              <Combobox
                options={definitions.map(d => ({ value: d.id, label: d.name, description: d.code }))}
                value={selectedCompId}
                onChange={setSelectedCompId}
                className="bg-slate-50 border-slate-100"
              />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.amount")} ({t("common.currency")})</label>
              <input
                required
                type="number"
                placeholder="1,000"
                className="w-full bg-slate-50 border border-slate-100 rounded-xl px-4 py-3 outline-none focus:ring-2 focus:ring-primary-500 transition-all font-bold text-sm"
                value={amount}
                onChange={e => setAmount(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.effective_date")}</label>
              <input
                required
                type="date"
                className="w-full bg-slate-50 border border-slate-100 rounded-xl px-4 py-3 outline-none focus:ring-2 focus:ring-primary-500 transition-all font-bold text-sm"
                value={effectiveDate}
                onChange={e => setEffectiveDate(e.target.value)}
              />
            </div>
          </div>
          <div className="mt-8 flex justify-end">
            <button
              type="submit"
              data-testid="save-salary-item-btn"
              disabled={assignMutation.isPending}
              className="bg-primary-600 text-white px-10 py-3 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100"
            >
              {assignMutation.isPending ? t("common.saving") : t("common.save")}
            </button>
          </div>
        </form>
      )}

      <div className="bg-white border border-slate-100 rounded-[32px] overflow-hidden shadow-sm">
        <table className="w-full text-left">
          <thead className="bg-slate-50/50 border-b border-slate-100">
            <tr>
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("payroll.employee_config.salary_item")}</th>
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.amount")}</th>
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center">{t("common.base_for_ot")}</th>
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.effective_date")}</th>
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50">
            {isLoading ? (
              <tr><td colSpan={5} className="px-6 py-12 text-center text-slate-400 italic">{t("common.loading")}</td></tr>
            ) : Object.keys(groupedComps).length === 0 ? (
              <tr><td colSpan={5} className="px-6 py-12 text-center text-slate-400 italic">{t("payroll.employee_config.no_config")}</td></tr>
            ) : (
              Object.entries(groupedComps).map(([defId, comps]) => {
                const latest = comps[0];
                const hasHistory = comps.length > 1;
                const isExpanded = expandedGroups[defId];

                return (
                  <React.Fragment key={defId}>
                    {/* Main Row (Latest Version) */}
                    <tr className={cn(
                      "group transition-all",
                      isExpanded ? "bg-slate-50/50" : "hover:bg-slate-50/50"
                    )}>
                      <td className="px-6 py-5">
                        <div className="flex items-center gap-3">
                          {hasHistory && (
                            <button 
                              onClick={() => toggleGroup(defId)}
                              className="p-1 hover:bg-white rounded-lg transition-colors text-slate-400 hover:text-primary-600 shadow-sm border border-transparent hover:border-slate-100"
                            >
                              {isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                            </button>
                          )}
                          {!hasHistory && <div className="w-6" />}
                          <div>
                            <div className="font-black text-slate-900 text-sm tracking-tight">{latest.name}</div>
                            <div className="text-[10px] text-slate-400 font-bold uppercase tracking-tighter">{latest.code}</div>
                          </div>
                        </div>
                      </td>
                      <td className="px-6 py-5 text-right">
                        <div className="font-mono font-black text-slate-900 text-sm">
                          {currencyFormatter.format(parseFloat(latest.amount))}
                        </div>
                      </td>
                      <td className="px-6 py-5 text-center">
                        {latest.is_base_for_ot ? (
                          <span className="inline-flex items-center gap-1 px-2.5 py-1 bg-emerald-50 text-emerald-600 rounded-full text-[9px] font-black uppercase tracking-tighter border border-emerald-100">
                            <ShieldCheck size={10} /> {t("common.yes")}
                          </span>
                        ) : (
                          <span className="inline-flex items-center gap-1 px-2.5 py-1 bg-slate-100 text-slate-400 rounded-full text-[9px] font-black uppercase tracking-tighter">
                            {t("common.no")}
                          </span>
                        )}
                      </td>
                      <td className="px-6 py-5 text-right">
                        <div className="flex flex-col items-end">
                          <div className="text-xs font-black text-slate-700">{formatDate(latest.effective_date)}</div>
                          <div className="text-[9px] font-black uppercase text-emerald-600 tracking-tighter bg-emerald-50 px-1 rounded">{t("common.active", "Đang áp dụng")}</div>
                        </div>
                      </td>
                      <td className="px-6 py-5 text-center">
                        <button className="p-2 text-slate-300 hover:text-rose-600 transition-colors opacity-0 group-hover:opacity-100">
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </td>
                    </tr>

                    {/* History Rows (Expandable) */}
                    {isExpanded && comps.slice(1).map((comp, idx) => (
                      <tr key={comp.id} className="bg-slate-50/30 text-slate-500 border-l-4 border-slate-200">
                        <td className="px-6 py-3 pl-16">
                          <div className="flex items-center gap-2">
                            <History size={12} className="text-slate-300" />
                            <span className="text-[10px] font-black uppercase tracking-widest">{t("common.expired", "Đã hết hạn")}</span>
                          </div>
                        </td>
                        <td className="px-6 py-3 text-right font-mono text-xs font-bold">
                          {currencyFormatter.format(parseFloat(comp.amount))}
                        </td>
                        <td className="px-6 py-3 text-center opacity-50">
                          {comp.is_base_for_ot ? t("common.yes") : t("common.no")}
                        </td>
                        <td className="px-6 py-3 text-right text-xs font-medium">
                          {formatDate(comp.effective_date)}
                        </td>
                        <td className="px-6 py-3 text-center">
                          <button className="p-1.5 text-slate-300 hover:text-rose-600 transition-colors">
                            <Trash2 className="w-3.5 h-3.5" />
                          </button>
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
      
      <div className="p-6 bg-amber-50/50 border border-amber-100 rounded-3xl flex gap-4">
        <div className="w-10 h-10 rounded-xl bg-amber-100 flex items-center justify-center text-amber-600 shrink-0">
          <AlertCircle size={20} />
        </div>
        <div className="text-xs text-amber-800 leading-relaxed font-medium">
          <p className="font-black uppercase tracking-widest mb-1 text-amber-900">{t("payroll.employee_config.effective_note_title", "Quy tắc áp dụng")}</p>
          <p className="opacity-80">{t("payroll.employee_config.effective_note")}</p>
        </div>
      </div>

      <div className="pt-8 border-t border-slate-100">
        <TemporalConfigTable 
          title={t("payroll.settings.std_days_override", "Đặc cách ngày công chuẩn")}
          records={individualConfigs}
          isLoading={isConfigsLoading}
          onSave={async (data) => {
            await saveIndividualConfigMutation.mutateAsync(data);
          }}
        />
      </div>
    </div>
  );
}
