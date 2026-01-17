import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, Coins, Calendar, Wallet, Trash2, ShieldCheck, AlertCircle } from "lucide-react";
import { 
  fetchSalaryComponents, 
  fetchEmployeeSalaryComponents, 
  assignEmployeeSalaryComponent 
} from "@/lib/api/payroll";
import { useToast } from "@/components/Toast";
import { useTranslation } from "react-i18next";
import { EmployeeSalaryComponent, SalaryComponentDefinition } from "@/lib/api/types";
import { Combobox } from "@/components/ui/Combobox";
import { format } from "date-fns";

interface Props {
  wsId: string;
  employeeId: string;
}

export default function EmployeeSalaryConfig({ wsId, employeeId }: Props) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  
  const [isAdding, setIsAdding] = useState(false);
  const [selectedCompId, setSelectedCompId] = useState("");
  const [amount, setAmount] = useState("");
  const [effectiveDate, setEffectiveDate] = useState(format(new Date(), "yyyy-MM-01"));

  const { data: definitions = [] } = useQuery({
    queryKey: ["salary-components", wsId],
    queryFn: () => fetchSalaryComponents(wsId),
  });

  const { data: employeeComps = [], isLoading } = useQuery({
    queryKey: ["employee-salary-components", employeeId],
    queryFn: () => fetchEmployeeSalaryComponents(wsId, employeeId),
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

  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-center justify-between px-2">
        <div className="flex items-center gap-2">
          <Wallet className="w-4 h-4 text-primary-500" />
          <h3 className="font-black text-[10px] text-slate-400 uppercase tracking-widest">
            {t("payroll.employee_config.title", "Cấu hình lương nhân viên")}
          </h3>
        </div>
        <button
          onClick={() => setIsAdding(!isAdding)}
          data-testid="add-salary-item-btn"
          className={`flex items-center gap-2 px-4 py-2 rounded-xl font-bold text-sm transition-all ${
            isAdding ? 'bg-slate-100 text-slate-500' : 'bg-primary-600 text-white shadow-lg shadow-primary-100'
          }`}
        >
          {isAdding ? t("common.cancel") : <><Plus className="w-4 h-4" /> {t("payroll.employee_config.add_btn", "Thêm khoản lương")}</>}
        </button>
      </div>

      {isAdding && (
        <form onSubmit={handleAssign} className="bg-slate-50 p-6 rounded-[32px] border border-slate-100 animate-in slide-in-from-top-4 duration-300">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Loại khoản lương</label>
              <Combobox
                options={definitions.map(d => ({ value: d.id, label: d.name, description: d.code }))}
                value={selectedCompId}
                onChange={setSelectedCompId}
                className="bg-white border-slate-200"
              />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Số tiền (VND/Tháng)</label>
              <input
                required
                type="number"
                placeholder="10,000,000"
                className="w-full bg-white border border-slate-200 rounded-xl px-4 py-2.5 outline-none focus:ring-2 focus:ring-primary-500 transition-all font-bold"
                value={amount}
                onChange={e => setAmount(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Ngày hiệu lực</label>
              <input
                required
                type="date"
                className="w-full bg-white border border-slate-200 rounded-xl px-4 py-2.5 outline-none focus:ring-2 focus:ring-primary-500 transition-all font-bold"
                value={effectiveDate}
                onChange={e => setEffectiveDate(e.target.value)}
              />
            </div>
          </div>
          <div className="mt-4 flex justify-end">
            <button
              type="submit"
              data-testid="save-salary-item-btn"
              disabled={assignMutation.isPending}
              className="bg-slate-900 text-white px-8 py-2.5 rounded-xl font-bold hover:bg-black transition-all shadow-lg"
            >
              {assignMutation.isPending ? t("common.saving") : t("common.save")}
            </button>
          </div>
        </form>
      )}

      <div className="bg-white border border-slate-100 rounded-[32px] overflow-hidden">
        <table className="w-full text-left">
          <thead className="bg-slate-50/50 border-b border-slate-100">
            <tr>
              <th className="px-6 py-3 text-[10px] font-black text-slate-400 uppercase tracking-widest">Khoản lương</th>
              <th className="px-6 py-3 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">Số tiền</th>
              <th className="px-6 py-3 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center">Tính OT</th>
              <th className="px-6 py-3 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">Hiệu lực</th>
              <th className="px-6 py-3 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50">
            {isLoading ? (
              <tr><td colSpan={5} className="px-6 py-12 text-center text-slate-400 italic">{t("common.loading")}</td></tr>
            ) : employeeComps.length === 0 ? (
              <tr><td colSpan={5} className="px-6 py-12 text-center text-slate-400 italic">Chưa có cấu hình lương</td></tr>
            ) : (
              employeeComps.map(comp => (
                <tr key={comp.id} className="hover:bg-slate-50/50 transition-colors">
                  <td className="px-6 py-4">
                    <div className="font-bold text-slate-900">{comp.name}</div>
                    <div className="text-[10px] text-slate-400 font-mono">{comp.code}</div>
                  </td>
                  <td className="px-6 py-4 text-right font-mono font-bold text-slate-700">
                    {new Intl.NumberFormat('vi-VN').format(parseFloat(comp.amount))}
                  </td>
                  <td className="px-6 py-4 text-center">
                    {comp.is_base_for_ot ? (
                      <span className="inline-flex items-center gap-1 px-2 py-0.5 bg-green-50 text-green-600 rounded-full text-[9px] font-black uppercase tracking-tighter border border-green-100">
                        <ShieldCheck size={10} /> Có
                      </span>
                    ) : (
                      <span className="inline-flex items-center gap-1 px-2 py-0.5 bg-slate-100 text-slate-400 rounded-full text-[9px] font-black uppercase tracking-tighter">
                        Không
                      </span>
                    )}
                  </td>
                  <td className="px-6 py-4 text-right text-xs font-bold text-slate-500">
                    {comp.effective_date}
                  </td>
                  <td className="px-6 py-4 text-center">
                    <button className="p-2 text-slate-300 hover:text-rose-600 transition-colors">
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
      
      <div className="p-4 bg-amber-50 border border-amber-100 rounded-2xl flex gap-3">
        <AlertCircle className="w-5 h-5 text-amber-500 shrink-0 mt-0.5" />
        <p className="text-xs text-amber-700 leading-relaxed italic">
          {t("payroll.employee_config.effective_note", "Lưu ý: Nếu có nhiều bản ghi cho cùng một khoản lương, bản ghi có ngày hiệu lực mới nhất (và gần nhất với ngày tính lương) sẽ được sử dụng.")}
        </p>
      </div>
    </div>
  );
}
