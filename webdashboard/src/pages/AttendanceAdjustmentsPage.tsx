import React from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { CheckCircle, XCircle, Clock, User, MessageSquare, Upload, FileDown } from "lucide-react";
import { 
  fetchWorkspaceAttendanceAdjustments, 
  approveAttendanceAdjustment, 
  getCurrentWorkspaceId,
  AttendanceAdjustmentRequest,
  fetchEmployees,
  uploadAttendanceExcel,
  fetchLatestImportJob,
  AttendanceImportJob
} from "../lib/api";

export default function AttendanceAdjustmentsPage() {
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const fileInputRef = React.useRef<HTMLInputElement>(null);

  // Poll for latest import job status
  const { data: latestJob } = useQuery({
    queryKey: ["latest-import-job", wsId],
    queryFn: () => fetchLatestImportJob(wsId!),
    enabled: !!wsId,
    refetchInterval: (query) => {
      // Poll every 2 seconds if the job is still processing
      const job = query.state.data as AttendanceImportJob | null;
      return job && job.status === "processing" ? 2000 : false;
    },
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
      if (fileInputRef.current) fileInputRef.current.value = "";
    },
    onError: (error: any) => {
      alert("Lỗi khi tải file: " + error.message);
      if (fileInputRef.current) fileInputRef.current.value = "";
    }
  });

  // When job finishes, refresh adjustment list
  React.useEffect(() => {
    if (latestJob && latestJob.status !== "processing") {
      queryClient.invalidateQueries({ queryKey: ["attendance-adjustments", wsId] });
      queryClient.invalidateQueries({ queryKey: ["time-history"] });
    }
  }, [latestJob?.status, wsId, queryClient]);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      uploadMutation.mutate(file);
    }
  };

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
      {latestJob && latestJob.status === "processing" && (
        <div className="bg-blue-50 border border-blue-100 rounded-3xl p-6 flex items-center justify-between animate-pulse">
          <div className="flex items-center gap-4">
            <div className="w-12 h-12 bg-blue-600 rounded-2xl flex items-center justify-center">
              <Upload className="w-6 h-6 text-white" />
            </div>
            <div>
              <h3 className="font-bold text-blue-900">Đang xử lý file chấm công...</h3>
              <p className="text-blue-700 text-sm">
                Đã xử lý {latestJob.processed_rows} / {latestJob.total_rows} dòng
              </p>
            </div>
          </div>
          <div className="w-48 bg-blue-200 h-2 rounded-full overflow-hidden">
            <div 
              className="bg-blue-600 h-full transition-all duration-500" 
              style={{ width: `${(latestJob.processed_rows / (latestJob.total_rows || 1)) * 100}%` }}
            />
          </div>
        </div>
      )}

      {latestJob && latestJob.status === "completed" && (
        <div className="bg-green-50 border border-green-100 rounded-3xl p-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <CheckCircle className="w-5 h-5 text-green-600" />
            <span className="text-green-800 text-sm font-medium">
              File tải lên gần nhất đã hoàn tất ({latestJob.processed_rows} dòng).
            </span>
          </div>
          <button 
            onClick={() => queryClient.setQueryData(["latest-import-job", wsId], null)}
            className="text-green-600 hover:text-green-800 text-xs font-bold"
          >
            Đóng
          </button>
        </div>
      )}

      {latestJob && latestJob.status === "failed" && (
        <div className="bg-red-50 border border-red-100 rounded-3xl p-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <XCircle className="w-5 h-5 text-red-600" />
            <div className="text-red-800 text-sm">
              <span className="font-bold">Lỗi xử lý file:</span> {latestJob.error_log?.split('\n')[0] || "Lỗi không xác định"}
            </div>
          </div>
          <button 
            onClick={() => queryClient.setQueryData(["latest-import-job", wsId], null)}
            className="text-red-600 hover:text-red-800 text-xs font-bold"
          >
            Đóng
          </button>
        </div>
      )}

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
                      <input
                        type="file"
                        accept=".csv,.xlsx,.xls"
                        onChange={handleFileChange}
                        className="hidden"
                        ref={fileInputRef}
                      />
                      <button 
            onClick={() => fileInputRef.current?.click()}
            disabled={uploadMutation.isPending}
            className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-2xl hover:bg-blue-700 transition-all font-semibold shadow-sm shadow-blue-200 text-sm disabled:opacity-50"
          >
            <Upload className="w-4 h-4" />
            {uploadMutation.isPending ? "Đang tải..." : "Nhập Excel"}
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
                      {new Date(req.requested_punch_in_time).toLocaleTimeString('vi-VN')} - 
                      {req.requested_punch_out_time ? new Date(req.requested_punch_out_time).toLocaleTimeString('vi-VN') : "?"}
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
    </div>
  );
}
