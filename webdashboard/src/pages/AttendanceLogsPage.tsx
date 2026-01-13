import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { fetchAttendanceLogs, fetchEmployees, fetchWorkspace, getCurrentWorkspaceId, updateTimeEntry, deleteTimeEntry } from "@/lib/api";
import { format } from "date-fns";
import { 
  ChevronLeft, 
  ChevronRight, 
  Filter, 
  Download,
  Search,
  Calendar,
  Edit2,
  Trash2,
  X,
  Save,
  Upload
} from "lucide-react";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import { useToast } from "@/components/Toast";
import AttendanceImportDialog from "@/components/AttendanceImportDialog";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// Simplified UI Components inline to avoid missing files errors
const Badge = ({ children, variant = "default" }: { children: React.ReactNode, variant?: "default" | "secondary" | "outline" | "destructive" }) => {
  const variants = {
    default: "bg-slate-900 text-slate-50 hover:bg-slate-900/80",
    secondary: "bg-slate-100 text-slate-900 hover:bg-slate-100/80",
    outline: "text-slate-950 border border-slate-200 hover:bg-slate-100",
    destructive: "bg-red-500 text-slate-50 hover:bg-red-500/80"
  };
  return (
    <div className={cn("inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-slate-950 focus:ring-offset-2", variants[variant])}>
      {children}
    </div>
  );
};

const Button = ({ children, variant = "default", className, disabled, onClick, type = "button" }: { children: React.ReactNode, variant?: string, className?: string, disabled?: boolean, onClick?: () => void, type?: "button" | "submit" }) => {
  const variants: Record<string, string> = {
    default: "bg-blue-600 text-white hover:bg-blue-700 shadow-sm",
    outline: "border border-slate-200 bg-white hover:bg-slate-50 text-slate-700",
    ghost: "hover:bg-slate-100 text-slate-600",
    destructive: "bg-rose-600 text-white hover:bg-rose-700 shadow-sm",
  };
  return (
    <button 
      type={type}
      disabled={disabled}
      onClick={onClick}
      className={cn(
        "inline-flex items-center justify-center rounded-xl px-4 py-2 text-sm font-semibold transition-all active:scale-95 disabled:opacity-50 disabled:pointer-events-none",
        variants[variant || "default"],
        className
      )}
    >
      {children}
    </button>
  );
};

export default function AttendanceLogsPage() {
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const toast = useToast();
  const [page, setPage] = useState(1);
  const [limit] = useState(20);
  const [employeeId, setEmployeeId] = useState<string>("all");
  const [fromDate, setFromDate] = useState<string>(format(new Date().setDate(new Date().getDate() - 30), "yyyy-MM-dd"));
  const [toDate, setToDate] = useState<string>(format(new Date(), "yyyy-MM-dd"));

  const [editingLog, setEditingLog] = useState<any>(null);
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);

  const { data: employees } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId!),
    enabled: !!wsId,
  });

  const { data: workspace } = useQuery({
    queryKey: ["workspace", wsId],
    queryFn: () => fetchWorkspace(wsId!),
    enabled: !!wsId,
  });

  const timeFormat = workspace?.time_format === "12h" ? "hh:mm:ss a" : "HH:mm:ss";

  const { data, isLoading } = useQuery({
    queryKey: ["attendance-logs", wsId, employeeId, fromDate, toDate, page],
    queryFn: () => fetchAttendanceLogs({
      employee_id: employeeId === "all" ? undefined : employeeId,
      from: fromDate ? new Date(fromDate).toISOString() : undefined,
      to: toDate ? new Date(toDate + "T23:59:59").toISOString() : undefined,
      page,
      limit,
    }),
    enabled: !!wsId,
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string, data: any }) => updateTimeEntry(wsId!, id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["attendance-logs"] });
      toast.success("Đã cập nhật log chấm công thành công.");
      setEditingLog(null);
    },
    onError: (e: any) => toast.error(e.message)
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteTimeEntry(wsId!, id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["attendance-logs"] });
      toast.success("Đã xóa log chấm công.");
    },
    onError: (e: any) => toast.error(e.message)
  });

  const getEmployeeName = (id: string) => {
    const emp = employees?.find(e => e.id === id);
    return emp ? `${emp.first_name} ${emp.last_name}` : id;
  };

  const handleUpdate = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const punchInDate = formData.get("punch_in_date") as string;
    const punchInTime = formData.get("punch_in_time") as string;
    const punchOutDate = formData.get("punch_out_date") as string;
    const punchOutTime = formData.get("punch_out_time") as string;

    const punchIn = new Date(`${punchInDate}T${punchInTime}`).toISOString();
    const punchOut = punchOutTime ? new Date(`${punchOutDate}T${punchOutTime}`).toISOString() : null;

    updateMutation.mutate({
      id: editingLog.id,
      data: {
        punch_in_time: punchIn,
        punch_out_time: punchOut,
        entry_type: formData.get("entry_type") as string,
        note: formData.get("note") as string,
      }
    });
  };

  const totalPages = Math.ceil((data?.total || 0) / limit);

  return (
    <div className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-700">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-slate-900">Nhật ký chấm công</h1>
          <p className="text-slate-500 mt-1">Xem và lọc chi tiết các bản ghi vào/ra của nhân viên.</p>
        </div>
        <div className="flex items-center gap-3">
          <Button variant="outline" onClick={() => window.print()}>
            <Download className="mr-2 h-4 w-4" /> Xuất dữ liệu
          </Button>
          <Button onClick={() => setIsImportDialogOpen(true)} className="bg-emerald-600 hover:bg-emerald-700">
            <Upload className="w-4 h-4 mr-2" /> Nhập file
          </Button>
        </div>
      </div>

      {/* Filters Card */}
      <div className="bg-white rounded-3xl border border-slate-200 shadow-sm overflow-hidden">
        <div className="p-6 border-b border-slate-100 bg-slate-50/50">
          <div className="flex items-center gap-2">
            <Filter className="h-5 w-5 text-blue-600" />
            <h3 className="font-bold text-slate-900">Bộ lọc</h3>
          </div>
        </div>
        <div className="p-6 grid grid-cols-1 md:grid-cols-4 gap-6">
          <div className="space-y-2">
            <label htmlFor="filter-employee" className="text-xs font-bold text-slate-500 uppercase tracking-widest flex items-center gap-2">
              <Search className="w-3 h-3" /> Nhân viên
            </label>
            <select 
              id="filter-employee"
              value={employeeId} 
              onChange={(e) => { setEmployeeId(e.target.value); setPage(1); }}
              className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 text-sm focus:ring-2 focus:ring-blue-500 outline-none transition-all"
            >
              <option value="all">Tất cả nhân viên</option>
              {employees?.map((emp) => (
                <option key={emp.id} value={emp.id}>
                  {emp.first_name} {emp.last_name} ({emp.employee_id_number})
                </option>
              ))}
            </select>
          </div>
          <div className="space-y-2">
            <label htmlFor="filter-from-date" className="text-xs font-bold text-slate-500 uppercase tracking-widest flex items-center gap-2">
              <Calendar className="w-3 h-3" /> Từ ngày
            </label>
            <input 
              id="filter-from-date"
              type="date" 
              value={fromDate} 
              onChange={(e) => { setFromDate(e.target.value); setPage(1); }}
              className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 text-sm focus:ring-2 focus:ring-blue-500 outline-none transition-all"
            />
          </div>
          <div className="space-y-2">
            <label htmlFor="filter-to-date" className="text-xs font-bold text-slate-500 uppercase tracking-widest flex items-center gap-2">
              <Calendar className="w-3 h-3" /> Đến ngày
            </label>
            <input 
              id="filter-to-date"
              type="date" 
              value={toDate} 
              onChange={(e) => { setToDate(e.target.value); setPage(1); }}
              className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 text-sm focus:ring-2 focus:ring-blue-500 outline-none transition-all"
            />
          </div>
          <div className="flex items-end">
            <Button variant="ghost" className="w-full" onClick={() => {
              setEmployeeId("all");
              setFromDate(format(new Date().setDate(new Date().getDate() - 30), "yyyy-MM-dd"));
              setToDate(format(new Date(), "yyyy-MM-dd"));
              setPage(1);
            }}>
              Đặt lại bộ lọc
            </Button>
          </div>
        </div>
      </div>

      {/* Table Card */}
      <div className="bg-white rounded-3xl border border-slate-200 shadow-sm overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-slate-50/50 border-b border-slate-100">
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Ngày</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Nhân viên</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Punch In</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Punch Out</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest text-center">Thời lượng</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest text-center">Loại</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Ghi chú</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest text-right">Thao tác</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {isLoading ? (
                <tr>
                  <td colSpan={8} className="px-6 py-12 text-center">
                    <div className="flex flex-col items-center gap-2">
                      <div className="w-6 h-6 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
                      <span className="text-slate-500 font-medium">Đang tải nhật ký...</span>
                    </div>
                  </td>
                </tr>
              ) : data?.data.length === 0 ? (
                <tr>
                  <td colSpan={8} className="px-6 py-12 text-center text-slate-500 font-medium">
                    Không tìm thấy nhật ký nào trong khoảng thời gian đã chọn.
                  </td>
                </tr>
              ) : (
                data?.data.map((log: any) => (
                  <tr key={log.id} className="hover:bg-slate-50/50 transition-colors group">
                    <td className="px-6 py-4">
                      <div className="font-semibold text-slate-900">
                        {format(new Date(log.punch_in_time), "dd/MM/yyyy")}
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <div className="font-medium text-slate-700">{getEmployeeName(log.employee_id)}</div>
                    </td>
                    <td className="px-6 py-4 font-mono text-sm text-blue-600 font-bold">
                      {format(new Date(log.punch_in_time), timeFormat)}
                    </td>
                    <td className="px-6 py-4 font-mono text-sm text-slate-600">
                      {log.punch_out_time ? format(new Date(log.punch_out_time), timeFormat) : "--:--"}
                    </td>
                    <td className="px-6 py-4 text-center">
                      {log.duration_minutes ? (
                        <Badge variant="secondary">
                          {Math.floor(log.duration_minutes / 60)}h {log.duration_minutes % 60}m
                        </Badge>
                      ) : "-"}
                    </td>
                    <td className="px-6 py-4 text-center">
                      <Badge variant={log.entry_type === "work" ? "default" : "outline"}>
                        {log.entry_type}
                      </Badge>
                    </td>
                    <td className="px-6 py-4 max-w-[150px] truncate text-slate-500 text-sm" title={log.note || ""}>
                      {log.note || "-"}
                    </td>
                    <td className="px-6 py-4 text-right">
                      <div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                        <button 
                          onClick={() => setEditingLog(log)}
                          title="Sửa bản ghi"
                          className="p-2 text-slate-400 hover:text-blue-600 hover:bg-blue-50 rounded-xl transition-all"
                        >
                          <Edit2 className="w-4 h-4" />
                        </button>
                        <button 
                          onClick={() => { if(confirm("Xóa bản ghi này?")) deleteMutation.mutate(log.id); }}
                          title="Xóa bản ghi"
                          className="p-2 text-slate-400 hover:text-rose-600 hover:bg-rose-50 rounded-xl transition-all"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>

        {/* Pagination */}
        <div className="p-6 bg-slate-50/50 border-t border-slate-100 flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div className="text-sm text-slate-500 font-medium">
            Hiển thị <span className="text-slate-900 font-bold">{data?.data.length || 0}</span> trên <span className="text-slate-900 font-bold">{data?.total || 0}</span> bản ghi
          </div>
          <div className="flex items-center gap-4">
            <Button 
              variant="outline" 
              disabled={page === 1} 
              onClick={() => setPage(p => p - 1)}
            >
              <ChevronLeft className="h-4 w-4 mr-2" /> Trước
            </Button>
            <div className="text-sm font-bold text-slate-900">
              Trang {page} / {totalPages || 1}
            </div>
            <Button 
              variant="outline" 
              disabled={page >= totalPages} 
              onClick={() => setPage(p => p + 1)}
            >
              Sau <ChevronRight className="h-4 w-4 ml-2" />
            </Button>
          </div>
        </div>
      </div>

      {/* Edit Modal */}
      {editingLog && (
        <div className="fixed inset-0 bg-slate-900/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-3xl shadow-2xl w-full max-w-md overflow-hidden animate-in zoom-in duration-200">
            <div className="px-6 py-4 border-b border-slate-100 flex justify-between items-center bg-slate-50/50">
              <h2 className="text-xl font-bold">Sửa nhật ký chấm công</h2>
              <button onClick={() => setEditingLog(null)} className="p-2 hover:bg-white rounded-xl transition-colors">
                <X className="w-5 h-5 text-slate-400" />
              </button>
            </div>
            <form onSubmit={handleUpdate} className="p-6 space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1">
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-widest">Ngày vào</label>
                  <input type="date" name="punch_in_date" required defaultValue={format(new Date(editingLog.punch_in_time), "yyyy-MM-dd")} className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none" />
                </div>
                <div className="space-y-1">
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-widest">Giờ vào</label>
                  <input type="time" name="punch_in_time" step="1" lang={workspace?.time_format === "12h" ? "en-US" : "vi-VN"} required defaultValue={format(new Date(editingLog.punch_in_time), "HH:mm:ss")} className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none" />
                </div>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1">
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-widest">Ngày ra</label>
                  <input type="date" name="punch_out_date" defaultValue={editingLog.punch_out_time ? format(new Date(editingLog.punch_out_time), "yyyy-MM-dd") : ""} className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none" />
                </div>
                <div className="space-y-1">
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-widest">Giờ ra</label>
                  <input type="time" name="punch_out_time" step="1" lang={workspace?.time_format === "12h" ? "en-US" : "vi-VN"} defaultValue={editingLog.punch_out_time ? format(new Date(editingLog.punch_out_time), "HH:mm:ss") : ""} className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none" />
                </div>
              </div>
              <div className="space-y-1">
                <label className="text-xs font-bold text-slate-500 uppercase tracking-widest">Loại</label>
                <select name="entry_type" defaultValue={editingLog.entry_type} className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none">
                  <option value="work">Work</option>
                  <option value="break">Break</option>
                  <option value="overtime">Overtime</option>
                </select>
              </div>
              <div className="space-y-1">
                <label className="text-xs font-bold text-slate-500 uppercase tracking-widest">Ghi chú</label>
                <textarea name="note" defaultValue={editingLog.note || ""} className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none h-20 resize-none" />
              </div>
              <div className="pt-4 flex gap-3">
                <Button variant="outline" className="flex-1" onClick={() => setEditingLog(null)}>Hủy</Button>
                <Button type="submit" className="flex-1 flex items-center justify-center gap-2" disabled={updateMutation.isPending}>
                  <Save className="w-4 h-4" />
                  Lưu thay đổi
                </Button>
              </div>
            </form>
          </div>
        </div>
      )}

      <AttendanceImportDialog 
        isOpen={isImportDialogOpen}
        onClose={() => setIsImportDialogOpen(false)}
        title="Nhập nhật ký quẹt thẻ từ file"
        endpoint="/attendance/logs/upload"
      />
    </div>
  );
}
