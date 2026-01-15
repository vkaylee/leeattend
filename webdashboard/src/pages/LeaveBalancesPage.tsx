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

  // Fetch All Balances in Workspace
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
    <div className="p-8 space-y-6">
      <div className="max-w-7xl mx-auto">
        <div className="mb-8 flex justify-between items-center">
          <div>
            <h1 className="text-3xl font-extrabold tracking-tight text-slate-900">
              {t("common.leave_balances")}
            </h1>
            <p className="text-slate-500 mt-1">Quản lý định mức nghỉ phép của nhân viên trong năm.</p>
          </div>
        </div>

        <div className="bg-white rounded-3xl border border-slate-200 shadow-sm overflow-hidden">
          <div className="p-6 border-b border-slate-100 bg-slate-50/50 flex flex-col md:flex-row md:items-center justify-between gap-4">
            <h2 className="text-lg font-bold text-slate-800">Bảng tổng hợp số dư</h2>
            <div className="flex items-center gap-3">
              <div className="relative">
                <Search className="absolute left-3 top-2.5 h-4 w-4 text-slate-400" />
                <input
                  type="text"
                  placeholder={t("common.search_placeholder")}
                  className="pl-10 pr-4 py-2 bg-white border border-slate-200 rounded-xl outline-none focus:ring-2 focus:ring-primary-500 transition-all text-sm w-[250px]"
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                />
              </div>
              <button className="p-2 bg-white border border-slate-200 rounded-xl text-slate-500 hover:bg-slate-50 transition-all">
                <Filter className="h-4 w-4" />
              </button>
            </div>
          </div>

          <div className="overflow-x-auto">
            <table className="w-full text-left">
              <thead>
                <tr className="bg-slate-50 border-b border-slate-100">
                  <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest">{t("common.employee")}</th>
                  <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest">{t("common.type")}</th>
                  <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest">Gốc</th>
                  <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest">Tích lũy</th>
                  <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest">Chuyển từ năm trước</th>
                  <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest">Đã dùng</th>
                  <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest">Còn lại</th>
                  <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest">Sử dụng</th>
                  <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest text-right">{t("common.actions")}</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-50">
                {isLoading ? (
                  <tr><td colSpan={9} className="px-6 py-12 text-center text-slate-400 italic">{t("common.loading")}</td></tr>
                ) : filteredBalances?.length === 0 ? (
                  <tr><td colSpan={9} className="px-6 py-12 text-center text-slate-400 italic">No data found.</td></tr>
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
    <tr className="hover:bg-slate-50/50 transition-colors">
      <td className="px-6 py-4">
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 bg-primary-50 text-primary-600 rounded-2xl flex items-center justify-center font-bold text-sm shadow-sm">
            {balance.employee_name.split(' ').map(n => n[0]).join('').slice(0, 2)}
          </div>
          <div>
            <div className="font-bold text-slate-900">{balance.employee_name}</div>
          </div>
        </div>
      </td>
      <td className="px-6 py-4">
        <span className="px-2.5 py-1 bg-slate-100 text-slate-600 rounded-lg text-xs font-bold uppercase tracking-wider capitalize">
          {balance.leave_type}
        </span>
      </td>
      <td className="px-6 py-4 text-sm font-medium text-slate-600">{balance.entitlement}</td>
      <td className="px-6 py-4 text-sm font-medium text-primary-600">
        <div className="flex items-center gap-1">
          {balance.accrued_amount}
          {balance.accrued_amount > 0 && (
            <div className="group relative">
              <Info className="h-3 w-3 text-slate-300 cursor-help" />
              <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 w-48 p-2 bg-slate-800 text-white text-[10px] rounded-lg opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-10">
                Bao gồm số ngày tích lũy định kỳ và các điều chỉnh thủ công.
              </div>
            </div>
          )}
        </div>
      </td>
      <td className="px-6 py-4 text-sm font-medium text-slate-500">{balance.carried_over}</td>
      <td className="px-6 py-4 text-sm font-medium text-rose-600">{balance.used}</td>
      <td className="px-6 py-4">
        <span className={`text-sm font-black px-2 py-1 rounded-lg ${balance.remaining > 0 ? 'text-emerald-600 bg-emerald-50' : 'text-slate-400 bg-slate-50'}`}>
          {balance.remaining}
        </span>
      </td>
      <td className="px-6 py-4">
        <div className="flex items-center gap-2">
          <div className="w-16 h-1.5 bg-slate-100 rounded-full overflow-hidden">
            <div 
              className="h-full bg-primary-500 rounded-full" 
              style={{ width: `${Math.min(100, usagePercent)}%` }}
            />
          </div>
          <span className="text-[10px] font-bold text-slate-400">
            {usagePercent}%
          </span>
        </div>
      </td>
      <td className="px-6 py-4 text-right">
        <button 
          onClick={onViewHistory}
          className="p-2 text-slate-400 hover:text-primary-600 hover:bg-primary-50 rounded-xl transition-all"
          title="Lịch sử & Điều chỉnh"
        >
          <History className="h-5 w-5" />
        </button>
      </td>
    </tr>
  );
}