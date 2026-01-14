import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { CalendarDays, CheckCircle2, XCircle, Send, Plus, X, Settings2, PlayCircle, UserPlus } from "lucide-react";
import { 
  fetchEmployees, 
  submitLeaveRequest, 
  fetchWorkspaceLeaveRequests, 
  approveLeaveRequest,
  fetchEmployeeLeaveBalances,
  fetchLeavePolicies,
  createLeavePolicy,
  assignLeavePolicy,
  triggerAccrualRun,
  getCurrentWorkspaceId,
  Employee,
  LeaveRequest,
  EmployeeLeaveBalance,
  LeavePolicy
} from "../lib/api";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

export default function LeaveRequestsPage() {
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const { formatDate } = useDateTimeFormatter();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isPolicyModalOpen, setIsPolicyModalOpen] = useState(false);
  const [isAssignModalOpen, setIsAssignModalOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<string>("pending");
  const [selectedEmpId, setSelectedEmpId] = useState<string>("");

  const { data: employees = [] } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId!),
    enabled: !!wsId,
  });

  const { data: balances = [], isLoading: balancesLoading } = useQuery({
    queryKey: ["leave-balances", wsId, selectedEmpId],
    queryFn: () => fetchEmployeeLeaveBalances(wsId!, selectedEmpId),
    enabled: !!wsId && !!selectedEmpId,
  });

  const { data: requests = [], isLoading } = useQuery({
    queryKey: ["leave-requests", wsId, activeTab],
    queryFn: () => fetchWorkspaceLeaveRequests(wsId!, activeTab === "all" ? undefined : activeTab),
    enabled: !!wsId && activeTab !== "policies",
  });

  const { data: policies = [] } = useQuery({
    queryKey: ["leave-policies", wsId],
    queryFn: () => fetchLeavePolicies(wsId!),
    enabled: !!wsId && activeTab === "policies",
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
      approveLeaveRequest(wsId!, data.id, { status: data.status }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-requests", wsId] });
    },
  });

  const policyMutation = useMutation({
    mutationFn: (data: Partial<LeavePolicy>) => createLeavePolicy(wsId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-policies", wsId] });
      setIsPolicyModalOpen(false);
    },
  });

  const assignMutation = useMutation({
    mutationFn: (data: { employee_id: string; leave_policy_id: string; effective_date: string }) => 
      assignLeavePolicy(wsId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-balances", wsId] });
      setIsAssignModalOpen(false);
    },
  });

  const accrualMutation = useMutation({
    mutationFn: (date: string) => triggerAccrualRun(wsId!, { date }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-balances", wsId] });
      alert("Đã chạy tích lũy phép thành công!");
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

  const handleCreatePolicy = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    policyMutation.mutate({
      name: formData.get("name") as string,
      leave_type: formData.get("leave_type") as string,
      annual_entitlement: Number(formData.get("annual_entitlement")),
      accrual_strategy: formData.get("accrual_strategy") as "monthly" | "lump_sum",
      carry_over_limit: Number(formData.get("carry_over_limit")),
    });
  };

  const handleAssignPolicy = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    assignMutation.mutate({
      employee_id: formData.get("employee_id") as string,
      leave_policy_id: formData.get("leave_policy_id") as string,
      effective_date: formData.get("effective_date") as string,
    });
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
        <div className="flex gap-2">
          {activeTab === "policies" ? (
            <>
              <button 
                onClick={() => accrualMutation.mutate(new Date().toISOString().split('T')[0])}
                disabled={accrualMutation.isPending}
                className="flex items-center gap-2 bg-emerald-600 text-white px-4 py-2 rounded-xl hover:bg-emerald-700 transition-colors shadow-lg shadow-emerald-200 disabled:opacity-50"
              >
                <PlayCircle className="w-5 h-5" />
                <span className="font-semibold">Chạy tích lũy</span>
              </button>
              <button 
                onClick={() => setIsAssignModalOpen(true)}
                className="flex items-center gap-2 bg-slate-800 text-white px-4 py-2 rounded-xl hover:bg-slate-900 transition-colors shadow-lg shadow-slate-200"
              >
                <UserPlus className="w-5 h-5" />
                <span className="font-semibold">Gán chính sách</span>
              </button>
              <button 
                onClick={() => setIsPolicyModalOpen(true)}
                className="flex items-center gap-2 bg-blue-600 text-white px-4 py-2 rounded-xl hover:bg-blue-700 transition-colors shadow-lg shadow-blue-200"
              >
                <Plus className="w-5 h-5" />
                <span className="font-semibold">Thêm chính sách</span>
              </button>
            </>
          ) : (
            <button 
              onClick={() => setIsModalOpen(true)}
              className="flex items-center gap-2 bg-blue-600 text-white px-4 py-2 rounded-xl hover:bg-blue-700 transition-colors shadow-lg shadow-blue-200"
            >
              <Plus className="w-5 h-5" />
              <span className="font-semibold">Tạo yêu cầu</span>
            </button>
          )}
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 p-1 bg-slate-100 w-fit rounded-2xl">
        {['pending', 'approved', 'rejected', 'all', 'policies'].map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-6 py-2 rounded-xl font-bold text-sm transition-all ${activeTab === tab ? 'bg-white text-blue-600 shadow-sm' : 'text-slate-500 hover:text-slate-700'}`}
          >
            {tab === 'pending' ? 'Chờ duyệt' : tab === 'approved' ? 'Đã duyệt' : tab === 'rejected' ? 'Từ chối' : tab === 'all' ? 'Tất cả' : 'Chính sách'}
          </button>
        ))}
      </div>

      {activeTab === "policies" ? (
        <div className="bg-white border border-slate-200 rounded-3xl overflow-hidden shadow-sm">
          <table className="w-full text-left">
            <thead className="bg-slate-50 border-b border-slate-200">
              <tr>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600">Tên chính sách</th>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600">Loại nghỉ</th>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600">Định mức/năm</th>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600">Chiến lược</th>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600">Carry-over</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {policies.length === 0 ? (
                <tr><td colSpan={5} className="px-6 py-8 text-center text-slate-500 italic">Chưa có chính sách nào.</td></tr>
              ) : policies.map((policy) => (
                <tr key={policy.id} className="hover:bg-slate-50/50 transition-colors">
                  <td className="px-6 py-4 font-medium text-slate-900">{policy.name}</td>
                  <td className="px-6 py-4 capitalize text-slate-600">{policy.leave_type}</td>
                  <td className="px-6 py-4 text-slate-600 font-bold">{policy.annual_entitlement} ngày</td>
                  <td className="px-6 py-4 text-sm text-slate-600">
                    {policy.accrual_strategy === 'monthly' ? 'Tích lũy hàng tháng' : 'Cấp một lần đầu năm'}
                  </td>
                  <td className="px-6 py-4 text-slate-600">{policy.carry_over_limit} ngày</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
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
                      {formatDate(req.start_date)} - {formatDate(req.end_date)}
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
      )}

      {/* Modals remain below */}
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
                <label htmlFor="leave-emp-select" className="text-sm font-semibold text-slate-700">Nhân viên</label>
                <select 
                  id="leave-emp-select"
                  name="employee_id" 
                  required 
                  value={selectedEmpId}
                  onChange={(e) => setSelectedEmpId(e.target.value)}
                  className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none"
                >
                  <option value="">-- Chọn nhân viên --</option>
                  {employees.map(e => <option key={e.id} value={e.id}>{e.first_name} {e.last_name}</option>)}
                </select>
              </div>

              {selectedEmpId && balances.length > 0 && (
                <div className="bg-blue-50/50 p-4 rounded-2xl border border-blue-100/50 grid grid-cols-2 gap-3">
                  {balances.map(b => (
                    <div key={b.leave_type} className="space-y-0.5">
                      <p className="text-[10px] font-black uppercase tracking-widest text-blue-600/60">
                        {b.leave_type === 'vacation' ? 'Phép năm' : b.leave_type === 'sick' ? 'Nghỉ ốm' : b.leave_type}
                      </p>
                      <p className="text-sm font-bold text-slate-700">
                        Còn <span className="text-blue-600 text-lg">{b.remaining}</span> / {b.entitlement + b.carried_over} ngày
                      </p>
                    </div>
                  ))}
                </div>
              )}

              <div className="space-y-1">
                <label htmlFor="leave-type-select" className="text-sm font-semibold text-slate-700">Loại nghỉ</label>
                <select id="leave-type-select" name="leave_type" required className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none">
                  <option value="vacation">Nghỉ phép năm</option>
                  <option value="sick">Nghỉ ốm</option>
                  <option value="personal">Việc riêng</option>
                </select>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1">
                  <label htmlFor="leave-start-date" className="text-sm font-semibold text-slate-700">Từ ngày</label>
                  <input id="leave-start-date" type="date" name="start_date" required className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none" />
                </div>
                <div className="space-y-1">
                  <label htmlFor="leave-end-date" className="text-sm font-semibold text-slate-700">Đến ngày</label>
                  <input id="leave-end-date" type="date" name="end_date" required className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none" />
                </div>
              </div>
              <div className="space-y-1">
                <label htmlFor="leave-reason" className="text-sm font-semibold text-slate-700">Lý do</label>
                <textarea id="leave-reason" name="reason" rows={3} className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none" placeholder="Nhập lý do nghỉ..."></textarea>
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

      {/* Policy Modal */}
      {isPolicyModalOpen && (
        <div className="fixed inset-0 bg-slate-900/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-3xl shadow-2xl w-full max-w-md overflow-hidden">
            <div className="px-6 py-4 border-b border-slate-100 flex justify-between items-center bg-slate-50/50">
              <h2 className="text-xl font-bold">Thêm chính sách nghỉ phép</h2>
              <button onClick={() => setIsPolicyModalOpen(false)} className="p-2 hover:bg-white rounded-xl transition-colors"><X className="w-5 h-5 text-slate-400" /></button>
            </div>
            <form onSubmit={handleCreatePolicy} className="p-6 space-y-4">
              <div className="space-y-1">
                <label htmlFor="policy-name" className="text-sm font-semibold text-slate-700">Tên chính sách</label>
                <input id="policy-name" name="name" required className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none" placeholder="VD: Phép năm tiêu chuẩn" />
              </div>
              <div className="space-y-1">
                <label htmlFor="policy-type" className="text-sm font-semibold text-slate-700">Loại nghỉ</label>
                <select id="policy-type" name="leave_type" required className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none">
                  <option value="vacation">Phép năm</option>
                  <option value="sick">Nghỉ ốm</option>
                </select>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1">
                  <label htmlFor="policy-entitlement" className="text-sm font-semibold text-slate-700">Định mức/năm</label>
                  <input id="policy-entitlement" type="number" name="annual_entitlement" required defaultValue={12} className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none" />
                </div>
                <div className="space-y-1">
                  <label htmlFor="policy-carryover" className="text-sm font-semibold text-slate-700">Carry-over limit</label>
                  <input id="policy-carryover" type="number" name="carry_over_limit" required defaultValue={5} className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none" />
                </div>
              </div>
              <div className="space-y-1">
                <label htmlFor="policy-strategy" className="text-sm font-semibold text-slate-700">Chiến lược tích lũy</label>
                <select id="policy-strategy" name="accrual_strategy" required className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none">
                  <option value="monthly">Tích lũy hàng tháng (1 ngày/tháng)</option>
                  <option value="lump_sum">Cấp toàn bộ vào đầu năm</option>
                </select>
              </div>
              <div className="pt-4 flex gap-3">
                <button type="button" onClick={() => setIsPolicyModalOpen(false)} className="flex-1 px-4 py-2 border border-slate-200 rounded-xl font-semibold hover:bg-slate-50 transition-colors">Hủy</button>
                <button type="submit" className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-xl font-semibold hover:bg-blue-700 transition-colors">Lưu chính sách</button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Assign Modal */}
      {isAssignModalOpen && (
        <div className="fixed inset-0 bg-slate-900/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-3xl shadow-2xl w-full max-w-md overflow-hidden">
            <div className="px-6 py-4 border-b border-slate-100 flex justify-between items-center bg-slate-50/50">
              <h2 className="text-xl font-bold">Gán chính sách cho nhân viên</h2>
              <button onClick={() => setIsAssignModalOpen(false)} className="p-2 hover:bg-white rounded-xl transition-colors"><X className="w-5 h-5 text-slate-400" /></button>
            </div>
            <form onSubmit={handleAssignPolicy} className="p-6 space-y-4">
              <div className="space-y-1">
                <label htmlFor="assign-emp" className="text-sm font-semibold text-slate-700">Nhân viên</label>
                <select id="assign-emp" name="employee_id" required className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none">
                  <option value="">-- Chọn nhân viên --</option>
                  {employees.map(e => <option key={e.id} value={e.id}>{e.first_name} {e.last_name}</option>)}
                </select>
              </div>
              <div className="space-y-1">
                <label htmlFor="assign-policy" className="text-sm font-semibold text-slate-700">Chính sách</label>
                <select id="assign-policy" name="leave_policy_id" required className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none">
                  <option value="">-- Chọn chính sách --</option>
                  {policies.map(p => <option key={p.id} value={p.id}>{p.name} ({p.leave_type})</option>)}
                </select>
              </div>
              <div className="space-y-1">
                <label htmlFor="assign-date" className="text-sm font-semibold text-slate-700">Ngày hiệu lực</label>
                <input id="assign-date" type="date" name="effective_date" required defaultValue={new Date().toISOString().split('T')[0]} className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none" />
              </div>
              <div className="pt-4 flex gap-3">
                <button type="button" onClick={() => setIsAssignModalOpen(false)} className="flex-1 px-4 py-2 border border-slate-200 rounded-xl font-semibold hover:bg-slate-50 transition-colors">Hủy</button>
                <button type="submit" className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-xl font-semibold hover:bg-blue-700 transition-colors">Gán ngay</button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}