import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Plus, 
  Search, 
  Edit2, 
  Trash2, 
  UserPlus, 
  X, 
  Building2, 
  Banknote, 
  ArrowRightLeft, 
  History,
  Calendar,
  ChevronRight,
  UserCircle
} from "lucide-react";
import { 
  fetchEmployees, 
  createEmployee, 
  updateEmployee, 
  deleteEmployee, 
  getCurrentWorkspaceId,
  Employee,
  fetchDepartments,
  fetchSalaryHistory,
  fetchEmployeeDepartmentHistory,
  addSalaryHistory,
  transferEmployee,
  fetchEmploymentHistory,
  rehireEmployee
} from "../lib/api";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

export default function EmployeesPage() {
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const { formatDate } = useDateTimeFormatter();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingEmployee, setEditingEmployee] = useState<Employee | null>(null);
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  
  // States for sub-modals
  const [isTransferModalOpen, setIsTransferModalOpen] = useState(false);
  const [isSalaryModalOpen, setIsSalaryModalOpen] = useState(false);
  const [isHistoryModalOpen, setIsHistoryModalOpen] = useState(false);
  const [isRehireModalOpen, setIsRehireModalOpen] = useState(false);

  const { data: employees = [], isLoading } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId),
    enabled: !!wsId,
  });

  const { data: departments } = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId),
    enabled: !!wsId,
  });

  const { data: salaryHistory } = useQuery({
    queryKey: ["salaryHistory", selectedEmployee?.id],
    queryFn: () => fetchSalaryHistory(wsId, selectedEmployee!.id),
    enabled: !!selectedEmployee,
  });

  const { data: deptHistory } = useQuery({
    queryKey: ["deptHistory", selectedEmployee?.id],
    queryFn: () => fetchEmployeeDepartmentHistory(wsId, selectedEmployee!.id),
    enabled: !!selectedEmployee,
  });

  const { data: employmentHistory } = useQuery({
    queryKey: ["employmentHistory", selectedEmployee?.id],
    queryFn: () => fetchEmploymentHistory(wsId, selectedEmployee!.id),
    enabled: !!selectedEmployee,
  });

  const createMutation = useMutation({
    mutationFn: (data: Partial<Employee>) => createEmployee(wsId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      setIsModalOpen(false);
    },
  });

  const updateMutation = useMutation({
    mutationFn: (data: { id: string; employee: Partial<Employee> }) => 
      updateEmployee(wsId, data.id, data.employee),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      setIsModalOpen(false);
      setEditingEmployee(null);
    },
  });

  const transferMutation = useMutation({
    mutationFn: (data: any) => transferEmployee(wsId, selectedEmployee!.id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["deptHistory", selectedEmployee?.id] });
      setIsTransferModalOpen(false);
    },
  });

  const salaryMutation = useMutation({
    mutationFn: (data: any) => addSalaryHistory(wsId, selectedEmployee!.id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["salaryHistory", selectedEmployee?.id] });
      setIsSalaryModalOpen(false);
    },
  });

  const rehireMutation = useMutation({
    mutationFn: (data: { hire_date: string }) => rehireEmployee(wsId, selectedEmployee!.id, data),
    onSuccess: (updatedEmp) => {
      queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      queryClient.invalidateQueries({ queryKey: ["employmentHistory", selectedEmployee?.id] });
      setSelectedEmployee(updatedEmp);
      setIsRehireModalOpen(false);
    },
  });

  const filteredEmployees = employees.filter(emp => 
    `${emp.first_name} ${emp.last_name}`.toLowerCase().includes(searchTerm.toLowerCase()) ||
    emp.employee_id_number.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const data = {
      first_name: formData.get("first_name") as string,
      last_name: formData.get("last_name") as string,
      employee_id_number: formData.get("employee_id_number") as string,
      hire_date: formData.get("hire_date") as string,
      is_active: formData.get("is_active") === "on",
    };

    if (editingEmployee) {
      updateMutation.mutate({ id: editingEmployee.id, employee: data });
    } else {
      createMutation.mutate(data);
    }
  };

  const handleTransferSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    transferMutation.mutate({
      department_id: formData.get("department_id"),
      effective_date: formData.get("effective_date"),
      notes: formData.get("notes"),
    });
  };

  const handleSalarySubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    salaryMutation.mutate({
      base_salary: formData.get("base_salary"),
      salary_type: formData.get("salary_type"),
      currency: "VND",
      effective_date: formData.get("effective_date"),
    });
  };

  const handleRehireSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    rehireMutation.mutate({
      hire_date: formData.get("hire_date") as string,
    });
  };

  if (!wsId) return <div className="p-8 text-center">Vui lòng chọn Workspace</div>;

  return (
    <div className="flex h-[calc(100vh-8rem)] gap-6 overflow-hidden">
      {/* Left List */}
      <div className={`flex-1 flex flex-col gap-6 overflow-hidden transition-all ${selectedEmployee ? 'max-w-[60%]' : 'max-w-full'}`}>
        <div className="flex justify-between items-center shrink-0">
          <div>
            <h1 className="text-3xl font-bold tracking-tight text-slate-900">Nhân viên</h1>
            <p className="text-slate-500">Quản lý hồ sơ và lịch sử công tác</p>
          </div>
          <button 
            onClick={() => { setEditingEmployee(null); setIsModalOpen(true); }}
            className="flex items-center gap-2 bg-blue-600 text-white px-5 py-2.5 rounded-2xl hover:bg-blue-700 transition-all shadow-lg shadow-blue-100 active:scale-95 font-semibold"
          >
            <UserPlus className="w-5 h-5" />
            Thêm nhân viên
          </button>
        </div>

        <div className="relative shrink-0">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 w-5 h-5" />
          <input 
            type="text" 
            placeholder="Tìm kiếm theo tên hoặc mã nhân viên..."
            className="w-full pl-12 pr-4 py-3.5 bg-white border border-slate-200 rounded-2xl focus:outline-none focus:ring-4 focus:ring-blue-500/10 focus:border-blue-500 transition-all"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>

        <div className="bg-white border border-slate-200 rounded-3xl flex-1 overflow-auto shadow-sm">
          <table className="w-full text-left">
            <thead className="bg-slate-50/50 sticky top-0 backdrop-blur-md border-b border-slate-200 z-10">
              <tr>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">Họ và tên</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">Mã NV</th>
                {!selectedEmployee && (
                  <>
                    <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">Ngày vào làm</th>
                    <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">Trạng thái</th>
                  </>
                )}
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider text-right">Thao tác</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {isLoading ? (
                <tr><td colSpan={5} className="px-6 py-12 text-center text-slate-400">Đang tải dữ liệu...</td></tr>
              ) : filteredEmployees.length === 0 ? (
                <tr><td colSpan={5} className="px-6 py-12 text-center text-slate-400">Không có dữ liệu hiển thị.</td></tr>
              ) : filteredEmployees.map((emp) => (
                <tr 
                  key={emp.id} 
                  onClick={() => setSelectedEmployee(emp)}
                  className={`cursor-pointer transition-all group ${selectedEmployee?.id === emp.id ? 'bg-blue-50/50' : 'hover:bg-slate-50'}`}
                >
                  <td className="px-6 py-4">
                    <div className="flex items-center gap-3">
                      <div className="w-10 h-10 bg-slate-100 rounded-full flex items-center justify-center text-slate-500 group-hover:bg-blue-100 group-hover:text-blue-600 transition-colors">
                        <UserCircle className="w-6 h-6" />
                      </div>
                      <div className="font-bold text-slate-900">{emp.first_name} {emp.last_name}</div>
                    </div>
                  </td>
                  <td className="px-6 py-4 font-mono text-sm text-slate-500">{emp.employee_id_number}</td>
                  {!selectedEmployee && (
                    <>
                      <td className="px-6 py-4 text-slate-600 text-sm">{formatDate(emp.hire_date)}</td>
                      <td className="px-6 py-4">
                        <span className={`px-2.5 py-1 rounded-full text-[10px] font-bold uppercase ${emp.is_active ? 'bg-green-100 text-green-700' : 'bg-slate-100 text-slate-600'}`}>
                          {emp.is_active ? 'Active' : 'Terminated'}
                        </span>
                      </td>
                    </>
                  )}
                  <td className="px-6 py-4 text-right">
                    <ChevronRight className={`w-5 h-5 text-slate-300 ml-auto transition-transform ${selectedEmployee?.id === emp.id ? 'translate-x-1 text-blue-500' : ''}`} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Right Detail Panel */}
      {selectedEmployee && (
        <div className="w-[40%] bg-white border border-slate-200 rounded-3xl flex flex-col shadow-xl overflow-hidden animate-in slide-in-from-right duration-300">
          <div className="p-6 border-b border-slate-100 bg-slate-50/30">
            <div className="flex justify-between items-start mb-6">
              <div className="flex items-center gap-4">
                <div className="w-16 h-16 bg-blue-100 text-blue-600 rounded-3xl flex items-center justify-center">
                  <UserCircle className="w-10 h-10" />
                </div>
                <div>
                  <h2 className="text-2xl font-bold text-slate-900">{selectedEmployee.first_name} {selectedEmployee.last_name}</h2>
                  <p className="text-slate-500 font-mono text-sm">{selectedEmployee.employee_id_number}</p>
                </div>
              </div>
              <button onClick={() => setSelectedEmployee(null)} className="p-2 hover:bg-slate-100 rounded-xl transition-colors">
                <X className="w-5 h-5 text-slate-400" />
              </button>
            </div>

            <div className="grid grid-cols-2 gap-3">
              <button 
                onClick={() => { setEditingEmployee(selectedEmployee); setIsModalOpen(true); }}
                className="flex items-center justify-center gap-2 py-2.5 bg-white border border-slate-200 text-slate-700 font-bold rounded-2xl hover:bg-slate-50 transition-all text-sm"
              >
                <Edit2 className="w-4 h-4" />
                Sửa hồ sơ
              </button>
              {selectedEmployee.is_active ? (
                <button 
                  onClick={() => {
                    const confirmTerm = window.confirm("Bạn có chắc muốn cho nhân viên này nghỉ việc?");
                    if (confirmTerm) {
                      updateMutation.mutate({ id: selectedEmployee.id, employee: { is_active: false, termination_date: new Date().toISOString().split('T')[0] } });
                    }
                  }}
                  className="flex items-center justify-center gap-2 py-2.5 bg-red-50 text-red-600 font-bold rounded-2xl hover:bg-red-100 transition-all text-sm"
                >
                  <Trash2 className="w-4 h-4" />
                  Nghỉ việc
                </button>
              ) : (
                <button 
                  onClick={() => setIsRehireModalOpen(true)}
                  className="flex items-center justify-center gap-2 py-2.5 bg-blue-600 text-white font-bold rounded-2xl hover:bg-blue-700 transition-all text-sm shadow-lg shadow-blue-100"
                >
                  <UserPlus className="w-4 h-4" />
                  Vào làm lại
                </button>
              )}
            </div>
          </div>

          <div className="flex-1 overflow-y-auto p-6 space-y-8">
            {/* Department History */}
            <section className="space-y-4">
              <div className="flex justify-between items-center">
                <div className="flex items-center gap-2 font-bold text-slate-900">
                  <Building2 className="w-5 h-5 text-blue-500" />
                  <span>Phòng ban hiện tại</span>
                </div>
                <button 
                  onClick={() => setIsTransferModalOpen(true)}
                  className="text-sm font-bold text-blue-600 hover:underline flex items-center gap-1"
                >
                  <ArrowRightLeft className="w-4 h-4" />
                  Điều chuyển
                </button>
              </div>
              
              <div className="bg-slate-50 rounded-2xl p-4 border border-slate-100">
                {deptHistory && deptHistory.length > 0 ? (
                  <div>
                    <div className="font-bold text-slate-900">
                      {departments?.find(d => d.id === deptHistory[0].department_id)?.name}
                    </div>
                    <div className="text-xs text-slate-500 mt-1">Hiệu lực từ: {formatDate(deptHistory[0].effective_date)}</div>
                  </div>
                ) : (
                  <div className="text-sm text-slate-400 italic">Chưa được gán phòng ban</div>
                )}
              </div>
            </section>

            {/* Salary History */}
            <section className="space-y-4">
              <div className="flex justify-between items-center">
                <div className="flex items-center gap-2 font-bold text-slate-900">
                  <Banknote className="w-5 h-5 text-green-500" />
                  <span>Mức lương & Đãi ngộ</span>
                </div>
                <button 
                  onClick={() => setIsSalaryModalOpen(true)}
                  className="text-sm font-bold text-blue-600 hover:underline flex items-center gap-1"
                >
                  <Plus className="w-4 h-4" />
                  Cập nhật
                </button>
              </div>

              <div className="space-y-3">
                {salaryHistory?.map((sal, idx) => (
                  <div key={sal.id} className={`p-4 rounded-2xl border ${idx === 0 ? 'bg-green-50/30 border-green-100' : 'bg-white border-slate-100'}`}>
                    <div className="flex justify-between items-start">
                      <div>
                        <div className="font-bold text-slate-900">
                          {new Intl.NumberFormat('vi-VN', { style: 'currency', currency: sal.currency }).format(parseFloat(sal.base_salary))}
                          <span className="text-xs text-slate-400 font-normal ml-2">/ {sal.salary_type}</span>
                        </div>
                        <div className="text-[10px] text-slate-500 mt-1 uppercase font-bold flex items-center gap-1">
                          <Calendar className="w-3 h-3" />
                          {formatDate(sal.effective_date)}
                        </div>
                      </div>
                      {idx === 0 && <span className="px-2 py-0.5 bg-green-100 text-green-700 text-[9px] font-bold rounded-full uppercase tracking-tighter">Current</span>}
                    </div>
                  </div>
                ))}
                {(!salaryHistory || salaryHistory.length === 0) && (
                  <div className="bg-slate-50 rounded-2xl p-4 border border-slate-100 text-sm text-slate-400 italic text-center">
                    Chưa có lịch sử lương
                  </div>
                )}
              </div>
            </section>

            {/* Timeline/Activity */}
            <section className="space-y-4">
              <div className="flex justify-between items-center">
                <div className="flex items-center gap-2 font-bold text-slate-900">
                  <History className="w-5 h-5 text-slate-400" />
                  <span>Lịch sử công tác</span>
                </div>
                <button 
                  onClick={() => setIsHistoryModalOpen(true)}
                  className="text-xs font-bold text-blue-600 hover:underline"
                >
                  Xem chi tiết
                </button>
              </div>
              <div className="relative pl-6 space-y-6 before:absolute before:left-2 before:top-2 before:bottom-2 before:w-0.5 before:bg-slate-100">
                {employmentHistory?.map((period, idx) => (
                  <React.Fragment key={period.id}>
                    {period.termination_date && (
                      <div className="relative">
                        <div className="absolute -left-[1.35rem] top-1 w-3 h-3 rounded-full bg-red-500 border-2 border-white shadow-sm ring-4 ring-red-50"></div>
                        <div className="text-sm font-bold text-slate-800">Nghỉ việc</div>
                        <div className="text-xs text-slate-500">{formatDate(period.termination_date)}</div>
                      </div>
                    )}
                    <div className="relative">
                      <div className={`absolute -left-[1.35rem] top-1 w-3 h-3 rounded-full border-2 border-white shadow-sm ring-4 ${idx === 0 && !period.termination_date ? 'bg-green-500 ring-green-50' : 'bg-blue-500 ring-blue-50'}`}></div>
                      <div className="text-sm font-bold text-slate-800">
                        {period.is_rehire ? "Quay lại làm việc" : "Gia nhập công ty"}
                      </div>
                      <div className="text-xs text-slate-500">{formatDate(period.hire_date)}</div>
                    </div>
                  </React.Fragment>
                ))}
              </div>
            </section>
          </div>
        </div>
      )}

      {/* Main Employee Form Modal */}
      {isModalOpen && (
        <div className="fixed inset-0 bg-slate-900/50 backdrop-blur-sm flex items-center justify-center z-[60] p-4">
          <div className="bg-white rounded-[32px] shadow-2xl w-full max-w-md overflow-hidden animate-in fade-in zoom-in duration-200">
            <div className="px-8 py-6 border-b border-slate-100 flex justify-between items-center bg-slate-50/30">
              <h2 className="text-2xl font-bold text-slate-900">{editingEmployee ? 'Sửa hồ sơ' : 'Thêm nhân viên mới'}</h2>
              <button onClick={() => setIsModalOpen(false)} className="p-2 hover:bg-white rounded-2xl transition-colors">
                <X className="w-6 h-6 text-slate-400" />
              </button>
            </div>
            <form onSubmit={handleSubmit} className="p-8 space-y-5">
              <div className="grid grid-cols-2 gap-5">
                <div className="space-y-2">
                  <label className="text-sm font-bold text-slate-700">Họ</label>
                  <input name="last_name" required defaultValue={editingEmployee?.last_name} className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl focus:ring-4 focus:ring-blue-500/10 outline-none transition-all" />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-bold text-slate-700">Tên</label>
                  <input name="first_name" required defaultValue={editingEmployee?.first_name} className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl focus:ring-4 focus:ring-blue-500/10 outline-none transition-all" />
                </div>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-bold text-slate-700">Mã nhân viên</label>
                <input name="employee_id_number" required defaultValue={editingEmployee?.employee_id_number} className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl focus:ring-4 focus:ring-blue-500/10 outline-none transition-all" />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-bold text-slate-700">Ngày vào làm</label>
                <input type="date" name="hire_date" required defaultValue={editingEmployee?.hire_date} className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl focus:ring-4 focus:ring-blue-500/10 outline-none transition-all" />
              </div>
              <div className="flex items-center gap-3 py-2">
                <input type="checkbox" name="is_active" id="is_active" defaultChecked={editingEmployee ? editingEmployee.is_active : true} className="w-5 h-5 text-blue-600 rounded-lg border-slate-300 focus:ring-blue-500" />
                <label htmlFor="is_active" className="text-sm font-bold text-slate-700">Đang hoạt động</label>
              </div>
              <div className="pt-4 grid grid-cols-2 gap-4">
                <button type="button" onClick={() => setIsModalOpen(false)} className="px-6 py-3 border border-slate-200 rounded-2xl font-bold text-slate-600 hover:bg-slate-50 transition-all">Hủy</button>
                <button type="submit" className="px-6 py-3 bg-blue-600 text-white rounded-2xl font-bold hover:bg-blue-700 transition-all shadow-lg shadow-blue-100">
                  {editingEmployee ? 'Cập nhật' : 'Lưu hồ sơ'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Transfer Modal */}
      {isTransferModalOpen && (
        <div className="fixed inset-0 bg-slate-900/50 backdrop-blur-sm flex items-center justify-center z-[70] p-4">
          <div className="bg-white rounded-[32px] shadow-2xl w-full max-w-md overflow-hidden animate-in fade-in zoom-in duration-200">
            <div className="p-8">
              <h2 className="text-2xl font-bold text-slate-900 mb-6">Điều chuyển phòng ban</h2>
              <form onSubmit={handleTransferSubmit} className="space-y-5">
                <div>
                  <label className="block text-sm font-bold text-slate-700 mb-2">Phòng ban mới</label>
                  <select name="department_id" required className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl outline-none appearance-none">
                    <option value="">-- Chọn phòng ban --</option>
                    {departments?.map(d => <option key={d.id} value={d.id}>{d.name}</option>)}
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-bold text-slate-700 mb-2">Ngày hiệu lực</label>
                  <input type="date" name="effective_date" required defaultValue={new Date().toISOString().split('T')[0]} className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl outline-none" />
                </div>
                <div>
                  <label className="block text-sm font-bold text-slate-700 mb-2">Ghi chú</label>
                  <textarea name="notes" className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl outline-none h-24 resize-none" />
                </div>
                <div className="grid grid-cols-2 gap-4 pt-4">
                  <button type="button" onClick={() => setIsTransferModalOpen(false)} className="px-6 py-3 border border-slate-200 text-slate-600 font-bold rounded-2xl">Hủy</button>
                  <button type="submit" className="px-6 py-3 bg-blue-600 text-white font-bold rounded-2xl shadow-lg shadow-blue-100 hover:bg-blue-700">Điều chuyển</button>
                </div>
              </form>
            </div>
          </div>
        </div>
      )}

      {/* History Modal */}
      {isHistoryModalOpen && selectedEmployee && (
        <div className="fixed inset-0 bg-slate-900/50 backdrop-blur-sm flex items-center justify-center z-[70] p-4">
          <div className="bg-white rounded-[32px] shadow-2xl w-full max-w-lg overflow-hidden animate-in fade-in zoom-in duration-200">
            <div className="p-8">
              <div className="flex justify-between items-center mb-6">
                <h2 className="text-2xl font-bold text-slate-900 text-center">Chi tiết lịch sử công tác</h2>
                <button onClick={() => setIsHistoryModalOpen(false)} className="p-2 hover:bg-slate-100 rounded-xl transition-colors">
                  <X className="w-5 h-5 text-slate-400" />
                </button>
              </div>
              
              <div className="space-y-4">
                {employmentHistory?.map((period) => (
                  <div key={period.id} className="p-4 bg-slate-50 border border-slate-100 rounded-2xl">
                    <div className="flex justify-between items-start">
                      <div>
                        <div className="font-bold text-slate-900 flex items-center gap-2">
                          {period.is_rehire ? "Giai đoạn tái tuyển dụng" : "Giai đoạn đầu tiên"}
                          {!period.termination_date && (
                            <span className="px-2 py-0.5 bg-green-100 text-green-700 text-[9px] font-bold rounded-full uppercase">Đang làm việc</span>
                          )}
                        </div>
                        <div className="text-sm text-slate-500 mt-1">
                          Từ {formatDate(period.hire_date)} {period.termination_date ? `đến ${formatDate(period.termination_date)}` : "(Hiện tại)"}
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
              
              <div className="mt-8">
                <button 
                  onClick={() => setIsHistoryModalOpen(false)}
                  className="w-full py-3 bg-slate-100 text-slate-600 font-bold rounded-2xl hover:bg-slate-200 transition-all"
                >
                  Đóng
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Rehire Modal */}
      {isRehireModalOpen && selectedEmployee && (
        <div className="fixed inset-0 bg-slate-900/50 backdrop-blur-sm flex items-center justify-center z-[70] p-4">
          <div className="bg-white rounded-[32px] shadow-2xl w-full max-w-md overflow-hidden animate-in fade-in zoom-in duration-200">
            <div className="p-8">
              <h2 className="text-2xl font-bold text-slate-900 mb-2 text-center">Tái tuyển dụng</h2>
              <p className="text-slate-500 text-center mb-6">Xác nhận ngày nhân viên bắt đầu giai đoạn làm việc mới</p>
              
              <form onSubmit={handleRehireSubmit} className="space-y-5">
                <div>
                  <label className="block text-sm font-bold text-slate-700 mb-2">Ngày vào làm lại</label>
                  <input 
                    type="date" 
                    name="hire_date" 
                    required 
                    defaultValue={new Date().toISOString().split('T')[0]} 
                    className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl outline-none" 
                  />
                </div>
                <div className="grid grid-cols-2 gap-4 pt-4">
                  <button type="button" onClick={() => setIsRehireModalOpen(false)} className="px-6 py-3 border border-slate-200 text-slate-600 font-bold rounded-2xl">Hủy</button>
                  <button type="submit" disabled={rehireMutation.isPending} className="px-6 py-3 bg-blue-600 text-white font-bold rounded-2xl shadow-lg shadow-blue-100 hover:bg-blue-700 disabled:opacity-50">
                    {rehireMutation.isPending ? "Đang xử lý..." : "Xác nhận"}
                  </button>
                </div>
              </form>
            </div>
          </div>
        </div>
      )}

      {/* Salary Update Modal */}
      {isSalaryModalOpen && (
        <div className="fixed inset-0 bg-slate-900/50 backdrop-blur-sm flex items-center justify-center z-[70] p-4">
          <div className="bg-white rounded-[32px] shadow-2xl w-full max-w-md overflow-hidden animate-in fade-in zoom-in duration-200">
            <div className="p-8">
              <h2 className="text-2xl font-bold text-slate-900 mb-6">Cập nhật lương</h2>
              <form onSubmit={handleSalarySubmit} className="space-y-5">
                <div>
                  <label className="block text-sm font-bold text-slate-700 mb-2">Lương cơ bản</label>
                  <input type="number" name="base_salary" required className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl outline-none" />
                </div>
                <div>
                  <label className="block text-sm font-bold text-slate-700 mb-2">Loại lương</label>
                  <select name="salary_type" required className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl outline-none appearance-none">
                    <option value="monthly">Lương tháng</option>
                    <option value="hourly">Lương giờ</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-bold text-slate-700 mb-2">Ngày hiệu lực</label>
                  <input type="date" name="effective_date" required defaultValue={new Date().toISOString().split('T')[0]} className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl outline-none" />
                </div>
                <div className="grid grid-cols-2 gap-4 pt-4">
                  <button type="button" onClick={() => setIsSalaryModalOpen(false)} className="px-6 py-3 border border-slate-200 text-slate-600 font-bold rounded-2xl">Hủy</button>
                  <button type="submit" className="px-6 py-3 bg-green-600 text-white font-bold rounded-2xl shadow-lg shadow-green-100 hover:bg-green-700">Cập nhật lương</button>
                </div>
              </form>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
