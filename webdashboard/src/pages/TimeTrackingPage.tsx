import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Clock, ArrowRightLeft, History, AlertCircle } from "lucide-react";
import { 
  fetchEmployees, 
  punchIn, 
  punchOut, 
  fetchTimeHistory, 
  getCurrentWorkspaceId,
  submitAttendanceAdjustment,
  Employee,
  TimeEntry
} from "../lib/api";
import { useToast } from "../components/Toast";

export default function TimeTrackingPage() {
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const toast = useToast();
  const [selectedEmployeeId, setSelectedEmployeeId] = useState<string>("");
  const [isAdjustmentModalOpen, setIsAdjustmentModalOpen] = useState(false);
  const [adjustmentData, setAdjustmentData] = useState({
    date: new Date().toISOString().split('T')[0],
    punchIn: "08:00",
    punchOut: "17:00",
    reason: "",
    isBreakOvertime: false,
    timeEntryId: undefined as string | undefined
  });

  const { data: employees = [] } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId!),
    enabled: !!wsId,
  });

  const selectedEmployee = employees.find(e => e.id === selectedEmployeeId);

  // Get last 7 days history
  const endDate = new Date().toISOString();
  const startDate = new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString();

  const { data: history = [], isLoading: isLoadingHistory } = useQuery({
    queryKey: ["time-history", wsId, selectedEmployeeId],
    queryFn: () => fetchTimeHistory(wsId!, selectedEmployeeId, startDate, endDate),
    enabled: !!wsId && !!selectedEmployeeId,
  });

  const activeEntry = history.find(entry => !entry.punch_out_time);

  const punchInMutation = useMutation({
    mutationFn: (data: { employee_id: string; note?: string }) => punchIn(wsId!, data),
    onSuccess: () => {
      toast.success("Bắt đầu ca làm việc thành công.");
      queryClient.invalidateQueries({ queryKey: ["time-history", wsId, selectedEmployeeId] });
    },
    onError: () => {
      toast.error("Không thể chấm công vào. Vui lòng thử lại.");
    }
  });

  const punchOutMutation = useMutation({
    mutationFn: (data: { note?: string }) => punchOut(wsId!, selectedEmployeeId, data),
    onSuccess: () => {
      toast.success("Đã kết thúc ca làm việc.");
      queryClient.invalidateQueries({ queryKey: ["time-history", wsId, selectedEmployeeId] });
    },
    onError: () => {
      toast.error("Không thể chấm công ra.");
    }
  });

  const adjustMutation = useMutation({
    mutationFn: (data: any) => submitAttendanceAdjustment(wsId!, data),
    onSuccess: () => {
      setIsAdjustmentModalOpen(false);
      toast.success("Yêu cầu xác nhận công đã được gửi tới quản trị viên.");
    },
    onError: (error: any) => {
      toast.error(error.message || "Lỗi khi gửi yêu cầu.");
    }
  });

  const handleAdjustSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const punchInTime = new Date(`${adjustmentData.date}T${adjustmentData.punchIn}:00Z`).toISOString();
    const punchOutTime = new Date(`${adjustmentData.date}T${adjustmentData.punchOut}:00Z`).toISOString();
    
    adjustMutation.mutate({
      employee_id: selectedEmployeeId,
      time_entry_id: adjustmentData.timeEntryId,
      requested_punch_in_time: punchInTime,
      requested_punch_out_time: punchOutTime,
      is_break_overtime: adjustmentData.isBreakOvertime,
      reason: adjustmentData.reason
    });
  };

  if (!wsId) return <div className="p-8 text-center font-bold text-slate-400 italic">Vui lòng chọn Workspace</div>;

  return (
    <div className="space-y-6 animate-in fade-in duration-500">
      <div>
        <h1 className="text-2xl font-bold tracking-tight text-slate-900">Chấm công</h1>
        <p className="text-slate-500">Ghi nhận giờ làm việc cho nhân viên.</p>
      </div>

      <div className="bg-white border border-slate-200 rounded-3xl p-6 shadow-sm">
        <label className="block text-sm font-semibold text-slate-700 mb-2 ml-1">Chọn nhân viên</label>
        <select 
          value={selectedEmployeeId}
          onChange={(e) => setSelectedEmployeeId(e.target.value)}
          className="w-full max-w-md px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl focus:ring-2 focus:ring-blue-500/20 outline-none transition-all font-bold text-slate-700 appearance-none"
        >
          <option value="">-- Chọn nhân viên để chấm công --</option>
          {employees.map(emp => (
            <option key={emp.id} value={emp.id}>{emp.first_name} {emp.last_name} ({emp.employee_id_number})</option>
          ))}
        </select>
      </div>

      {selectedEmployee && (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* Punch Card */}
          <div className="md:col-span-1 bg-white border border-slate-200 rounded-3xl p-8 shadow-sm flex flex-col items-center justify-center text-center space-y-6">
            <div className={`p-4 rounded-full transition-colors duration-500 ${activeEntry ? 'bg-orange-100 text-orange-600 animate-pulse' : 'bg-green-100 text-green-600'}`}>
              <Clock className="w-12 h-12" />
            </div>
            <div>
              <h2 className="text-xl font-bold text-slate-900">{selectedEmployee.first_name} {selectedEmployee.last_name}</h2>
              <p className="text-slate-500 text-sm font-medium">{activeEntry ? 'Đang trong ca làm việc' : 'Sẵn sàng chấm công'}</p>
            </div>

            {activeEntry ? (
              <div className="w-full space-y-4">
                <div className="text-sm bg-slate-50 p-4 rounded-2xl border border-slate-100">
                  <div className="text-slate-400 font-black uppercase text-[10px] tracking-widest mb-1">Bắt đầu lúc</div>
                  <div className="font-black text-2xl text-slate-800">{new Date(activeEntry.punch_in_time).toLocaleTimeString('vi-VN')}</div>
                </div>
                <button 
                  onClick={() => punchOutMutation.mutate({})}
                  disabled={punchOutMutation.isPending}
                  className="w-full bg-red-600 text-white py-4 rounded-2xl font-black uppercase tracking-widest text-xs hover:bg-red-700 transition-all shadow-lg shadow-red-100 disabled:opacity-50"
                >
                  Kết thúc ca (Punch Out)
                </button>
              </div>
            ) : (
              <button 
                onClick={() => punchInMutation.mutate({ employee_id: selectedEmployeeId })}
                disabled={punchInMutation.isPending}
                className="w-full bg-blue-600 text-white py-4 rounded-2xl font-black uppercase tracking-widest text-xs hover:bg-blue-700 transition-all shadow-lg shadow-blue-100 disabled:opacity-50"
              >
                Bắt đầu ca (Punch In)
              </button>
            )}

            <button 
              onClick={() => {
                setAdjustmentData({ ...adjustmentData, timeEntryId: undefined });
                setIsAdjustmentModalOpen(true);
              }}
              className="text-blue-600 text-xs font-black uppercase tracking-widest hover:underline"
            >
              Quên chấm công? Gửi xác nhận
            </button>
          </div>

          {/* History */}
          <div className="md:col-span-2 bg-white border border-slate-200 rounded-3xl overflow-hidden shadow-sm flex flex-col">
            <div className="px-6 py-4 border-b border-slate-100 bg-slate-50/50 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <History className="w-5 h-5 text-slate-400" />
                <h3 className="font-bold text-slate-700">Lịch sử gần đây (7 ngày)</h3>
              </div>
            </div>
            <div className="flex-1 overflow-y-auto max-h-[400px]">
              <table className="w-full text-left border-collapse">
                <thead className="sticky top-0 bg-white border-b border-slate-100">
                  <tr>
                    <th className="px-6 py-3 text-[10px] font-black text-slate-400 uppercase tracking-widest">Ngày</th>
                    <th className="px-6 py-3 text-[10px] font-black text-slate-400 uppercase tracking-widest">Vào</th>
                    <th className="px-6 py-3 text-[10px] font-black text-slate-400 uppercase tracking-widest">Ra</th>
                    <th className="px-6 py-3 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">Thao tác</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-50">
                  {isLoadingHistory ? (
                    <tr><td colSpan={4} className="px-6 py-8 text-center text-slate-400 italic font-medium">Đang tải lịch sử...</td></tr>
                  ) : history.length === 0 ? (
                    <tr><td colSpan={4} className="px-6 py-8 text-center text-slate-400 italic font-medium">Chưa có dữ liệu chấm công.</td></tr>
                  ) : history.map(entry => (
                    <tr key={entry.id} className="hover:bg-slate-50/50 transition-colors">
                      <td className="px-6 py-4 text-sm font-bold text-slate-700">{new Date(entry.punch_in_time).toLocaleDateString('vi-VN')}</td>
                      <td className="px-6 py-4 text-sm font-medium text-slate-600">{new Date(entry.punch_in_time).toLocaleTimeString('vi-VN')}</td>
                      <td className="px-6 py-4 text-sm font-medium text-slate-600">
                        {entry.punch_out_time ? new Date(entry.punch_out_time).toLocaleTimeString('vi-VN') : (
                          <span className="text-orange-500 font-bold bg-orange-50 px-2 py-0.5 rounded-lg text-[10px] uppercase tracking-widest">Đang làm</span>
                        )}
                      </td>
                      <td className="px-6 py-4 text-sm text-right">
                        <button 
                          onClick={() => {
                            setAdjustmentData({
                              date: new Date(entry.punch_in_time).toISOString().split('T')[0],
                              punchIn: new Date(entry.punch_in_time).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' }),
                              punchOut: entry.punch_out_time ? new Date(entry.punch_out_time).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' }) : "17:00",
                              reason: "",
                              isBreakOvertime: entry.is_break_overtime,
                              timeEntryId: entry.id
                            });
                            setIsAdjustmentModalOpen(true);
                          }}
                          className="text-blue-600 hover:text-blue-800 font-black text-[10px] uppercase tracking-widest"
                        >
                          Hiệu chỉnh
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {/* Adjustment Modal */}
      {isAdjustmentModalOpen && (
        <div className="fixed inset-0 bg-slate-900/40 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in duration-200">
          <div className="bg-white rounded-[40px] p-8 max-w-md w-full shadow-2xl overflow-hidden animate-in zoom-in-95 duration-200 border border-slate-100">
            <h2 className="text-2xl font-black text-slate-900 uppercase tracking-tight mb-1">Xác nhận công</h2>
            <p className="text-xs font-bold text-slate-400 mb-8 uppercase tracking-widest">GỬI YÊU CẦU ĐIỀU CHỈNH GIỜ CÔNG</p>
            
            <form onSubmit={handleAdjustSubmit} className="space-y-6">
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Ngày làm việc</label>
                <input 
                  type="date" 
                  value={adjustmentData.date}
                  onChange={e => setAdjustmentData({...adjustmentData, date: e.target.value})}
                  className="w-full px-5 py-4 bg-slate-50 border border-slate-200 rounded-2xl font-bold text-slate-700 focus:ring-2 focus:ring-blue-500 outline-none transition-all" 
                  required
                />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Giờ vào</label>
                  <input 
                    type="time" 
                    value={adjustmentData.punchIn}
                    onChange={e => setAdjustmentData({...adjustmentData, punchIn: e.target.value})}
                    className="w-full px-5 py-4 bg-slate-50 border border-slate-200 rounded-2xl font-bold text-slate-700 focus:ring-2 focus:ring-blue-500 outline-none transition-all" 
                    required
                  />
                </div>
                <div className="space-y-2">
                  <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Giờ ra</label>
                  <input 
                    type="time" 
                    value={adjustmentData.punchOut}
                    onChange={e => setAdjustmentData({...adjustmentData, punchOut: e.target.value})}
                    className="w-full px-5 py-4 bg-slate-50 border border-slate-200 rounded-2xl font-bold text-slate-700 focus:ring-2 focus:ring-blue-500 outline-none transition-all" 
                    required
                  />
                </div>
              </div>
              <div className="flex items-center gap-3 p-4 bg-blue-50 border border-blue-100 rounded-2xl">
                <input 
                  type="checkbox" 
                  id="isBreakOvertime"
                  checked={adjustmentData.isBreakOvertime}
                  onChange={e => setAdjustmentData({...adjustmentData, isBreakOvertime: e.target.checked})}
                  className="w-5 h-5 rounded-lg border-slate-300 text-blue-600 focus:ring-blue-500 transition-all cursor-pointer"
                />
                <label htmlFor="isBreakOvertime" className="text-[11px] font-black text-blue-900 cursor-pointer uppercase tracking-tight">
                  Làm việc thông trưa (Tăng ca nghỉ)
                </label>
              </div>
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Lý do điều chỉnh</label>
                <textarea 
                  value={adjustmentData.reason}
                  onChange={e => setAdjustmentData({...adjustmentData, reason: e.target.value})}
                  className="w-full px-5 py-4 bg-slate-50 border border-slate-200 rounded-2xl font-medium text-slate-700 focus:ring-2 focus:ring-blue-500 outline-none transition-all min-h-[100px]" 
                  placeholder="VD: Quên mang thẻ, máy chấm công lỗi..."
                  required
                ></textarea>
              </div>
              <div className="flex gap-4 pt-4">
                <button 
                  type="button"
                  onClick={() => setIsAdjustmentModalOpen(false)}
                  className="flex-1 px-4 py-4 border border-slate-200 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-400 hover:bg-slate-50 transition-all"
                >
                  Hủy
                </button>
                <button 
                  type="submit"
                  disabled={adjustMutation.isPending}
                  className="flex-1 px-4 py-4 bg-blue-600 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-blue-700 transition-all shadow-lg shadow-blue-100 disabled:opacity-50"
                >
                  Gửi yêu cầu
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {!selectedEmployeeId && (
        <div className="flex flex-col items-center justify-center py-32 text-slate-400 border-2 border-dashed border-slate-200 rounded-[40px] bg-slate-50/30">
          <AlertCircle className="w-16 h-16 mb-6 opacity-10" />
          <p className="font-black text-xs uppercase tracking-widest text-slate-300">Chọn nhân sự để bắt đầu chấm công</p>
        </div>
      )}
    </div>
  );
}