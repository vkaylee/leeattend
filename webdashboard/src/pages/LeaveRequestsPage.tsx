import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { CalendarDays, CheckCircle2, XCircle, Send, Plus, X } from "lucide-react";
import { 
  fetchEmployees, 
  submitLeaveRequest, 
  fetchWorkspaceLeaveRequests, 
  approveLeaveRequest,
  getCurrentWorkspaceId,
  Employee,
  LeaveRequest
} from "../lib/api";

export default function LeaveRequestsPage() {
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<string>("pending");

  const { data: employees = [] } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId!),
    enabled: !!wsId,
  });

  const { data: requests = [], isLoading } = useQuery({
    queryKey: ["leave-requests", wsId, activeTab],
    queryFn: () => fetchWorkspaceLeaveRequests(wsId!, activeTab === "all" ? undefined : activeTab),
    enabled: !!wsId,
  });

  const submitMutation = useMutation({
    mutationFn: (data: { employee_id: string; leave_type: string; start_date: string; end_date: string; reason?: string }) => 
      submitLeaveRequest(wsId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-requests", wsId] });
      setIsModalOpen(false);
    },
  });

  const approveMutation = useMutation({
    mutationFn: (data: { id: string; status: string }) => 
      approveLeaveRequest(wsId!, data.id, data.status),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-requests", wsId] });
    },
  });

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const data = {
      employee_id: formData.get("employee_id") as string,
      leave_type: formData.get("leave_type") as string,
      start_date: formData.get("start_date") as string,
      end_date: formData.get("end_date") as string,
      reason: formData.get("reason") as string,
    };
    submitMutation.mutate(data);
  };

  const getStatusStyle = (status: string) => {
    switch (status) {
      case "pending": return "bg-orange-100 text-orange-700";
      case "approved": return "bg-green-100 text-green-700";
      case "rejected": return "bg-red-100 text-red-700";
      default: return "bg-slate-100 text-slate-700";
    }
  };

  if (!wsId) return <div className="p-8 text-center">Vui lòng chọn Workspace</div>;

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Quản lý nghỉ phép</h1>
          <p className="text-slate-500">Duyệt và theo dõi các yêu cầu nghỉ phép của nhân viên.</p>
        </div>
        <button 
          onClick={() => setIsModalOpen(true)}
          className="flex items-center gap-2 bg-blue-600 text-white px-4 py-2 rounded-xl hover:bg-blue-700 transition-colors shadow-lg shadow-blue-200"
        >
          <Plus className="w-5 h-5" />
          <span className="font-semibold">Tạo yêu cầu</span>
        </button>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 p-1 bg-slate-100 w-fit rounded-2xl">
        {['pending', 'approved', 'rejected', 'all'].map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-6 py-2 rounded-xl font-bold text-sm transition-all ${activeTab === tab ? 'bg-white text-blue-600 shadow-sm' : 'text-slate-500 hover:text-slate-700'}`}
          >
            {tab === 'pending' ? 'Chờ duyệt' : tab === 'approved' ? 'Đã duyệt' : tab === 'rejected' ? 'Từ chối' : 'Tất cả'}
          </button>
        ))}
      </div>

      <div className="bg-white border border-slate-200 rounded-3xl overflow-hidden shadow-sm">
        <table className="w-full text-left">
          <thead className="bg-slate-50 border-b border-slate-200">
            <tr>
              <th className="px-6 py-4 text-sm font-semibold text-slate-600">Nhân viên</th>
              <th className="px-6 py-4 text-sm font-semibold text-slate-600">Loại</th>
              <th className="px-6 py-4 text-sm font-semibold text-slate-600">Thời gian</th>
              <th className="px-6 py-4 text-sm font-semibold text-slate-600">Lý do</th>
              <th className="px-6 py-4 text-sm font-semibold text-slate-600">Trạng thái</th>
              <th className="px-6 py-4 text-sm font-semibold text-slate-600 text-right">Thao tác</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-100">
            {isLoading ? (
              <tr><td colSpan={6} className="px-6 py-8 text-center text-slate-500">Đang tải...</td></tr>
            ) : requests.length === 0 ? (
              <tr><td colSpan={6} className="px-6 py-8 text-center text-slate-500 italic">Không có yêu cầu nào.</td></tr>
            ) : requests.map((req) => {
              const emp = employees.find(e => e.id === req.employee_id);
              return (
                <tr key={req.id} className="hover:bg-slate-50/50 transition-colors">
                  <td className="px-6 py-4">
                    <div className="font-medium text-slate-900">{emp ? `${emp.first_name} ${emp.last_name}` : 'N/A'}</div>
                  </td>
                  <td className="px-6 py-4 capitalize text-slate-600">{req.leave_type}</td>
                  <td className="px-6 py-4 text-sm text-slate-600">
                    {new Date(req.start_date).toLocaleDateString('vi-VN')} - {new Date(req.end_date).toLocaleDateString('vi-VN')}
                  </td>
                  <td className="px-6 py-4 text-sm text-slate-500 max-w-xs truncate">{req.reason || '-'}</td>
                  <td className="px-6 py-4">
                    <span className={`px-2 py-1 rounded-full text-xs font-bold ${getStatusStyle(req.status)}`}>
                      {req.status === 'pending' ? 'Chờ duyệt' : req.status === 'approved' ? 'Đã duyệt' : 'Từ chối'}
                    </span>
                  </td>
                  <td className="px-6 py-4 text-right">
                    {req.status === 'pending' && (
                      <div className="flex justify-end gap-2">
                        <button 
                          onClick={() => approveMutation.mutate({ id: req.id, status: 'approved' })}
                          className="p-2 text-green-600 hover:bg-green-50 rounded-lg transition-all"
                          title="Duyệt"
                        >
                          <CheckCircle2 className="w-5 h-5" />
                        </button>
                        <button 
                          onClick={() => approveMutation.mutate({ id: req.id, status: 'rejected' })}
                          className="p-2 text-red-600 hover:bg-red-50 rounded-lg transition-all"
                          title="Từ chối"
                        >
                          <XCircle className="w-5 h-5" />
                        </button>
                      </div>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {/* Submit Modal */}
      {isModalOpen && (
        <div className="fixed inset-0 bg-slate-900/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-3xl shadow-2xl w-full max-w-md overflow-hidden animate-in fade-in zoom-in duration-200">
            <div className="px-6 py-4 border-b border-slate-100 flex justify-between items-center bg-slate-50/50">
              <h2 className="text-xl font-bold">Gửi yêu cầu nghỉ phép</h2>
              <button onClick={() => setIsModalOpen(false)} className="p-2 hover:bg-white rounded-xl transition-colors">
                <X className="w-5 h-5 text-slate-400" />
              </button>
            </div>
            <form onSubmit={handleSubmit} className="p-6 space-y-4">
              <div className="space-y-1">
                <label className="text-sm font-semibold text-slate-700">Nhân viên</label>
                <select name="employee_id" required className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none">
                  <option value="">-- Chọn nhân viên --</option>
                  {employees.map(e => <option key={e.id} value={e.id}>{e.first_name} {e.last_name}</option>)}
                </select>
              </div>
              <div className="space-y-1">
                <label className="text-sm font-semibold text-slate-700">Loại nghỉ</label>
                <select name="leave_type" required className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none">
                  <option value="vacation">Nghỉ phép năm</option>
                  <option value="sick">Nghỉ ốm</option>
                  <option value="personal">Việc riêng</option>
                </select>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1">
                  <label className="text-sm font-semibold text-slate-700">Từ ngày</label>
                  <input type="date" name="start_date" required className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none" />
                </div>
                <div className="space-y-1">
                  <label className="text-sm font-semibold text-slate-700">Đến ngày</label>
                  <input type="date" name="end_date" required className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none" />
                </div>
              </div>
              <div className="space-y-1">
                <label className="text-sm font-semibold text-slate-700">Lý do</label>
                <textarea name="reason" rows={3} className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none" placeholder="Nhập lý do nghỉ..."></textarea>
              </div>
              <div className="pt-4 flex gap-3">
                <button type="button" onClick={() => setIsModalOpen(false)} className="flex-1 px-4 py-2 border border-slate-200 rounded-xl font-semibold hover:bg-slate-50 transition-colors">Hủy</button>
                <button type="submit" className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-xl font-semibold hover:bg-blue-700 transition-colors flex items-center justify-center gap-2 shadow-lg shadow-blue-200">
                  <Send className="w-4 h-4" />
                  <span>Gửi yêu cầu</span>
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
