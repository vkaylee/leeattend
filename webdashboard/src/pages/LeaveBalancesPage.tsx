import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { fetchWorkspaceLeaveBalances, getCurrentWorkspaceId, WorkspaceLeaveBalance } from "@/lib/api";
import { Search, Filter, History, Info } from "lucide-react";
import { LeaveHistoryModal } from "@/components/leave/LeaveHistoryModal";

export default function LeaveBalancesPage() {

  const { t } = useTranslation();

  const wsId = getCurrentWorkspaceId();

  const [searchTerm, setSearchTerm] = useState("");

  const [selectedEmployee, setSelectedEmployee] = useState<{ id: string, name: string } | null>(null);



  const { data: balances, isLoading } = useQuery({ queryKey: ["workspace-leave-balances", wsId], queryFn: () => fetchWorkspaceLeaveBalances(wsId!), enabled: !!wsId });



  const filteredBalances = balances?.filter(b => 

    b.employee_name.toLowerCase().includes(searchTerm.toLowerCase()) ||

    b.leave_type.toLowerCase().includes(searchTerm.toLowerCase())

  );



  return (

    <div className="space-y-10 animate-in fade-in duration-700 pb-20">

      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">

        <div className="space-y-1">

          <h1 className="text-4xl font-black tracking-tighter text-slate-900 uppercase leading-none">{t("leave_balances.title")}</h1>

          <p className="text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] opacity-60">{t("leave_balances.subtitle")}</p>

        </div>

      </div>



      <div className="bg-white border border-slate-100 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">

        <div className="p-8 border-b border-slate-50 flex flex-col md:flex-row md:items-center justify-between gap-6 bg-slate-50/30 px-10">

          <div className="flex items-center gap-3"><Filter className="w-5 h-5 text-primary-600" /><h2 className="text-xl font-black text-slate-800 tracking-tight uppercase">{t("leave_balances.summary_table")}</h2></div>

          <div className="relative group w-full md:w-80">

            <Search className="absolute left-5 top-1/2 -translate-y-1/2 text-slate-300 group-focus-within:text-primary-500 transition-colors" size={18} />

            <input type="text" placeholder={t("common.search_placeholder")} className="w-full pl-14 pr-6 py-3.5 bg-white border border-slate-200 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight shadow-sm" value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} />

          </div>

        </div>



        <div className="overflow-x-auto">

          <table className="w-full text-left border-collapse">

            <thead>

              <tr className="bg-slate-50/50 border-b border-slate-100">

                <th className="px-10 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em]">{t("common.employee")}</th>

                <th className="px-8 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em]">{t("common.type")}</th>

                <th className="px-8 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] text-right">{t("leave_balances.table.base")}</th>

                <th className="px-8 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] text-right">{t("leave_balances.table.accrued")}</th>

                <th className="px-8 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] text-right">{t("leave_balances.table.carried_over")}</th>

                <th className="px-8 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] text-right">{t("leave_balances.table.used")}</th>

                <th className="px-8 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] text-right">{t("leave_balances.table.remaining")}</th>

                <th className="px-8 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em]">{t("leave_balances.table.usage")}</th>

                <th className="px-10 py-6 border-b border-slate-100"></th>

              </tr>

            </thead>

            <tbody className="divide-y divide-slate-50">

              {isLoading ? (

                <tr><td colSpan={9} className="px-10 py-20 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse">{t("common.loading")}</td></tr>

              ) : filteredBalances?.length === 0 ? (

                <tr><td colSpan={9} className="px-10 py-32 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("attendance_logs.messages.no_data")}</td></tr>

              ) : filteredBalances?.map((balance, idx) => (

                <BalanceRow key={`${balance.employee_id}-${balance.leave_type}-${idx}`} balance={balance} onViewHistory={() => setSelectedEmployee({ id: balance.employee_id, name: balance.employee_name })} />

              ))}

            </tbody>

          </table>

        </div>

      </div>



      {selectedEmployee && (

        <LeaveHistoryModal wsId={wsId!} employeeId={selectedEmployee.id} employeeName={selectedEmployee.name} isOpen={!!selectedEmployee} onClose={() => setSelectedEmployee(null)} />

      )}

    </div>

  );

}



function BalanceRow({ balance, onViewHistory }: { balance: WorkspaceLeaveBalance, onViewHistory: () => void }) {

  const { t } = useTranslation();

  const totalEntitled = balance.entitlement + balance.accrued_amount + balance.carried_over;

  const usagePercent = Math.round((balance.used / (totalEntitled || 1)) * 100);



  return (

    <tr className="hover:bg-slate-50/50 transition-all group">

      <td className="px-10 py-6">

        <div className="flex items-center gap-5">

          <div className="w-14 h-14 bg-primary-50 rounded-2xl flex items-center justify-center text-primary-600 font-black text-lg shadow-inner border border-primary-100/50 uppercase group-hover:scale-110 transition-transform">

            {balance.employee_name.split(' ').map(n => n[0]).join('').slice(0, 2)}

          </div>

          <div><div className="font-black text-slate-900 text-sm tracking-tight uppercase leading-none">{balance.employee_name}</div></div>

        </div>

      </td>

      <td className="px-8 py-6">

        <span className="px-3 py-1 bg-slate-50 text-slate-500 rounded-lg text-[10px] font-black uppercase tracking-widest border border-slate-100 shadow-sm">

          {balance.leave_type}

        </span>

      </td>

      <td className="px-8 py-6 text-right font-mono text-sm font-black text-slate-500">{balance.entitlement}</td>

      <td className="px-8 py-6 text-right font-mono text-sm font-black text-primary-600">

        <div className="flex items-center justify-end gap-2 group/info relative">

          {balance.accrued_amount}

          {balance.accrued_amount > 0 && <Info className="h-3.5 w-3.5 text-slate-200 group-hover/info:text-primary-400 transition-colors cursor-help" />}

          <div className="absolute bottom-full right-0 mb-3 w-64 p-4 bg-slate-900 text-white text-[10px] font-bold leading-relaxed rounded-[1.5rem] opacity-0 group-hover/info:opacity-100 transition-all duration-300 pointer-events-none z-10 shadow-premium-2xl border border-white/10 translate-y-2 group-hover/info:translate-y-0">

            <div className="flex items-center gap-2 mb-2 pb-2 border-b border-white/10 text-primary-400"><Info size={12} /><span className="uppercase tracking-[0.2em] font-black">Accrual Details</span></div>

            {t("leave_balances.table.accrued_info")}

          </div>

        </div>

      </td>

      <td className="px-8 py-6 text-right font-mono text-sm font-black text-slate-400">{balance.carried_over}</td>

      <td className="px-8 py-6 text-right font-mono text-sm font-black text-rose-500">{balance.used}</td>

      <td className="px-8 py-6 text-right">

        <span className={cn("text-sm font-black px-3 py-1.5 rounded-xl font-mono shadow-inner border border-transparent transition-all uppercase tracking-tighter", balance.remaining > 0 ? "text-emerald-600 bg-emerald-50 border-emerald-100" : "text-slate-400 bg-slate-100 border-slate-200 opacity-60")}>

          {balance.remaining}

        </span>

      </td>

      <td className="px-8 py-6">

        <div className="flex items-center gap-4">

          <div className="w-24 h-3 bg-slate-100 rounded-full overflow-hidden shadow-inner border border-slate-200/50">

            <div className={cn("h-full rounded-full transition-all duration-1000 shadow-sm", usagePercent > 80 ? "bg-rose-500 shadow-rose-200" : usagePercent > 50 ? "bg-amber-500 shadow-amber-200" : "bg-primary-600 shadow-primary-200")} style={{ width: `${Math.min(100, usagePercent)}%` }} />

          </div>

          <span className="text-[10px] font-black text-slate-400 font-mono tracking-tighter w-8">{usagePercent}%</span>

        </div>

      </td>

      <td className="px-10 py-6 text-right">

        <button onClick={onViewHistory} className="p-3 bg-white text-slate-300 hover:text-primary-600 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90 opacity-0 group-hover:opacity-100 translate-x-2 group-hover:translate-x-0" title={t("leave_balances.table.history_tooltip")}><History className="h-5 w-5" /></button>

      </td>

    </tr>

  );

}
