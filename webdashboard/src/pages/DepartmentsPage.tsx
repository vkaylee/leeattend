import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Building2, 
  Plus, 
  Users, 
  User as UserIcon, 
  ArrowRightLeft, 
  History,
  MoreVertical,
  Edit2,
  Trash2
} from "lucide-react";
import { 
  fetchDepartments, 
  createDepartment, 
  getCurrentWorkspaceId,
  fetchEmployees
} from "../lib/api";

export default function DepartmentsPage() {
  const queryClient = useQueryClient();
  const wsId = getCurrentWorkspaceId() || "";
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [newDept, setNewDept] = useState({ name: "", description: "", manager_id: "" });

  const { data: departments, isLoading } = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId),
    enabled: !!wsId,
  });

  const { data: employees } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId),
    enabled: !!wsId,
  });

  const createMutation = useMutation({
    mutationFn: (data: any) => createDepartment(wsId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["departments", wsId] });
      setIsAddModalOpen(false);
      setNewDept({ name: "", description: "", manager_id: "" });
    },
  });

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-end">
        <div>
          <h1 className="text-3xl font-bold text-slate-900 tracking-tight">Phòng ban</h1>
          <p className="text-slate-500 mt-1">Quản lý cơ cấu tổ chức và điều chuyển nhân sự</p>
        </div>
        <button
          onClick={() => setIsAddModalOpen(true)}
          className="flex items-center gap-2 bg-blue-600 hover:bg-blue-700 text-white px-5 py-2.5 rounded-2xl font-semibold transition-all shadow-lg shadow-blue-100 active:scale-95"
        >
          <Plus className="w-5 h-5" />
          Thêm phòng ban
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {departments?.map((dept) => (
          <div key={dept.id} className="bg-white rounded-3xl border border-slate-200 p-6 shadow-sm hover:shadow-md transition-shadow relative group">
            <div className="flex items-start justify-between mb-4">
              <div className="p-3 bg-blue-50 rounded-2xl text-blue-600">
                <Building2 className="w-6 h-6" />
              </div>
              <button className="text-slate-400 hover:text-slate-600 p-1 opacity-0 group-hover:opacity-100 transition-opacity">
                <MoreVertical className="w-5 h-5" />
              </button>
            </div>

            <h3 className="text-xl font-bold text-slate-900 mb-1">{dept.name}</h3>
            <p className="text-slate-500 text-sm mb-4 line-clamp-2 min-h-[2.5rem]">
              {dept.description || "Chưa có mô tả cho phòng ban này."}
            </p>

            <div className="space-y-3 pt-4 border-t border-slate-50">
              <div className="flex items-center justify-between text-sm">
                <div className="flex items-center gap-2 text-slate-500">
                  <UserIcon className="w-4 h-4" />
                  <span>Trưởng phòng:</span>
                </div>
                <span className="font-semibold text-slate-700">
                  {employees?.find(e => e.id === dept.manager_id)?.last_name || "Chưa gán"}
                </span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <div className="flex items-center gap-2 text-slate-500">
                  <Users className="w-4 h-4" />
                  <span>Nhân sự:</span>
                </div>
                <span className="font-semibold text-slate-700">0 nhân viên</span>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-3 mt-6">
              <button className="flex items-center justify-center gap-2 py-2 text-sm font-semibold text-slate-600 bg-slate-50 hover:bg-slate-100 rounded-xl transition-colors">
                <Edit2 className="w-4 h-4" />
                Sửa
              </button>
              <button className="flex items-center justify-center gap-2 py-2 text-sm font-semibold text-blue-600 bg-blue-50 hover:bg-blue-100 rounded-xl transition-colors">
                <ArrowRightLeft className="w-4 h-4" />
                Điều chuyển
              </button>
            </div>
          </div>
        ))}

        {departments?.length === 0 && (
          <div className="col-span-full bg-white rounded-3xl border-2 border-dashed border-slate-200 p-12 text-center">
            <div className="bg-slate-50 w-16 h-16 rounded-full flex items-center justify-center mx-auto mb-4">
              <Building2 className="text-slate-400 w-8 h-8" />
            </div>
            <h3 className="text-lg font-bold text-slate-900">Chưa có phòng ban nào</h3>
            <p className="text-slate-500 mb-6">Bắt đầu bằng việc tạo phòng ban đầu tiên cho tổ chức của bạn.</p>
            <button
              onClick={() => setIsAddModalOpen(true)}
              className="inline-flex items-center gap-2 text-blue-600 font-bold hover:underline"
            >
              <Plus className="w-5 h-5" />
              Tạo ngay
            </button>
          </div>
        )}
      </div>

      {/* Add Modal */}
      {isAddModalOpen && (
        <div className="fixed inset-0 bg-slate-900/40 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-[32px] shadow-2xl w-full max-w-lg overflow-hidden animate-in fade-in zoom-in duration-200">
            <div className="p-8">
              <div className="flex justify-between items-start mb-6">
                <div>
                  <h2 className="text-2xl font-bold text-slate-900">Thêm phòng ban</h2>
                  <p className="text-slate-500">Tạo một đơn vị tổ chức mới</p>
                </div>
                <button 
                  onClick={() => setIsAddModalOpen(false)}
                  className="text-slate-400 hover:text-slate-600 p-1"
                >
                  <Plus className="w-6 h-6 rotate-45" />
                </button>
              </div>

              <div className="space-y-5">
                <div>
                  <label className="block text-sm font-bold text-slate-700 mb-2">Tên phòng ban</label>
                  <input
                    type="text"
                    value={newDept.name}
                    onChange={(e) => setNewDept({ ...newDept, name: e.target.value })}
                    placeholder="VD: Phòng Kỹ thuật"
                    className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all"
                  />
                </div>

                <div>
                  <label className="block text-sm font-bold text-slate-700 mb-2">Mô tả</label>
                  <textarea
                    value={newDept.description}
                    onChange={(e) => setNewDept({ ...newDept, description: e.target.value })}
                    placeholder="Mô tả chức năng, nhiệm vụ..."
                    className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all h-24 resize-none"
                  />
                </div>

                <div>
                  <label className="block text-sm font-bold text-slate-700 mb-2">Trưởng phòng (Manager)</label>
                  <select
                    value={newDept.manager_id}
                    onChange={(e) => setNewDept({ ...newDept, manager_id: e.target.value })}
                    className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all appearance-none"
                  >
                    <option value="">-- Chọn nhân viên --</option>
                    {employees?.map(emp => (
                      <option key={emp.id} value={emp.id}>{emp.first_name} {emp.last_name}</option>
                    ))}
                  </select>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4 mt-8">
                <button
                  onClick={() => setIsAddModalOpen(false)}
                  className="px-6 py-3 border border-slate-200 text-slate-600 font-bold rounded-2xl hover:bg-slate-50 transition-all"
                >
                  Hủy
                </button>
                <button
                  onClick={() => createMutation.mutate(newDept)}
                  disabled={!newDept.name || createMutation.isPending}
                  className="px-6 py-3 bg-blue-600 text-white font-bold rounded-2xl hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed shadow-lg shadow-blue-100 transition-all"
                >
                  {createMutation.isPending ? "Đang lưu..." : "Lưu phòng ban"}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
