import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Banknote, 
  Calculator, 
  Calendar, 
  ChevronRight, 
  Download, 
  FileText, 
  History, 
  Plus, 
  Search, 
  UserCircle,
  X,
  CreditCard,
  CheckCircle2,
  AlertCircle
} from "lucide-react";
import { 
  fetchEmployees, 
  fetchEmployeePayslips, 
  calculatePayroll, 
  getCurrentWorkspaceId,
  Employee,
  Payslip
} from "../lib/api";
import { useToast } from "../components/Toast";

export default function PayrollPage() {
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const toast = useToast();
  
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [isCalcModalOpen, setIsCalcModalOpen] = useState(false);
  
  const [calcYear, setCalcYear] = useState(new Date().getFullYear());
  const [calcMonth, setCalcMonth] = useState(new Date().getMonth() + 1);

  const { data: employees = [], isLoading: empsLoading } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId),
    enabled: !!wsId,
  });

  const { data: payslips = [], isLoading: payslipsLoading } = useQuery({
    queryKey: ["payslips", selectedEmployee?.id],
    queryFn: () => fetchEmployeePayslips(wsId, selectedEmployee!.id),
    enabled: !!selectedEmployee,
  });

  const calcMutation = useMutation({
    mutationFn: (data: { year: number; month: number }) => 
      calculatePayroll(wsId, selectedEmployee!.id, data.year, data.month),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["payslips", selectedEmployee?.id] });
      toast.success("Đã tính lương thành công.");
      setIsCalcModalOpen(false);
    },
    onError: (e: any) => toast.error(e.message, "Lỗi tính lương")
  });

  const filteredEmployees = employees.filter(emp => 
    `${emp.first_name} ${emp.last_name}`.toLowerCase().includes(searchTerm.toLowerCase()) ||
    emp.employee_id_number.toLowerCase().includes(searchTerm.toLowerCase())
  );

  if (!wsId) return <div className="p-8 text-center">Vui lòng chọn Workspace</div>;

  return (
    <div className="flex h-[calc(100vh-8rem)] gap-6 overflow-hidden animate-in fade-in duration-500">
      {/* Left Sidebar: Employee List */}
      <div className={`flex-1 flex flex-col gap-6 overflow-hidden transition-all ${selectedEmployee ? 'max-w-[350px]' : 'max-w-full'}`}>
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-slate-900">Tính lương</h1>
          <p className="text-slate-500">Quản lý thu nhập và phiếu lương</p>
        </div>

        <div className="relative">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 w-5 h-5" />
          <input 
            type="text" 
            placeholder="Tìm nhân viên..."
            className="w-full pl-12 pr-4 py-3.5 bg-white border border-slate-200 rounded-2xl outline-none focus:ring-4 focus:ring-blue-500/10 focus:border-blue-500 transition-all"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>

        <div className="bg-white border border-slate-200 rounded-3xl flex-1 overflow-auto shadow-sm">
          {empsLoading ? (
            <div className="p-8 text-center text-slate-400 italic">Đang tải...</div>
          ) : (
            <div className="divide-y divide-slate-50">
              {filteredEmployees.map(emp => (
                <button
                  key={emp.id}
                  onClick={() => setSelectedEmployee(emp)}
                  className={`w-full flex items-center gap-4 p-4 hover:bg-slate-50 transition-all text-left group ${selectedEmployee?.id === emp.id ? 'bg-blue-50/50' : ''}`}
                >
                  <div className={`w-10 h-10 rounded-2xl flex items-center justify-center font-bold text-base transition-colors ${selectedEmployee?.id === emp.id ? 'bg-blue-600 text-white' : 'bg-slate-100 text-slate-400'}`}>
                    {emp.last_name[0]}
                  </div>
                  <div className="flex-1">
                    <div className={`font-bold text-sm ${selectedEmployee?.id === emp.id ? 'text-blue-700' : 'text-slate-900'}`}>
                      {emp.first_name} {emp.last_name}
                    </div>
                    <div className="text-[10px] text-slate-400 font-black tracking-widest uppercase">{emp.employee_id_number}</div>
                  </div>
                  <ChevronRight className={`w-4 h-4 transition-transform ${selectedEmployee?.id === emp.id ? 'text-blue-400 translate-x-1' : 'text-slate-200'}`} />
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Main Content: Payslips */}
      <div className="flex-1 flex flex-col gap-6 overflow-hidden">
        {!selectedEmployee ? (
          <div className="bg-white border border-slate-200 rounded-[40px] flex-1 flex flex-col items-center justify-center text-center p-12 shadow-sm">
            <div className="bg-slate-50 p-6 rounded-[32px] mb-6">
              <Banknote className="w-12 h-12 text-slate-300" />
            </div>
            <h3 className="text-xl font-bold text-slate-900">Chọn nhân sự để quản lý lương</h3>
            <p className="text-slate-500 mt-2 max-w-xs mx-auto text-sm">
              Xem lịch sử phiếu lương hoặc tính toán bảng lương tháng mới cho nhân viên.
            </p>
          </div>
        ) : (
          <>
            <div className="bg-white p-6 rounded-[32px] border border-slate-200 shadow-sm flex items-center justify-between">
              <div className="flex items-center gap-4">
                <div className="w-14 h-14 bg-blue-600 rounded-[22px] flex items-center justify-center text-white text-xl font-black shadow-lg shadow-blue-100">
                  {selectedEmployee.last_name[0]}
                </div>
                <div>
                  <h2 className="text-xl font-bold text-slate-900">{selectedEmployee.first_name} {selectedEmployee.last_name}</h2>
                  <p className="text-[10px] font-black uppercase tracking-widest text-slate-400">Hồ sơ lương nhân sự</p>
                </div>
              </div>
              <button 
                onClick={() => setIsCalcModalOpen(true)}
                className="bg-slate-900 text-white px-6 py-3 rounded-2xl font-bold flex items-center gap-2 hover:bg-black transition-all shadow-lg shadow-slate-200 active:scale-95"
              >
                <Calculator className="w-5 h-5" />
                Tính lương mới
              </button>
            </div>

            <div className="flex-1 bg-white border border-slate-200 rounded-[40px] overflow-hidden shadow-sm flex flex-col">
              <div className="p-6 border-b border-slate-100 bg-slate-50/50 flex items-center gap-2">
                <History className="w-4 h-4 text-slate-400" />
                <h3 className="font-black text-[10px] text-slate-400 uppercase tracking-widest">Lịch sử phiếu lương (Payslips)</h3>
              </div>
              
              <div className="flex-1 overflow-auto">
                <table className="w-full text-left">
                  <thead className="bg-slate-50/30 sticky top-0 backdrop-blur-md border-b border-slate-100">
                    <tr>
                      <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">Kỳ lương</th>
                      <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">Lương cơ bản</th>
                      <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">Tăng ca</th>
                      <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">Khấu trừ</th>
                      <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">Thực nhận</th>
                      <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center">Trạng thái</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-50">
                    {payslipsLoading ? (
                      <tr><td colSpan={6} className="px-8 py-12 text-center text-slate-400 italic">Đang tải phiếu lương...</td></tr>
                    ) : payslips.length === 0 ? (
                      <tr>
                        <td colSpan={6} className="px-8 py-20 text-center">
                          <div className="bg-slate-50 w-12 h-12 rounded-2xl flex items-center justify-center mx-auto mb-4">
                            <FileText className="text-slate-200 w-6 h-6" />
                          </div>
                          <p className="text-xs font-bold text-slate-300 uppercase tracking-widest">Chưa có phiếu lương nào</p>
                        </td>
                      </tr>
                    ) : payslips.map(slip => (
                      <tr key={slip.id} className="hover:bg-slate-50/50 transition-colors group">
                        <td className="px-8 py-6">
                          <div className="font-bold text-slate-900">Tháng {new Date(slip.period_start).getMonth() + 1}/{new Date(slip.period_start).getFullYear()}</div>
                          <div className="text-[10px] text-slate-400 font-bold mt-1">
                            {new Date(slip.period_start).toLocaleDateString('vi-VN')} - {new Date(slip.period_end).toLocaleDateString('vi-VN')}
                          </div>
                        </td>
                        <td className="px-8 py-6 text-right font-mono text-sm text-slate-600">
                          {new Intl.NumberFormat('vi-VN').format(parseFloat(slip.base_pay_amount))}
                        </td>
                        <td className="px-8 py-6 text-right font-mono text-sm text-emerald-600 font-bold">
                          +{new Intl.NumberFormat('vi-VN').format(parseFloat(slip.overtime_pay_amount))}
                        </td>
                        <td className="px-8 py-6 text-right font-mono text-sm text-rose-600 font-bold">
                          -{new Intl.NumberFormat('vi-VN').format(parseFloat(slip.deductions_amount))}
                        </td>
                        <td className="px-8 py-6 text-right">
                          <div className="font-black text-blue-600 text-lg">
                            {new Intl.NumberFormat('vi-VN').format(parseFloat(slip.net_pay_amount))}
                          </div>
                        </td>
                        <td className="px-8 py-6 text-center">
                          {slip.status === 'paid' ? (
                            <span className="inline-flex items-center gap-1 px-3 py-1 bg-emerald-100 text-emerald-700 rounded-full text-[10px] font-black uppercase tracking-tighter">
                              <CheckCircle2 size={10} /> ĐÃ CHI TRẢ
                            </span>
                          ) : (
                            <span className="inline-flex items-center gap-1 px-3 py-1 bg-blue-100 text-blue-700 rounded-full text-[10px] font-black uppercase tracking-tighter">
                              <AlertCircle size={10} /> ĐÃ CHỐT
                            </span>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          </>
        )}
      </div>

      {/* Calculate Modal */}
      {isCalcModalOpen && (
        <div className="fixed inset-0 bg-slate-900/50 backdrop-blur-sm flex items-center justify-center z-[70] p-4">
          <div className="bg-white rounded-[40px] shadow-2xl w-full max-w-md overflow-hidden animate-in zoom-in duration-200">
            <div className="p-8 border-b border-slate-50 flex justify-between items-center bg-slate-50/30">
              <h2 className="text-2xl font-black text-slate-900 uppercase tracking-tight">Tính lương tháng</h2>
              <button onClick={() => setIsCalcModalOpen(false)} className="p-2 hover:bg-white rounded-2xl transition-colors">
                <X className="w-6 h-6 text-slate-300" />
              </button>
            </div>
            <div className="p-8 space-y-6">
              <div className="bg-blue-50/50 p-6 rounded-[32px] border border-blue-100 flex items-center gap-4">
                <div className="w-12 h-12 bg-white rounded-2xl flex items-center justify-center shadow-sm">
                  <UserCircle className="w-8 h-8 text-blue-600" />
                </div>
                <div>
                  <div className="font-bold text-slate-900">{selectedEmployee?.first_name} {selectedEmployee?.last_name}</div>
                  <div className="text-[10px] font-black text-blue-600 uppercase tracking-widest">{selectedEmployee?.employee_id_number}</div>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Tháng</label>
                  <select 
                    value={calcMonth}
                    onChange={(e) => setCalcMonth(parseInt(e.target.value))}
                    className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-5 py-4 font-bold text-slate-700 outline-none focus:ring-2 focus:ring-blue-500 appearance-none"
                  >
                    {Array.from({ length: 12 }, (_, i) => (
                      <option key={i + 1} value={i + 1}>Tháng {i + 1}</option>
                    ))}
                  </select>
                </div>
                <div className="space-y-2">
                  <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Năm</label>
                  <select 
                    value={calcYear}
                    onChange={(e) => setCalcYear(parseInt(e.target.value))}
                    className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-5 py-4 font-bold text-slate-700 outline-none focus:ring-2 focus:ring-blue-500 appearance-none"
                  >
                    {[2025, 2026, 2027].map(y => (
                      <option key={y} value={y}>{y}</option>
                    ))}
                  </select>
                </div>
              </div>

              <div className="bg-amber-50 p-4 rounded-2xl border border-amber-100 flex items-start gap-3">
                <AlertCircle className="w-5 h-5 text-amber-500 shrink-0 mt-0.5" />
                <p className="text-[10px] font-bold text-amber-700 leading-normal">
                  Hệ thống sẽ dựa trên dữ liệu chấm công đã chốt và mức lương cơ bản hiệu lực trong tháng {calcMonth}/{calcYear} để tính toán.
                </p>
              </div>

              <button
                onClick={() => calcMutation.mutate({ year: calcYear, month: calcMonth })}
                disabled={calcMutation.isPending}
                className="w-full bg-blue-600 text-white font-black py-5 rounded-[28px] flex items-center justify-center gap-3 hover:bg-blue-700 transition-all active:scale-[0.98] shadow-xl shadow-blue-100 disabled:opacity-50 uppercase tracking-widest text-xs"
              >
                {calcMutation.isPending ? "Đang xử lý..." : (
                  <>
                    <Calculator className="w-5 h-5" />
                    Bắt đầu tính lương
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
