import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { fetchRawAttendanceLogs, fetchEmployees, fetchWorkspace, getCurrentWorkspaceId } from "@/lib/api";
import { format } from "date-fns";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { 
  ChevronLeft, 
  ChevronRight, 
  Filter, 
  Download,
  Search,
  Calendar,
  Upload,
  Clock,
  HardDrive,
  User,
  Users
} from "lucide-react";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import AttendanceImportDialog from "@/components/AttendanceImportDialog";
import { Combobox } from "@/components/ui/Combobox";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// Simplified UI Components inline
const Badge = ({ children, variant = "default", className }: { children: React.ReactNode, variant?: "default" | "secondary" | "outline" | "destructive", className?: string }) => {
  const variants = {
    default: "bg-slate-900 text-slate-50 hover:bg-slate-900/80",
    secondary: "bg-slate-100 text-slate-900 hover:bg-slate-100/80",
    outline: "text-slate-950 border border-slate-200 hover:bg-slate-100",
    destructive: "bg-red-500 text-slate-50 hover:bg-red-500/80"
  };
  return (
    <div className={cn("inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-slate-950 focus:ring-offset-2", variants[variant], className)}>
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
  const { formatDate, formatTime } = useDateTimeFormatter();
  const [page, setPage] = useState(1);
  const [limit] = useState(20);
  const [employeeId, setEmployeeId] = useState<string>("all");

  // Helper to get month range
  const getMonthRange = (offset: number = 0) => {
    const d = new Date();
    d.setMonth(d.getMonth() + offset);
    const firstDay = new Date(d.getFullYear(), d.getMonth(), 1).toISOString().split('T')[0];
    const lastDay = new Date(d.getFullYear(), d.getMonth() + 1, 0).toISOString().split('T')[0];
    return { start: firstDay, end: lastDay };
  };

  const [filterMode, setFilterMode] = useState<"this_month" | "last_month" | "custom">("this_month");
  const [dateRange, setDateRange] = useState(getMonthRange(0));

  const handleFilterModeChange = (mode: "this_month" | "last_month" | "custom") => {
    setFilterMode(mode);
    setPage(1);
    if (mode === "this_month") {
      setDateRange(getMonthRange(0));
    } else if (mode === "last_month") {
      setDateRange(getMonthRange(-1));
    }
  };

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

  const { data, isLoading } = useQuery({
    queryKey: ["attendance-raw-logs", wsId, employeeId, dateRange, page],
    queryFn: () => {
      // Correctly handle date boundaries in local time to UTC conversion
      const from = dateRange.start ? new Date(`${dateRange.start}T00:00:00`).toISOString() : undefined;
      const to = dateRange.end ? new Date(`${dateRange.end}T23:59:59`).toISOString() : undefined;
      
      return fetchRawAttendanceLogs({
        employee_id: employeeId === "all" ? undefined : employeeId,
        from,
        to,
        page,
        limit,
      });
    },
    enabled: !!wsId,
  });

  const getEmployeeName = (id: string) => {
    const emp = employees?.find(e => e.id === id);
    return emp ? `${emp.first_name} ${emp.last_name}` : "Hệ thống";
  };

  const getEmployeeID = (id: string) => {
    const emp = employees?.find(e => e.id === id);
    return emp ? emp.employee_id_number : "";
  };

  const totalPages = Math.ceil((data?.total || 0) / limit);

  return (
    <div className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-700">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-slate-900">Nhật ký quẹt thẻ</h1>
          <p className="text-slate-500 mt-1">Dữ liệu thô từ máy chấm công và các nguồn quẹt thẻ.</p>
        </div>
        <div className="flex items-center gap-3">
          <Button variant="outline" onClick={() => window.print()}>
            <Download className="mr-2 h-4 w-4" /> Xuất dữ liệu
          </Button>
          {/* Note: setIsImportDialogOpen used to be here, but using local state now */}
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
        <div className="p-6 space-y-6">
          <div className="flex flex-wrap items-center justify-between gap-4">
            <div className="flex items-center gap-2 bg-slate-100 p-1 rounded-2xl">
              <button
                onClick={() => handleFilterModeChange("this_month")}
                className={`px-4 py-2 rounded-xl text-sm font-bold transition-all ${
                  filterMode === "this_month" ? "bg-white shadow-sm text-blue-600" : "text-slate-500 hover:text-slate-700"
                }`}
              >
                Tháng này
              </button>
              <button
                onClick={() => handleFilterModeChange("last_month")}
                className={`px-4 py-2 rounded-xl text-sm font-bold transition-all ${
                  filterMode === "last_month" ? "bg-white shadow-sm text-blue-600" : "text-slate-500 hover:text-slate-700"
                }`}
              >
                Tháng trước
              </button>
              <button
                onClick={() => handleFilterModeChange("custom")}
                className={`px-4 py-2 rounded-xl text-sm font-bold transition-all ${
                  filterMode === "custom" ? "bg-white shadow-sm text-blue-600" : "text-slate-500 hover:text-slate-700"
                }`}
              >
                Tùy chỉnh
              </button>
            </div>

            <div className="flex-1 max-w-md">
              <Combobox
                options={[
                  { value: "all", label: "Tất cả nhân viên", icon: <Users className="w-4 h-4" /> },
                  ...(employees?.map((emp) => ({
                    value: emp.id,
                    label: `${emp.first_name} ${emp.last_name}`,
                    description: `Mã NV: ${emp.employee_id_number}`,
                    icon: <User className="w-4 h-4" />
                  })) || [])
                ]}
                value={employeeId}
                onChange={(val) => { setEmployeeId(val); setPage(1); }}
                placeholder="Tìm nhân viên..."
                className="bg-slate-50 border-slate-200"
              />
            </div>
          </div>

          {filterMode === "custom" && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-4 border-t border-slate-100 animate-in slide-in-from-top-2 duration-300">
              <div className="space-y-1">
                <label htmlFor="filter-from-date" className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-2">
                  <Calendar className="w-3 h-3" /> Từ ngày
                </label>
                <input 
                  id="filter-from-date"
                  type="date" 
                  value={dateRange.start} 
                  onChange={(e) => { setDateRange(prev => ({ ...prev, start: e.target.value })); setPage(1); }}
                  className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2 text-sm focus:ring-2 focus:ring-blue-500 outline-none transition-all"
                />
              </div>
              <div className="space-y-1">
                <label htmlFor="filter-to-date" className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-2">
                  <Calendar className="w-3 h-3" /> Đến ngày
                </label>
                <input 
                  id="filter-to-date"
                  type="date" 
                  value={dateRange.end} 
                  onChange={(e) => { setDateRange(prev => ({ ...prev, end: e.target.value })); setPage(1); }}
                  className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2 text-sm focus:ring-2 focus:ring-blue-500 outline-none transition-all"
                />
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Table Card */}
      <div className="bg-white rounded-3xl border border-slate-200 shadow-sm overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-slate-50/50 border-b border-slate-100">
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Thời gian</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Mã NV</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Nhân viên</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Nguồn</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Thiết bị</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest text-right">Trạng thái</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {isLoading ? (
                <tr>
                  <td colSpan={6} className="px-6 py-12 text-center">
                    <div className="flex flex-col items-center gap-2">
                      <div className="w-6 h-6 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
                      <span className="text-slate-500 font-medium">Đang tải nhật ký...</span>
                    </div>
                  </td>
                </tr>
              ) : data?.data.length === 0 ? (
                <tr>
                  <td colSpan={6} className="px-6 py-12 text-center text-slate-500 font-medium">
                    Không tìm thấy dữ liệu quẹt thẻ nào.
                  </td>
                </tr>
              ) : (
                data?.data.map((log: any) => (
                  <tr key={log.id} className="hover:bg-slate-50/50 transition-colors group">
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-3">
                        <div className="p-2 bg-blue-50 text-blue-600 rounded-lg">
                          <Clock className="w-4 h-4" />
                        </div>
                        <div>
                          <div className="font-bold text-slate-900">
                            {formatDate(log.punch_time)}
                          </div>
                          <div className="text-xs text-slate-500 font-mono">
                            {formatTime(log.punch_time)}
                          </div>
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <Badge variant="outline">{getEmployeeID(log.employee_id) || "N/A"}</Badge>
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-2">
                        <User className="w-4 h-4" />
                        <div className="font-medium text-slate-700">{getEmployeeName(log.employee_id)}</div>
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <Badge variant="secondary" className="capitalize">
                        {log.source}
                      </Badge>
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-2 text-slate-500 text-sm">
                        <HardDrive className="w-3 h-3" />
                        {log.device_id || "N/A"}
                      </div>
                    </td>
                    <td className="px-6 py-4 text-right">
                      <Badge variant="default" className="bg-emerald-50 text-emerald-700 border-emerald-100">
                        Đã ghi nhận
                      </Badge>
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

      <AttendanceImportDialog 
        isOpen={isImportDialogOpen}
        onClose={() => setIsImportDialogOpen(false)}
        title="Nhập nhật ký quẹt thẻ từ file"
        endpoint="/attendance/logs/upload"
        targetType="log"
      />
    </div>
  );
}
