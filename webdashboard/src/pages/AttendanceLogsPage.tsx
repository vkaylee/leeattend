import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { fetchAttendanceLogs, fetchEmployees, getCurrentWorkspaceId } from "@/lib/api";
import { format } from "date-fns";
import { 
  ChevronLeft, 
  ChevronRight, 
  Filter, 
  Download,
  Search,
  Calendar
} from "lucide-react";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

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

const Button = ({ children, variant = "default", className, disabled, onClick }: { children: React.ReactNode, variant?: string, className?: string, disabled?: boolean, onClick?: () => void }) => {
  const variants: Record<string, string> = {
    default: "bg-blue-600 text-white hover:bg-blue-700 shadow-sm",
    outline: "border border-slate-200 bg-white hover:bg-slate-50 text-slate-700",
    ghost: "hover:bg-slate-100 text-slate-600",
  };
  return (
    <button 
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
  const [page, setPage] = useState(1);
  const [limit] = useState(20);
  const [employeeId, setEmployeeId] = useState<string>("all");
  const [fromDate, setFromDate] = useState<string>(format(new Date().setDate(new Date().getDate() - 30), "yyyy-MM-dd"));
  const [toDate, setToDate] = useState<string>(format(new Date(), "yyyy-MM-dd"));

  const { data: employees } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId!),
    enabled: !!wsId,
  });

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

  const getEmployeeName = (id: string) => {
    const emp = employees?.find(e => e.id === id);
    return emp ? `${emp.first_name} ${emp.last_name}` : id;
  };

  const totalPages = Math.ceil((data?.total || 0) / limit);

  return (
    <div className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-700">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-slate-900">Attendance Logs</h1>
          <p className="text-slate-500 mt-1">View and filter detailed punch-in/out records.</p>
        </div>
        <div className="flex items-center gap-3">
          <Button variant="outline" onClick={() => window.print()}>
            <Download className="mr-2 h-4 w-4" /> Export Data
          </Button>
        </div>
      </div>

      {/* Filters Card */}
      <div className="bg-white rounded-3xl border border-slate-200 shadow-sm overflow-hidden">
        <div className="p-6 border-b border-slate-100 bg-slate-50/50">
          <div className="flex items-center gap-2">
            <Filter className="h-5 w-5 text-blue-600" />
            <h3 className="font-bold text-slate-900">Filters</h3>
          </div>
        </div>
        <div className="p-6 grid grid-cols-1 md:grid-cols-4 gap-6">
          <div className="space-y-2">
            <label className="text-xs font-bold text-slate-500 uppercase tracking-widest flex items-center gap-2">
              <Search className="w-3 h-3" /> Employee
            </label>
            <select 
              value={employeeId} 
              onChange={(e) => { setEmployeeId(e.target.value); setPage(1); }}
              className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 text-sm focus:ring-2 focus:ring-blue-500 outline-none transition-all"
            >
              <option value="all">All Employees</option>
              {employees?.map((emp) => (
                <option key={emp.id} value={emp.id}>
                  {emp.first_name} {emp.last_name} ({emp.employee_id_number})
                </option>
              ))}
            </select>
          </div>
          <div className="space-y-2">
            <label className="text-xs font-bold text-slate-500 uppercase tracking-widest flex items-center gap-2">
              <Calendar className="w-3 h-3" /> From Date
            </label>
            <input 
              type="date" 
              value={fromDate} 
              onChange={(e) => { setFromDate(e.target.value); setPage(1); }}
              className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 text-sm focus:ring-2 focus:ring-blue-500 outline-none transition-all"
            />
          </div>
          <div className="space-y-2">
            <label className="text-xs font-bold text-slate-500 uppercase tracking-widest flex items-center gap-2">
              <Calendar className="w-3 h-3" /> To Date
            </label>
            <input 
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
              Reset Filters
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
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Date</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Employee</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Punch In</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Punch Out</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Duration</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Type</th>
                <th className="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-widest">Note</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {isLoading ? (
                <tr>
                  <td colSpan={7} className="px-6 py-12 text-center">
                    <div className="flex flex-col items-center gap-2">
                      <div className="w-6 h-6 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
                      <span className="text-slate-500 font-medium">Loading logs...</span>
                    </div>
                  </td>
                </tr>
              ) : data?.data.length === 0 ? (
                <tr>
                  <td colSpan={7} className="px-6 py-12 text-center text-slate-500 font-medium">
                    No logs found for the selected period.
                  </td>
                </tr>
              ) : (
                data?.data.map((log: any) => (
                  <tr key={log.id} className="hover:bg-slate-50/50 transition-colors">
                    <td className="px-6 py-4">
                      <div className="font-semibold text-slate-900">
                        {format(new Date(log.punch_in_time), "dd/MM/yyyy")}
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <div className="font-medium text-slate-700">{getEmployeeName(log.employee_id)}</div>
                    </td>
                    <td className="px-6 py-4 font-mono text-sm text-blue-600 font-bold">
                      {format(new Date(log.punch_in_time), "HH:mm:ss")}
                    </td>
                    <td className="px-6 py-4 font-mono text-sm text-slate-600">
                      {log.punch_out_time ? format(new Date(log.punch_out_time), "HH:mm:ss") : "--:--"}
                    </td>
                    <td className="px-6 py-4">
                      {log.duration_minutes ? (
                        <Badge variant="secondary">
                          {Math.floor(log.duration_minutes / 60)}h {log.duration_minutes % 60}m
                        </Badge>
                      ) : "-"}
                    </td>
                    <td className="px-6 py-4">
                      <Badge variant={log.entry_type === "work" ? "default" : "outline"}>
                        {log.entry_type}
                      </Badge>
                    </td>
                    <td className="px-6 py-4 max-w-[200px] truncate text-slate-500 text-sm" title={log.note || ""}>
                      {log.note || "-"}
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
            Showing <span className="text-slate-900 font-bold">{data?.data.length || 0}</span> of <span className="text-slate-900 font-bold">{data?.total || 0}</span> attendance logs
          </div>
          <div className="flex items-center gap-4">
            <Button 
              variant="outline" 
              disabled={page === 1} 
              onClick={() => setPage(p => p - 1)}
            >
              <ChevronLeft className="h-4 w-4 mr-2" /> Previous
            </Button>
            <div className="text-sm font-bold text-slate-900">
              Page {page} of {totalPages || 1}
            </div>
            <Button 
              variant="outline" 
              disabled={page >= totalPages} 
              onClick={() => setPage(p => p + 1)}
            >
              Next <ChevronRight className="h-4 w-4 ml-2" />
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}