import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { CheckCircle, XCircle, Clock, MessageSquare, Upload, FileDown } from "lucide-react";
import { 
  fetchWorkspace,
  fetchEmployees,
  fetchWorkspaceAttendanceAdjustments,
  approveAttendanceAdjustment,
  getCurrentWorkspaceId
} from "../lib/api";
import { formatTime } from "../lib/utils";
import AttendanceImportDialog from "@/components/AttendanceImportDialog";

export default function AttendanceAdjustmentsPage() {
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);

  const { data: workspace } = useQuery({
    queryKey: ["workspace", wsId],
    queryFn: () => fetchWorkspace(wsId!),
    enabled: !!wsId,
  });

  const { data: requests = [], isLoading } = useQuery({
    queryKey: ["attendance-adjustments", wsId],
    queryFn: () => fetchWorkspaceAttendanceAdjustments(wsId!),
    enabled: !!wsId,
  });

  const { data: employees = [] } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId!),
    enabled: !!wsId,
  });

  const approveMutation = useMutation({
    mutationFn: ({ id, status }: { id: string; status: string }) => 
      approveAttendanceAdjustment(wsId!, id, { status }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["attendance-adjustments", wsId] });
      queryClient.invalidateQueries({ queryKey: ["time-history"] });
    },
  });

  const uploadMutation = useMutation({
    mutationFn: (file: File) => uploadAttendanceExcel(wsId!, file),
    onSuccess: () => {
      // Invalidate latest job to start polling
      queryClient.invalidateQueries({ queryKey: ["latest-import-job", wsId] });
    },
  });

  const downloadTemplate = () => {
    // Basic CSV template
    const headers = "empNo,empName,empDt";
    const example = "EMP001,Nguyen Van A,11/01/2026 08:00";
    const blob = new Blob([`\uFEFF${headers}\n${example}`], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.setAttribute("download", "attendance_template.csv");
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  const getEmployeeName = (id: string) => {
    const emp = employees.find(e => e.id === id);
    return emp ? `${emp.first_name} ${emp.last_name}` : "Unknown";
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "approved":
        return <span className="px-3 py-1 bg-green-100 text-green-700 rounded-full text-xs font-bold">Đã duyệt</span>;
      case "rejected":
        return <span className="px-3 py-1 bg-red-100 text-red-700 rounded-full text-xs font-bold">Từ chối</span>;
      default:
        return <span className="px-3 py-1 bg-orange-100 text-orange-700 rounded-full text-xs font-bold">Chờ duyệt</span>;
    }
  };

  if (!wsId) return <div className="p-8 text-center">Vui lòng chọn Workspace</div>;

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Xác nhận công</h1>
          <p className="text-slate-500">Phê duyệt các yêu cầu điều chỉnh/bổ sung công từ nhân viên.</p>
        </div>
        <div className="flex items-center gap-2">
          <button 
            onClick={downloadTemplate}
            className="flex items-center gap-2 px-4 py-2 bg-white border border-slate-200 text-slate-600 rounded-2xl hover:bg-slate-50 transition-all font-semibold text-sm"
          >
            <FileDown className="w-4 h-4" />
            Mẫu Excel
          </button>
          <button 
            onClick={() => setIsImportDialogOpen(true)}
            className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-2xl hover:bg-blue-700 transition-all font-semibold shadow-sm shadow-blue-200 text-sm disabled:opacity-50"
          >
            <Upload className="w-4 h-4" />
            Nhập Excel
          </button>
        </div>
      </div>

      <div className="bg-white border border-slate-200 rounded-3xl overflow-hidden shadow-sm">
        <div className="overflow-x-auto">
          <table className="w-full text-left">
            <thead>
              <tr className="bg-slate-50 border-b border-slate-100">
                <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase">Nhân viên</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase">Yêu cầu (Vào - Ra)</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase">Lý do</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase">Trạng thái</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase text-right">Thao tác</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {isLoading ? (
                <tr><td colSpan={5} className="px-6 py-12 text-center text-slate-400 italic">Đang tải...</td></tr>
              ) : requests.length === 0 ? (
                <tr><td colSpan={5} className="px-6 py-12 text-center text-slate-400 italic">Chưa có yêu cầu nào.</td></tr>
              ) : requests.map((req) => (
                <tr key={req.id} className="hover:bg-slate-50/50 transition-colors">
                  <td className="px-6 py-4">
                    <div className="flex items-center gap-3">
                      <div className="w-8 h-8 bg-blue-100 text-blue-600 rounded-full flex items-center justify-center font-bold text-xs">
                        {getEmployeeName(req.employee_id).charAt(0)}
                      </div>
                      <div className="font-semibold text-sm">{getEmployeeName(req.employee_id)}</div>
                    </div>
                  </td>
                  <td className="px-6 py-4">
                    <div className="text-sm font-medium">
                      {new Date(req.requested_punch_in_time).toLocaleDateString('vi-VN')}
                    </div>
                    <div className="text-xs text-slate-500 flex items-center gap-1">
                      <Clock className="w-3 h-3" />
                      {formatTime(new Date(req.requested_punch_in_time).toLocaleTimeString('vi-VN'), workspace?.time_format)} - 
                      {req.requested_punch_out_time ? formatTime(new Date(req.requested_punch_out_time).toLocaleTimeString('vi-VN'), workspace?.time_format) : "?"}
                    </div>
                    {req.is_break_overtime && (
                      <div className="mt-1">
                        <span className="px-2 py-0.5 bg-blue-600 text-white rounded-md text-[10px] font-bold uppercase tracking-wider">
                          Làm thông trưa
                        </span>
                      </div>
                    )}
                  </td>
                  <td className="px-6 py-4 max-w-xs">
                    <div className="text-sm text-slate-600 truncate flex items-center gap-2">
                      <MessageSquare className="w-4 h-4 text-slate-300" />
                      {req.reason || "-"}
                    </div>
                  </td>
                  <td className="px-6 py-4">
                    {getStatusBadge(req.status)}
                  </td>
                  <td className="px-6 py-4 text-right">
                    {req.status === "pending" && (
                      <div className="flex items-center justify-end gap-2">
                        <button 
                          onClick={() => approveMutation.mutate({ id: req.id, status: "rejected" })}
                          disabled={approveMutation.isPending}
                          className="p-2 text-red-600 hover:bg-red-50 rounded-xl transition-colors"
                          title="Từ chối"
                        >
                          <XCircle className="w-6 h-6" />
                        </button>
                        <button 
                          onClick={() => approveMutation.mutate({ id: req.id, status: "approved" })}
                          disabled={approveMutation.isPending}
                          className="p-2 text-green-600 hover:bg-green-50 rounded-xl transition-colors"
                          title="Phê duyệt"
                        >
                          <CheckCircle className="w-6 h-6" />
                        </button>
                      </div>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      <AttendanceImportDialog 
        isOpen={isImportDialogOpen}
        onClose={() => setIsImportDialogOpen(false)}
        title="Nhập dữ liệu chấm công từ file"
        endpoint="/attendance/upload"
      />
    </div>
  );
}
