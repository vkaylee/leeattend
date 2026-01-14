import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Trash2,
  UserCircle,
  Building2,
  Calendar,
  Clock,
  Briefcase,
  Search,
  Plus,
  UserPlus,
  ChevronRight,
  X,
  Edit2,
  ArrowRightLeft,
  Banknote,
  History as HistoryIcon,
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
import { Combobox } from "@/components/ui/Combobox";
import Modal from "@/components/ui/Modal";

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

  // Combobox states
  const [selectedDeptId, setSelectedDeptId] = useState<string>("");
  const [salaryType, setSalaryType] = useState<string>("monthly");

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
      department_id: selectedDeptId,
      effective_date: formData.get("effective_date"),
      notes: formData.get("notes"),
    });
  };

  const handleSalarySubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    salaryMutation.mutate({
      base_salary: formData.get("base_salary"),
      salary_type: salaryType,
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
    <div className="flex h-[calc(100vh-10rem)] gap-8 overflow-hidden animate-enter">
      {/* Left List */}
      <div className={`flex-1 flex flex-col gap-8 overflow-hidden transition-all duration-500 ${selectedEmployee ? 'max-w-[55%]' : 'max-w-full'}`}>
        <div className="flex justify-between items-center shrink-0">
          <div>
            <h1 className="text-3xl font-extrabold tracking-tight text-slate-900">Nhân viên</h1>
            <p className="text-slate-500 mt-1">Quản lý hồ sơ và lịch sử công tác toàn hệ thống</p>
          </div>
          <button 
            onClick={() => { setEditingEmployee(null); setIsModalOpen(true); }}
            className="flex items-center gap-2 bg-violet-600 text-white px-6 py-3 rounded-2xl hover:bg-violet-700 hover:scale-105 transition-all shadow-xl shadow-violet-200 active:scale-95 font-bold"
          >
            <UserPlus className="w-5 h-5" />
            Thêm nhân viên
          </button>
        </div>

        <div className="relative shrink-0 group">
          <Search className="absolute left-5 top-1/2 -translate-y-1/2 text-slate-400 w-5 h-5 group-focus-within:text-violet-500 transition-colors" />
          <input 
            type="text" 
            placeholder="Tìm kiếm theo tên hoặc mã nhân viên..."
            className="w-full pl-14 pr-6 py-4 bg-white border border-slate-100 rounded-[1.5rem] shadow-sm focus:outline-none focus:ring-4 focus:ring-violet-500/10 focus:border-violet-300 transition-all text-lg"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>

        <div className="bg-white border border-slate-50 rounded-[2rem] flex-1 overflow-auto shadow-sm">
          <table className="w-full text-left border-separate border-spacing-0">
            <thead className="bg-slate-50/50 sticky top-0 backdrop-blur-md z-10">
              <tr>
                <th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] border-b border-slate-100">Họ và tên</th>
                <th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] border-b border-slate-100">Mã NV</th>
                {!selectedEmployee && (
                  <>
                    <th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] border-b border-slate-100">Ngày vào làm</th>
                    <th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] border-b border-slate-100">Trạng thái</th>
                  </>
                )}
                <th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] border-b border-slate-100 text-right">Thao tác</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {isLoading ? (
                <tr><td colSpan={5} className="px-8 py-20 text-center text-slate-400 font-medium">Đang tải dữ liệu...</td></tr>
              ) : filteredEmployees.length === 0 ? (
                <tr><td colSpan={5} className="px-8 py-20 text-center text-slate-400 font-medium">Không có dữ liệu hiển thị.</td></tr>
              ) : filteredEmployees.map((emp) => (
                <tr 
                  key={emp.id} 
                  onClick={() => setSelectedEmployee(emp)}
                  className={`cursor-pointer transition-all group ${selectedEmployee?.id === emp.id ? 'bg-violet-50/50' : 'hover:bg-slate-50/50'}`}
                >
                  <td className="px-8 py-5">
                    <div className="flex items-center gap-4">
                      <div className="w-12 h-12 bg-slate-100 rounded-2xl flex items-center justify-center text-slate-400 group-hover:bg-violet-100 group-hover:text-violet-600 transition-all duration-300">
                        <UserCircle className="w-7 h-7" />
                      </div>
                      <div className="font-bold text-slate-900 text-lg">{emp.first_name} {emp.last_name}</div>
                    </div>
                  </td>
                  <td className="px-8 py-5 font-mono text-sm text-slate-500">{emp.employee_id_number}</td>
                  {!selectedEmployee && (
                    <>
                      <td className="px-8 py-5 text-slate-600 text-sm font-medium">{formatDate(emp.hire_date)}</td>
                      <td className="px-8 py-5">
                        <span className={`px-4 py-1.5 rounded-xl text-[10px] font-black uppercase tracking-widest ${emp.is_active ? 'bg-emerald-100 text-emerald-700' : 'bg-slate-100 text-slate-500'}`}>
                          {emp.is_active ? 'Active' : 'Terminated'}
                        </span>
                      </td>
                    </>
                  )}
                  <td className="px-8 py-5 text-right">
                    <div className="bg-slate-50 w-10 h-10 rounded-xl flex items-center justify-center ml-auto group-hover:bg-violet-100 transition-colors">
                      <ChevronRight className={`w-5 h-5 text-slate-300 transition-transform ${selectedEmployee?.id === emp.id ? 'translate-x-1 text-violet-500' : 'group-hover:text-violet-400'}`} />
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Right Detail Panel */}
      {selectedEmployee && (
        <div className="w-[45%] bg-white border border-slate-50 rounded-[2.5rem] flex flex-col shadow-2xl shadow-slate-200/50 overflow-hidden animate-in slide-in-from-right duration-500">
          <div className="p-8 border-b border-slate-50 bg-slate-50/30">
            <div className="flex justify-between items-start mb-8">
              <div className="flex items-center gap-5">
                <div className="w-20 h-20 bg-violet-100 text-violet-600 rounded-[2rem] flex items-center justify-center shadow-inner">
                  <UserCircle className="w-12 h-12" />
                </div>
                <div>
                  <h2 className="text-3xl font-black text-slate-900 tracking-tight">{selectedEmployee.first_name} {selectedEmployee.last_name}</h2>
                  <p className="text-violet-500 font-mono text-sm font-bold mt-1 tracking-wider">{selectedEmployee.employee_id_number}</p>
                </div>
              </div>
              <button onClick={() => setSelectedEmployee(null)} className="p-3 hover:bg-white rounded-2xl transition-all shadow-sm active:scale-90">
                <X className="w-6 h-6 text-slate-400" />
              </button>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <button 
                onClick={() => { setEditingEmployee(selectedEmployee); setIsModalOpen(true); }}
                className="flex items-center justify-center gap-2 py-3.5 bg-white border border-slate-100 text-slate-700 font-bold rounded-2xl hover:bg-slate-50 hover:border-violet-200 transition-all text-sm shadow-sm"
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
                  className="flex items-center justify-center gap-2 py-3.5 bg-rose-50 text-rose-600 font-bold rounded-2xl hover:bg-rose-100 transition-all text-sm"
                >
                  <Trash2 className="w-4 h-4" />
                  Nghỉ việc
                </button>
              ) : (
                <button 
                  onClick={() => setIsRehireModalOpen(true)}
                  className="flex items-center justify-center gap-2 py-3.5 bg-violet-600 text-white font-bold rounded-2xl hover:bg-violet-700 transition-all text-sm shadow-xl shadow-violet-200"
                >
                  <UserPlus className="w-4 h-4" />
                  Vào làm lại
                </button>
              )}
            </div>
          </div>

          <div className="flex-1 overflow-y-auto p-8 space-y-10 scrollbar-hide">
            {/* Department History */}
            <section className="space-y-5">
              <div className="flex justify-between items-center">
                <div className="flex items-center gap-3 font-black text-slate-900 uppercase tracking-widest text-xs">
                  <Building2 className="w-5 h-5 text-violet-500" />
                  <span>Phòng ban hiện tại</span>
                </div>
                <button 
                  onClick={() => setIsTransferModalOpen(true)}
                  className="text-xs font-bold text-violet-600 hover:underline flex items-center gap-1.5 bg-violet-50 px-3 py-1.5 rounded-xl transition-colors"
                >
                  <ArrowRightLeft className="w-3.5 h-3.5" />
                  Điều chuyển
                </button>
              </div>
              
              <div className="bg-slate-50/50 rounded-3xl p-6 border border-slate-50 group hover:border-violet-100 transition-all">
                {deptHistory && deptHistory.length > 0 ? (
                  <div>
                    <div className="font-extrabold text-slate-900 text-xl tracking-tight">
                      {departments?.find(d => d.id === deptHistory[0].department_id)?.name}
                    </div>
                    <div className="text-xs text-slate-400 mt-2 font-bold uppercase tracking-wider flex items-center gap-2">
                       <Calendar className="w-3.5 h-3.5" />
                       Hiệu lực: {formatDate(deptHistory[0].effective_date)}
                    </div>
                  </div>
                ) : (
                  <div className="text-sm text-slate-400 italic font-medium">Chưa được gán phòng ban</div>
                )}
              </div>
            </section>

            {/* Salary History */}
            <section className="space-y-5">
              <div className="flex justify-between items-center">
                <div className="flex items-center gap-3 font-black text-slate-900 uppercase tracking-widest text-xs">
                  <Banknote className="w-5 h-5 text-emerald-500" />
                  <span>Mức lương & Đãi ngộ</span>
                </div>
                <button 
                  onClick={() => setIsSalaryModalOpen(true)}
                  className="text-xs font-bold text-violet-600 hover:underline flex items-center gap-1.5 bg-violet-50 px-3 py-1.5 rounded-xl transition-colors"
                >
                  <Plus className="w-3.5 h-3.5" />
                  Cập nhật
                </button>
              </div>

              <div className="space-y-4">
                {salaryHistory?.map((sal, idx) => (
                  <div key={sal.id} className={`p-6 rounded-3xl border transition-all ${idx === 0 ? 'bg-emerald-50/30 border-emerald-100 shadow-sm' : 'bg-white border-slate-50 hover:border-slate-200'}`}>
                    <div className="flex justify-between items-start">
                      <div>
                        <div className="font-black text-slate-900 text-xl tracking-tight">
                          {new Intl.NumberFormat('vi-VN', { style: 'currency', currency: sal.currency }).format(parseFloat(sal.base_salary))}
                          <span className="text-xs text-slate-400 font-bold ml-2 uppercase tracking-widest">/ {sal.salary_type}</span>
                        </div>
                        <div className="text-[10px] text-slate-500 mt-2 uppercase font-black flex items-center gap-1.5 tracking-widest">
                          <Calendar className="w-3.5 h-3.5" />
                          {formatDate(sal.effective_date)}
                        </div>
                      </div>
                      {idx === 0 && <span className="px-3 py-1 bg-emerald-100 text-emerald-700 text-[9px] font-black rounded-full uppercase tracking-widest shadow-sm">Current</span>}
                    </div>
                  </div>
                ))}
                {(!salaryHistory || salaryHistory.length === 0) && (
                  <div className="bg-slate-50 rounded-3xl p-8 border border-slate-50 text-sm text-slate-400 italic font-medium text-center">
                    Chưa có lịch sử lương
                  </div>
                )}
              </div>
            </section>

            {/* Timeline/Activity */}
            <section className="space-y-6">
              <div className="flex justify-between items-center">
                <div className="flex items-center gap-3 font-black text-slate-900 uppercase tracking-widest text-xs">
                  <HistoryIcon className="w-5 h-5 text-slate-400" />
                  <span>Lịch sử công tác</span>
                </div>
                <button 
                  onClick={() => setIsHistoryModalOpen(true)}
                  className="text-[10px] font-black text-slate-400 hover:text-violet-600 uppercase tracking-widest transition-colors"
                >
                  Xem chi tiết →
                </button>
              </div>
              <div className="relative pl-8 space-y-8 before:absolute before:left-3 before:top-2 before:bottom-2 before:w-[2px] before:bg-slate-100">
                {employmentHistory?.map((period, idx) => (
                  <React.Fragment key={period.id}>
                    {period.termination_date && (
                      <div className="relative animate-in fade-in slide-in-from-left duration-300">
                        <div className="absolute -left-[1.65rem] top-1.5 w-4 h-4 rounded-full bg-rose-500 border-[3px] border-white shadow-md ring-4 ring-rose-50"></div>
                        <div className="text-sm font-black text-slate-800 uppercase tracking-tight">Nghỉ việc</div>
                        <div className="text-xs text-slate-500 mt-1 font-bold">{formatDate(period.termination_date)}</div>
                      </div>
                    )}
                    <div className="relative animate-in fade-in slide-in-from-left duration-300">
                      <div className={`absolute -left-[1.65rem] top-1.5 w-4 h-4 rounded-full border-[3px] border-white shadow-md ring-4 ${idx === 0 && !period.termination_date ? 'bg-emerald-500 ring-emerald-50' : 'bg-violet-500 ring-violet-50'}`}></div>
                      <div className="text-sm font-black text-slate-800 uppercase tracking-tight">
                        {period.is_rehire ? "Quay lại làm việc" : "Gia nhập công ty"}
                      </div>
                      <div className="text-xs text-slate-500 mt-1 font-bold">{formatDate(period.hire_date)}</div>
                    </div>
                  </React.Fragment>
                ))}
              </div>
            </section>
          </div>
        </div>
      )}

      {/* Main Employee Form Modal */}
      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title={editingEmployee ? 'Sửa hồ sơ' : 'Thêm nhân viên'}
        subtitle="Hồ sơ nhân sự"
        icon={<UserCircle className="w-6 h-6" />}
        maxWidth="lg"
      >
        <form onSubmit={handleSubmit} className="space-y-6">
          <div className="grid grid-cols-2 gap-6">
            <div className="space-y-2">
              <label className="text-xs font-black text-slate-400 uppercase tracking-widest px-1">Họ</label>
              <input name="last_name" required defaultValue={editingEmployee?.last_name} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 outline-none transition-all font-bold" />
            </div>
            <div className="space-y-2">
              <label className="text-xs font-black text-slate-400 uppercase tracking-widest px-1">Tên</label>
              <input name="first_name" required defaultValue={editingEmployee?.first_name} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 outline-none transition-all font-bold" />
            </div>
          </div>
          <div className="space-y-2">
            <label className="text-xs font-black text-slate-400 uppercase tracking-widest px-1">Mã nhân viên</label>
            <input name="employee_id_number" required defaultValue={editingEmployee?.employee_id_number} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 outline-none transition-all font-mono font-bold" />
          </div>
          <div className="space-y-2">
            <label className="text-xs font-black text-slate-400 uppercase tracking-widest px-1">Ngày vào làm</label>
            <input type="date" name="hire_date" required defaultValue={editingEmployee?.hire_date} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 outline-none transition-all font-bold" />
          </div>
          <div className="flex items-center gap-4 py-2">
            <input type="checkbox" name="is_active" id="is_active" defaultChecked={editingEmployee ? editingEmployee.is_active : true} className="w-6 h-6 text-primary-600 rounded-xl border-slate-200 focus:ring-primary-500 transition-all cursor-pointer" />
            <label htmlFor="is_active" className="text-sm font-black text-slate-700 uppercase tracking-widest cursor-pointer">Đang hoạt động</label>
          </div>
          <div className="pt-6 grid grid-cols-2 gap-5">
            <button type="button" onClick={() => setIsModalOpen(false)} className="px-8 py-4 border border-slate-100 rounded-2xl font-bold text-slate-500 hover:bg-slate-50 transition-all">Hủy bỏ</button>
            <button type="submit" className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black hover:bg-slate-800 transition-all shadow-xl shadow-slate-200 hover:scale-[1.02] active:scale-95">
              {editingEmployee ? 'Cập nhật' : 'Lưu hồ sơ'}
            </button>
          </div>
        </form>
      </Modal>


      {/* Transfer Modal */}
      <Modal
        isOpen={isTransferModalOpen}
        onClose={() => setIsTransferModalOpen(false)}
        title="Điều chuyển phòng ban"
        subtitle="Quản lý vị trí"
        icon={<Building2 className="w-6 h-6" />}
      >
        <form onSubmit={handleTransferSubmit} className="space-y-5">
          <div className="space-y-2">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Phòng ban mới</label>
            <Combobox
              options={departments?.map(d => ({
                value: d.id,
                label: d.name,
                icon: <Building2 className="w-4 h-4" />
              })) || []}
              value={selectedDeptId}
              onChange={setSelectedDeptId}
              placeholder="Chọn phòng ban..."
              className="bg-slate-50 border-slate-200"
            />
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
            <button type="submit" className="px-6 py-3 bg-slate-900 text-white font-bold rounded-2xl shadow-lg shadow-slate-100 hover:bg-slate-800">Điều chuyển</button>
          </div>
        </form>
      </Modal>

      {/* History Modal */}
      <Modal
        isOpen={isHistoryModalOpen && !!selectedEmployee}
        onClose={() => setIsHistoryModalOpen(false)}
        title="Lịch sử công tác"
        subtitle="Dữ liệu nhân sự"
        icon={<HistoryIcon className="w-6 h-6" />}
        maxWidth="lg"
      >
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
      </Modal>

      {/* Rehire Modal */}
      <Modal
        isOpen={isRehireModalOpen && !!selectedEmployee}
        onClose={() => setIsRehireModalOpen(false)}
        title="Tái tuyển dụng"
        subtitle="Kích hoạt hồ sơ"
        icon={<UserPlus className="w-6 h-6" />}
      >
        <p className="text-slate-500 mb-6 font-medium">Xác nhận ngày nhân viên bắt đầu giai đoạn làm việc mới</p>
        
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
            <button type="submit" disabled={rehireMutation.isPending} className="px-6 py-3 bg-primary-600 text-white font-bold rounded-2xl shadow-lg shadow-primary-100 hover:bg-primary-700 disabled:opacity-50">
              {rehireMutation.isPending ? "Đang xử lý..." : "Xác nhận"}
            </button>
          </div>
        </form>
      </Modal>

      {/* Salary Update Modal */}
      <Modal
        isOpen={isSalaryModalOpen}
        onClose={() => setIsSalaryModalOpen(false)}
        title="Cập nhật lương"
        subtitle="Quản lý đãi ngộ"
        icon={<Banknote className="w-6 h-6" />}
      >
        <form onSubmit={handleSalarySubmit} className="space-y-5">
          <div>
            <label className="block text-sm font-bold text-slate-700 mb-2">Lương cơ bản</label>
            <input type="number" name="base_salary" required className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl outline-none" />
          </div>
          <div className="space-y-2">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">Loại lương</label>
            <Combobox
              options={[
                { value: "monthly", label: "Lương tháng", icon: <Banknote className="w-4 h-4" /> },
                { value: "hourly", label: "Lương giờ", icon: <Clock className="w-4 h-4" /> },
              ]}
              value={salaryType}
              onChange={setSalaryType}
              className="bg-slate-50 border-slate-200"
            />
          </div>
          <div>
            <label className="block text-sm font-bold text-slate-700 mb-2">Ngày hiệu lực</label>
            <input type="date" name="effective_date" required defaultValue={new Date().toISOString().split('T')[0]} className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl outline-none" />
          </div>
          <div className="grid grid-cols-2 gap-4 pt-4">
            <button type="button" onClick={() => setIsSalaryModalOpen(false)} className="px-6 py-3 border border-slate-200 text-slate-600 font-bold rounded-2xl">Hủy</button>
            <button type="submit" className="px-6 py-3 bg-emerald-600 text-white font-bold rounded-2xl shadow-lg shadow-emerald-100 hover:bg-emerald-700">Cập nhật lương</button>
          </div>
        </form>
      </Modal>
    </div>
  );
}