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

  const { data: balances, isLoading } = useQuery({
    queryKey: ["workspace-leave-balances", wsId],
    queryFn: () => fetchWorkspaceLeaveBalances(wsId!),
    enabled: !!wsId,
  });

  const filteredBalances = balances?.filter(b => 
    b.employee_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    b.leave_type.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div className="space-y-8 animate-in fade-in duration-700">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 px-2">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-slate-900">{t("leave_balances.title")}</h1>
          <p className="text-slate-500 mt-1 font-medium">{t("leave_balances.subtitle")}</p>
        </div>
      </div>

      <div className="bg-white rounded-[32px] border border-slate-200 shadow-sm overflow-hidden">
        <div className="p-6 border-b border-slate-100 bg-slate-50/50 flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div className="flex items-center gap-2">
            <Filter className="h-5 w-5 text-primary-600" />
            <h3 className="font-bold text-slate-900">{t("leave_balances.summary_table")}</h3>
          </div>
          <div className="flex items-center gap-3">
            <div className="relative group">
              <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 group-focus-within:text-primary-500 transition-colors" size={18} />
              <input
                type="text"
                placeholder={t("common.search_placeholder")}
                className="pl-12 pr-6 py-2.5 bg-white border border-slate-200 rounded-[1.25rem] outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 transition-all font-bold text-sm w-[300px] shadow-sm"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
              />
            </div>
          </div>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-slate-50/50 border-b border-slate-100">
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.employee")}</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.type")}</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("leave_balances.table.base")}</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("leave_balances.table.accrued")}</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("leave_balances.table.carried_over")}</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("leave_balances.table.used")}</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("leave_balances.table.remaining")}</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("leave_balances.table.usage")}</th>
                <th className="px-8 py-4 text-right"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {isLoading ? (
                <tr><td colSpan={9} className="px-8 py-12 text-center text-slate-400 font-bold italic">{t("common.loading")}</td></tr>
              ) : filteredBalances?.length === 0 ? (
                <tr><td colSpan={9} className="px-8 py-20 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("attendance_logs.messages.no_data")}</td></tr>
              ) : filteredBalances?.map((balance, idx) => (
                <BalanceRow 
                  key={`${balance.employee_id}-${balance.leave_type}-${idx}`} 
                  balance={balance} 
                  onViewHistory={() => setSelectedEmployee({ id: balance.employee_id, name: balance.employee_name })}
                />
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {selectedEmployee && (
        <LeaveHistoryModal
          wsId={wsId!}
          employeeId={selectedEmployee.id}
          employeeName={selectedEmployee.name}
          isOpen={!!selectedEmployee}
          onClose={() => setSelectedEmployee(null)}
        />
      )}
    </div>
  );
}

function BalanceRow({ balance, onViewHistory }: { balance: WorkspaceLeaveBalance, onViewHistory: () => void }) {
  const { t } = useTranslation();
  const totalEntitled = balance.entitlement + balance.accrued_amount + balance.carried_over;
  const usagePercent = Math.round((balance.used / (totalEntitled || 1)) * 100);

  return (
    <tr className="hover:bg-slate-50/50 transition-colors group">
      <td className="px-8 py-5">
        <div className="flex items-center gap-4">
          <div className="w-10 h-10 bg-slate-100 rounded-xl flex items-center justify-center text-slate-400 font-black text-xs shadow-inner">
            {balance.employee_name.split(' ').map(n => n[0]).join('').slice(0, 2).toUpperCase()}
          </div>
          <div>
            <div className="font-black text-slate-900 text-sm tracking-tight">{balance.employee_name}</div>
          </div>
        </div>
      </td>
      <td className="px-8 py-5">
        <span className="px-2.5 py-1 bg-slate-100 text-slate-600 rounded-lg text-[10px] font-black uppercase tracking-tighter border border-slate-200">
          {balance.leave_type}
        </span>
      </td>
      <td className="px-8 py-5 text-right font-mono text-sm font-bold text-slate-500">{balance.entitlement}</td>
      <td className="px-8 py-5 text-right font-mono text-sm font-black text-primary-600">
        <div className="flex items-center justify-end gap-1.5">
          {balance.accrued_amount}
          {balance.accrued_amount > 0 && (
            <div className="group/info relative">
              <Info className="h-3.5 w-3.5 text-slate-300 cursor-help" />
              <div className="absolute bottom-full right-0 mb-2 w-56 p-3 bg-slate-900 text-white text-[10px] font-bold leading-relaxed rounded-xl opacity-0 group-hover/info:opacity-100 transition-opacity pointer-events-none z-10 shadow-xl">
                {t("leave_balances.table.accrued_info")}
              </div>
            </div>
          )}
        </div>
      </td>
      <td className="px-8 py-5 text-right font-mono text-sm font-bold text-slate-400">{balance.carried_over}</td>
      <td className="px-8 py-5 text-right font-mono text-sm font-black text-rose-500">{balance.used}</td>
      <td className="px-8 py-5 text-right">
        <span className={cn("text-sm font-black px-2.5 py-1 rounded-lg font-mono shadow-sm border", balance.remaining > 0 ? "text-emerald-600 bg-emerald-50 border-emerald-100" : "text-slate-400 bg-slate-50 border-slate-100")}>
          {balance.remaining}
        </span>
      </td>
      <td className="px-8 py-5">
        <div className="flex items-center gap-3">
          <div className="w-20 h-2 bg-slate-100 rounded-full overflow-hidden shadow-inner">
            <div 
              className={cn("h-full rounded-full transition-all duration-1000 shadow-sm", usagePercent > 80 ? "bg-rose-500" : usagePercent > 50 ? "bg-amber-500" : "bg-primary-500")} 
              style={{ width: `${Math.min(100, usagePercent)}%` }}
            />
          </div>
          <span className="text-[10px] font-black text-slate-400 font-mono">
            {usagePercent}%
          </span>
        </div>
      </td>
      <td className="px-8 py-5 text-right">
        <button 
          onClick={onViewHistory}
          className="p-2.5 bg-white text-slate-400 hover:text-primary-600 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90 opacity-0 group-hover:opacity-100"
          title={t("leave_balances.table.history_tooltip")}
        >
          <History className="h-5 w-5" />
        </button>
      </td>
    </tr>
  );
}