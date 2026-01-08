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

export default function TimeTrackingPage() {
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const [selectedEmployeeId, setSelectedEmployeeId] = useState<string>("");
  const [isAdjustmentModalOpen, setIsAdjustmentModalOpen] = useState(false);
  const [adjustmentData, setAdjustmentData] = useState({
    date: new Date().toISOString().split('T')[0],
    punchIn: "08:00",
    punchOut: "17:00",
    reason: "",
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
      queryClient.invalidateQueries({ queryKey: ["time-history", wsId, selectedEmployeeId] });
    },
  });

  const punchOutMutation = useMutation({
    mutationFn: (data: { note?: string }) => punchOut(wsId!, selectedEmployeeId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["time-history", wsId, selectedEmployeeId] });
    },
  });

  const adjustMutation = useMutation({
    mutationFn: (data: any) => submitAttendanceAdjustment(wsId!, data),
    onSuccess: () => {
      setIsAdjustmentModalOpen(false);
      alert("Yêu cầu đã được gửi!");
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
      reason: adjustmentData.reason
    });
  };

  if (!wsId) return <div className="p-8 text-center">Vui lòng chọn Workspace</div>;

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Chấm công</h1>
        <p className="text-slate-500">Ghi nhận giờ làm việc cho nhân viên.</p>
      </div>

      <div className="bg-white border border-slate-200 rounded-3xl p-6 shadow-sm">
        <label className="block text-sm font-semibold text-slate-700 mb-2">Chọn nhân viên</label>
        <select 
          value={selectedEmployeeId}
          onChange={(e) => setSelectedEmployeeId(e.target.value)}
          className="w-full max-w-md px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl focus:ring-2 focus:ring-blue-500/20 outline-none transition-all"
        >
          <option value="">-- Chọn nhân viên --</option>
          {employees.map(emp => (
            <option key={emp.id} value={emp.id}>{emp.first_name} {emp.last_name} ({emp.employee_id_number})</option>
          ))}
        </select>
      </div>

      {selectedEmployee && (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* Punch Card */}
          <div className="md:col-span-1 bg-white border border-slate-200 rounded-3xl p-8 shadow-sm flex flex-col items-center justify-center text-center space-y-6">
            <div className={`p-4 rounded-full ${activeEntry ? 'bg-orange-100 text-orange-600 animate-pulse' : 'bg-green-100 text-green-600'}`}>
              <Clock className="w-12 h-12" />
            </div>
            <div>
              <h2 className="text-xl font-bold">{selectedEmployee.first_name} {selectedEmployee.last_name}</h2>
              <p className="text-slate-500 text-sm">{activeEntry ? 'Đang trong ca làm việc' : 'Sẵn sàng chấm công'}</p>
            </div>

            {activeEntry ? (
              <div className="w-full space-y-4">
                <div className="text-sm bg-slate-50 p-3 rounded-2xl border border-slate-100">
                  <div className="text-slate-400 font-medium">Bắt đầu lúc</div>
                  <div className="font-bold text-lg">{new Date(activeEntry.punch_in_time).toLocaleTimeString('vi-VN')}</div>
                </div>
                <button 
                  onClick={() => punchOutMutation.mutate({})}
                  disabled={punchOutMutation.isPending}
                  className="w-full bg-red-600 text-white py-4 rounded-2xl font-bold hover:bg-red-700 transition-all shadow-lg shadow-red-200"
                >
                  Kết thúc ca (Punch Out)
                </button>
              </div>
            ) : (
              <button 
                onClick={() => punchInMutation.mutate({ employee_id: selectedEmployeeId })}
                disabled={punchInMutation.isPending}
                className="w-full bg-blue-600 text-white py-4 rounded-2xl font-bold hover:bg-blue-700 transition-all shadow-lg shadow-blue-200"
              >
                Bắt đầu ca (Punch In)
              </button>
            )}

            <button 
              onClick={() => {
                setAdjustmentData({ ...adjustmentData, timeEntryId: undefined });
                setIsAdjustmentModalOpen(true);
              }}
              className="text-blue-600 text-sm font-medium hover:underline"
            >
              Quên chấm công? Gửi xác nhận
            </button>
          </div>

          {/* History */}
          <div className="md:col-span-2 bg-white border border-slate-200 rounded-3xl overflow-hidden shadow-sm flex flex-col">
            <div className="px-6 py-4 border-b border-slate-100 bg-slate-50/50 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <History className="w-5 h-5 text-slate-400" />
                <h3 className="font-bold">Lịch sử gần đây (7 ngày)</h3>
              </div>
            </div>
            <div className="flex-1 overflow-y-auto max-h-[400px]">
              <table className="w-full text-left">
                <thead className="sticky top-0 bg-white border-b border-slate-100">
                  <tr>
                    <th className="px-6 py-3 text-xs font-bold text-slate-400 uppercase">Ngày</th>
                    <th className="px-6 py-3 text-xs font-bold text-slate-400 uppercase">Vào</th>
                    <th className="px-6 py-3 text-xs font-bold text-slate-400 uppercase">Ra</th>
                    <th className="px-6 py-3 text-xs font-bold text-slate-400 uppercase text-right">Thao tác</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-50">
                  {isLoadingHistory ? (
                    <tr><td colSpan={4} className="px-6 py-8 text-center text-slate-400 italic">Đang tải...</td></tr>
                  ) : history.length === 0 ? (
                    <tr><td colSpan={4} className="px-6 py-8 text-center text-slate-400 italic">Chưa có dữ liệu.</td></tr>
                  ) : history.map(entry => (
                    <tr key={entry.id} className="hover:bg-slate-50/50 transition-colors">
                      <td className="px-6 py-4 text-sm font-medium">{new Date(entry.punch_in_time).toLocaleDateString('vi-VN')}</td>
                      <td className="px-6 py-4 text-sm text-slate-600">{new Date(entry.punch_in_time).toLocaleTimeString('vi-VN')}</td>
                      <td className="px-6 py-4 text-sm text-slate-600">
                        {entry.punch_out_time ? new Date(entry.punch_out_time).toLocaleTimeString('vi-VN') : (
                          <span className="text-orange-500 font-medium">Đang làm...</span>
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
                              timeEntryId: entry.id
                            });
                            setIsAdjustmentModalOpen(true);
                          }}
                          className="text-blue-600 hover:text-blue-800 font-medium"
                        >
                          Sửa
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
        <div className="fixed inset-0 bg-slate-900/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-3xl p-8 max-w-md w-full shadow-2xl">
            <h2 className="text-xl font-bold mb-6">Yêu cầu xác nhận công</h2>
            <form onSubmit={handleAdjustSubmit} className="space-y-4">
              <div>
                <label className="block text-sm font-bold text-slate-700 mb-1">Ngày</label>
                <input 
                  type="date" 
                  value={adjustmentData.date}
                  onChange={e => setAdjustmentData({...adjustmentData, date: e.target.value})}
                  className="w-full px-4 py-2 border border-slate-200 rounded-xl" 
                  required
                />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-bold text-slate-700 mb-1">Giờ vào</label>
                  <input 
                    type="time" 
                    value={adjustmentData.punchIn}
                    onChange={e => setAdjustmentData({...adjustmentData, punchIn: e.target.value})}
                    className="w-full px-4 py-2 border border-slate-200 rounded-xl" 
                    required
                  />
                </div>
                <div>
                  <label className="block text-sm font-bold text-slate-700 mb-1">Giờ ra</label>
                  <input 
                    type="time" 
                    value={adjustmentData.punchOut}
                    onChange={e => setAdjustmentData({...adjustmentData, punchOut: e.target.value})}
                    className="w-full px-4 py-2 border border-slate-200 rounded-xl" 
                    required
                  />
                </div>
              </div>
              <div>
                <label className="block text-sm font-bold text-slate-700 mb-1">Lý do</label>
                <textarea 
                  value={adjustmentData.reason}
                  onChange={e => setAdjustmentData({...adjustmentData, reason: e.target.value})}
                  className="w-full px-4 py-2 border border-slate-200 rounded-xl min-h-[100px]" 
                  placeholder="VD: Quên mang thẻ, hệ thống lỗi..."
                  required
                ></textarea>
              </div>
              <div className="flex gap-4 pt-4">
                <button 
                  type="button"
                  onClick={() => setIsAdjustmentModalOpen(false)}
                  className="flex-1 px-4 py-3 border border-slate-200 rounded-2xl font-bold hover:bg-slate-50 transition-all"
                >
                  Hủy
                </button>
                <button 
                  type="submit"
                  disabled={adjustMutation.isPending}
                  className="flex-1 px-4 py-3 bg-blue-600 text-white rounded-2xl font-bold hover:bg-blue-700 transition-all"
                >
                  Gửi yêu cầu
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {!selectedEmployeeId && (
        <div className="flex flex-col items-center justify-center py-20 text-slate-400 border-2 border-dashed border-slate-200 rounded-3xl">
          <AlertCircle className="w-12 h-12 mb-4 opacity-20" />
          <p className="font-medium italic">Chọn một nhân viên để bắt đầu chấm công</p>
        </div>
      )}
    </div>
  );
}
