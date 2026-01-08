import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, Search, Edit2, Trash2, UserPlus, X } from "lucide-react";
import { 
  fetchEmployees, 
  createEmployee, 
  updateEmployee, 
  deleteEmployee, 
  getCurrentWorkspaceId,
  Employee
} from "../lib/api";

export default function EmployeesPage() {
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingEmployee, setEditingEmployee] = useState<Employee | null>(null);
  const [searchTerm, setSearchTerm] = useState("");

  const { data: employees = [], isLoading } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId!),
    enabled: !!wsId,
  });

  const createMutation = useMutation({
    mutationFn: (data: Partial<Employee>) => createEmployee(wsId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      setIsModalOpen(false);
    },
  });

  const updateMutation = useMutation({
    mutationFn: (data: { id: string; employee: Partial<Employee> }) => 
      updateEmployee(wsId!, data.id, data.employee),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      setIsModalOpen(false);
      setEditingEmployee(null);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteEmployee(wsId!, id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
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

  if (!wsId) return <div className="p-8 text-center">Vui lòng chọn Workspace</div>;

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Danh sách nhân viên</h1>
          <p className="text-slate-500">Quản lý hồ sơ nhân sự trong workspace của bạn.</p>
        </div>
        <button 
          onClick={() => { setEditingEmployee(null); setIsModalOpen(true); }}
          className="flex items-center gap-2 bg-blue-600 text-white px-4 py-2 rounded-xl hover:bg-blue-700 transition-colors shadow-lg shadow-blue-200"
        >
          <UserPlus className="w-5 h-5" />
          <span className="font-semibold">Thêm nhân viên</span>
        </button>
      </div>

      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 w-5 h-5" />
        <input 
          type="text" 
          placeholder="Tìm kiếm theo tên hoặc mã nhân viên..."
          className="w-full pl-10 pr-4 py-3 bg-white border border-slate-200 rounded-2xl focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all"
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
      </div>

      <div className="bg-white border border-slate-200 rounded-3xl overflow-hidden shadow-sm">
        <table className="w-full text-left">
          <thead className="bg-slate-50 border-b border-slate-200">
            <tr>
              <th className="px-6 py-4 text-sm font-semibold text-slate-600">Họ và tên</th>
              <th className="px-6 py-4 text-sm font-semibold text-slate-600">Mã NV</th>
              <th className="px-6 py-4 text-sm font-semibold text-slate-600">Ngày vào làm</th>
              <th className="px-6 py-4 text-sm font-semibold text-slate-600">Trạng thái</th>
              <th className="px-6 py-4 text-sm font-semibold text-slate-600 text-right">Thao tác</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-100">
            {isLoading ? (
              <tr><td colSpan={5} className="px-6 py-8 text-center text-slate-500">Đang tải...</td></tr>
            ) : filteredEmployees.length === 0 ? (
              <tr><td colSpan={5} className="px-6 py-8 text-center text-slate-500">Không tìm thấy nhân viên nào.</td></tr>
            ) : filteredEmployees.map((emp) => (
              <tr key={emp.id} className="hover:bg-slate-50/50 transition-colors group">
                <td className="px-6 py-4">
                  <div className="font-medium text-slate-900">{emp.first_name} {emp.last_name}</div>
                </td>
                <td className="px-6 py-4 font-mono text-sm text-slate-600">{emp.employee_id_number}</td>
                <td className="px-6 py-4 text-slate-600">{new Date(emp.hire_date).toLocaleDateString('vi-VN')}</td>
                <td className="px-6 py-4">
                  <span className={`px-2 py-1 rounded-full text-xs font-medium ${emp.is_active ? 'bg-green-100 text-green-700' : 'bg-slate-100 text-slate-600'}`}>
                    {emp.is_active ? 'Đang làm việc' : 'Đã nghỉ việc'}
                  </span>
                </td>
                <td className="px-6 py-4 text-right">
                  <div className="flex justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button 
                      onClick={() => { setEditingEmployee(emp); setIsModalOpen(true); }}
                      className="p-2 text-slate-400 hover:text-blue-600 hover:bg-blue-50 rounded-lg transition-all"
                    >
                      <Edit2 className="w-4 h-4" />
                    </button>
                    <button 
                      onClick={() => { if(confirm('Xác nhận xóa nhân viên này?')) deleteMutation.mutate(emp.id); }}
                      className="p-2 text-slate-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-all"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Modal Form */}
      {isModalOpen && (
        <div className="fixed inset-0 bg-slate-900/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-3xl shadow-2xl w-full max-w-md overflow-hidden animate-in fade-in zoom-in duration-200">
            <div className="px-6 py-4 border-b border-slate-100 flex justify-between items-center bg-slate-50/50">
              <h2 className="text-xl font-bold">{editingEmployee ? 'Sửa thông tin' : 'Thêm nhân viên mới'}</h2>
              <button onClick={() => setIsModalOpen(false)} className="p-2 hover:bg-white rounded-xl transition-colors">
                <X className="w-5 h-5 text-slate-400" />
              </button>
            </div>
            <form onSubmit={handleSubmit} className="p-6 space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1">
                  <label className="text-sm font-semibold text-slate-700">Họ</label>
                  <input name="last_name" required defaultValue={editingEmployee?.last_name} className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-blue-500/20 outline-none" />
                </div>
                <div className="space-y-1">
                  <label className="text-sm font-semibold text-slate-700">Tên</label>
                  <input name="first_name" required defaultValue={editingEmployee?.first_name} className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-blue-500/20 outline-none" />
                </div>
              </div>
              <div className="space-y-1">
                <label className="text-sm font-semibold text-slate-700">Mã nhân viên</label>
                <input name="employee_id_number" required defaultValue={editingEmployee?.employee_id_number} className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-blue-500/20 outline-none" />
              </div>
              <div className="space-y-1">
                <label className="text-sm font-semibold text-slate-700">Ngày vào làm</label>
                <input type="date" name="hire_date" required defaultValue={editingEmployee?.hire_date} className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-blue-500/20 outline-none" />
              </div>
              <div className="flex items-center gap-2 py-2">
                <input type="checkbox" name="is_active" id="is_active" defaultChecked={editingEmployee ? editingEmployee.is_active : true} className="w-4 h-4 text-blue-600 rounded" />
                <label htmlFor="is_active" className="text-sm font-medium text-slate-700">Đang hoạt động</label>
              </div>
              <div className="pt-4 flex gap-3">
                <button type="button" onClick={() => setIsModalOpen(false)} className="flex-1 px-4 py-2 border border-slate-200 rounded-xl font-semibold hover:bg-slate-50 transition-colors">Hủy</button>
                <button type="submit" className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-xl font-semibold hover:bg-blue-700 transition-colors shadow-lg shadow-blue-200">
                  {editingEmployee ? 'Cập nhật' : 'Lưu'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
