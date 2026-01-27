import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { fetchRawAttendanceLogs, fetchEmployees, fetchWorkspace, getCurrentWorkspaceId, fetchDevices } from "@/lib/api";
import { format, endOfMonth } from "date-fns";
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
  Users,
  ChevronDown,
  CalendarRange
} from "lucide-react";
import { cn } from "@/lib/utils";
import AttendanceImportDialog from "@/components/AttendanceImportDialog";
import { EmployeeSearch } from "@/components/common/EmployeeSearch";
import { MonthYearPicker } from "@/components/common/MonthYearPicker";

// Simplified UI Components inline
const Badge = ({ children, variant = "default", className }: { children: React.ReactNode, variant?: "default" | "secondary" | "outline" | "destructive", className?: string }) => {
  const variants = {
    default: "bg-slate-900 text-slate-50 hover:bg-slate-900/80",
    secondary: "bg-slate-100 text-slate-700 hover:bg-slate-100/80",
    outline: "text-slate-700 border border-slate-200 hover:bg-slate-100",
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
    default: "bg-primary-600 text-white hover:bg-primary-700 shadow-sm",
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
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const { formatDate, formatTime } = useDateTimeFormatter();
  const [page, setPage] = useState(1);
  const [limit] = useState(20);
  const [employeeId, setEmployeeId] = useState<string>("all");

  const getMonthRange = (offset: number = 0) => {
    const d = new Date(); d.setMonth(d.getMonth() + offset);
    const firstDay = format(new Date(d.getFullYear(), d.getMonth(), 1), 'yyyy-MM-dd');
    const lastDay = format(new Date(d.getFullYear(), d.getMonth() + 1, 0), 'yyyy-MM-dd');
    return { start: firstDay, end: lastDay };
  };

  const [filterMode, setFilterMode] = useState<"this_month" | "last_month" | "custom" | "specific_month">("this_month");
  const [dateRange, setDateRange] = useState(getMonthRange(0));
  const [specificMonth, setSpecificMonth] = useState({ month: new Date().getMonth() + 1, year: new Date().getFullYear() });
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);

  const handleFilterModeChange = (mode: "this_month" | "last_month" | "custom" | "specific_month") => {
    setFilterMode(mode); setPage(1);
    if (mode === "this_month") setDateRange(getMonthRange(0));
    else if (mode === "last_month") setDateRange(getMonthRange(-1));
  };

  const handleSpecificMonthChange = (m: number, y: number) => {
    setSpecificMonth({ month: m, year: y }); setFilterMode("specific_month"); setPage(1);
    const start = format(new Date(y, m - 1, 1), 'yyyy-MM-dd');
    const end = format(endOfMonth(new Date(y, m - 1, 1)), 'yyyy-MM-dd');
    setDateRange({ start, end });
  };

  const { data: employeesData } = useQuery({ queryKey: ["employees", wsId], queryFn: () => fetchEmployees(wsId!, 1, 3000), enabled: !!wsId });
  const employees = employeesData?.data || [];
  const { data, isLoading } = useQuery({
    queryKey: ["attendance-raw-logs", wsId, employeeId, dateRange, page],
    queryFn: () => {
      const from = dateRange.start ? new Date(`${dateRange.start}T00:00:00`).toISOString() : undefined;
      const to = dateRange.end ? new Date(`${dateRange.end}T23:59:59`).toISOString() : undefined;
      return fetchRawAttendanceLogs({ employee_id: employeeId === "all" ? undefined : employeeId, from, to, page, limit });
    },
    enabled: !!wsId,
  });

  const { data: devicesData } = useQuery({
    queryKey: ["devices", wsId],
    queryFn: () => fetchDevices(),
    enabled: !!wsId,
  });

  const getDeviceName = (id?: string) => {
    if (!id) return null;
    const device = devicesData?.find(d => d.id === id);
    return device ? device.name : null;
  };

  const getEmployeeName = (id: string) => { const emp = employees.find(e => e.id === id); return emp ? emp.full_name : t("common.system"); };
  const getEmployeeID = (id: string) => { const emp = employees.find(e => e.id === id); return emp ? emp.employee_id_number : ""; };
  const totalPages = Math.ceil((data?.total || 0) / limit);

  return (
    <div className="space-y-8 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div>
          <h1 data-testid="attendance-logs-title" className="text-3xl font-bold tracking-tight text-slate-700">{t("attendance_logs.title")}</h1>
          <p className="text-slate-500 mt-1 font-medium">{t("attendance_logs.subtitle")}</p>
        </div>
        <div className="flex items-center gap-3">
          <button onClick={() => window.print()} className="flex items-center gap-2 bg-white border border-slate-200 text-slate-700 px-5 py-2.5 rounded-2xl hover:bg-slate-50 transition-all font-bold text-[10px] uppercase tracking-widest shadow-sm"><Download size={14} /> {t("attendance_logs.export_btn")}</button>
          <button onClick={() => setIsImportDialogOpen(true)} className="flex items-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 font-bold text-[10px] uppercase tracking-widest active:scale-95"><Upload size={14} /> {t("attendance_logs.import_btn")}</button>
        </div>
      </div>

      <div className="bg-white rounded-[2rem] border border-slate-200 shadow-sm p-2 flex flex-col xl:flex-row xl:items-center justify-between gap-4">
        <div className="flex items-center bg-slate-100/50 rounded-2xl p-1 shadow-inner border border-slate-200/50">
          {["this_month", "last_month"].map(mode => (
            <button key={mode} onClick={() => handleFilterModeChange(mode as any)} className={cn("px-5 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", filterMode === mode ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}>{t(`common.filters.${mode}`)}</button>
          ))}
          <div className="h-6 w-px bg-slate-200 mx-1" />
          <div className="relative group">
            <button className={cn("flex items-center gap-2 px-5 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", filterMode === "specific_month" ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}>
              <CalendarRange size={14} /> {filterMode === "specific_month" ? `T${specificMonth.month}/${specificMonth.year}` : t("common.filters.specific_month")}
            </button>
            <div className="absolute left-0 top-full mt-2 z-[100] hidden group-hover:block animate-in zoom-in-95 duration-200 origin-top-left"><MonthYearPicker month={specificMonth.month} year={specificMonth.year} onChange={handleSpecificMonthChange} /></div>
          </div>
          <button onClick={() => handleFilterModeChange("custom")} className={cn("px-5 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", filterMode === "custom" ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}>{t("common.filters.custom")}</button>
        </div>

        <div className="flex-1 max-w-md flex items-center gap-3 px-2">
          <div className="flex-1"><EmployeeSearch data-testid="employee-search" workspaceId={wsId!} value={employeeId === "all" ? "" : employeeId} onChange={(val) => { setEmployeeId(val || "all"); setPage(1); }} placeholder={employeeId === "all" ? t("common.all_employees") : undefined} /></div>
          {employeeId !== "all" && <button onClick={() => { setEmployeeId("all"); setPage(1); }} className="p-2.5 bg-slate-100 text-slate-500 hover:text-primary-600 rounded-xl transition-all"><Users size={18} /></button>}
        </div>
      </div>

      {filterMode === "custom" && (
        <div className="bg-white p-6 rounded-[2rem] border border-slate-200 shadow-sm grid grid-cols-1 md:grid-cols-2 gap-8 animate-in slide-in-from-top-2 duration-300 max-w-2xl">
          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1 flex items-center gap-2"><Calendar size={12} /> {t("common.from_date")}</label>
            <input type="date" value={dateRange.start} onChange={(e) => { setDateRange(prev => ({ ...prev, start: e.target.value })); setPage(1); }} className="w-full bg-slate-50 border border-slate-100 rounded-xl px-4 py-3 outline-none focus:ring-4 focus:ring-primary-500/10 transition-all font-bold text-sm" />
          </div>
          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1 flex items-center gap-2"><Calendar size={12} /> {t("common.to_date")}</label>
            <input type="date" value={dateRange.end} onChange={(e) => { setDateRange(prev => ({ ...prev, end: e.target.value })); setPage(1); }} className="w-full bg-slate-50 border border-slate-100 rounded-xl px-4 py-3 outline-none focus:ring-4 focus:ring-primary-500/10 transition-all font-bold text-sm" />
          </div>
        </div>
      )}

      <div className="bg-white rounded-[2.5rem] border border-slate-200 shadow-sm overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-slate-50/50 border-b border-slate-100">
                <th className="px-8 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t("common.time")}</th>
                <th className="px-8 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t("common.employee")}</th>
                <th className="px-8 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t("common.source")}</th>
                <th className="px-8 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest text-right">{t("common.status")}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {isLoading ? (
                <tr><td colSpan={4} className="px-8 py-12 text-center text-slate-500 font-bold italic">{t("common.loading")}</td></tr>
              ) : data?.data.length === 0 ? (
                <tr><td colSpan={4} className="px-8 py-20 text-center text-slate-500 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("attendance_logs.messages.no_data")}</td></tr>
              ) : (
                data?.data.map((log: any) => (
                  <tr key={log.id} className="hover:bg-slate-50/50 transition-colors group">
                    <td className="px-8 py-5">
                      <div className="flex items-center gap-4">
                        <div className="w-10 h-10 rounded-xl bg-primary-50 flex items-center justify-center text-primary-600 shadow-inner border border-primary-100/50"><Clock size={18} /></div>
                        <div>
                          <div className="font-bold text-slate-700 text-sm tracking-tight uppercase">{formatDate(log.punch_time)}</div>
                          <div className="text-[10px] font-bold text-slate-500 font-mono uppercase tracking-tighter">{formatTime(log.punch_time)}</div>
                        </div>
                      </div>
                    </td>
                    <td className="px-8 py-5">
                      <div className="font-bold text-slate-800 text-sm tracking-tight uppercase">{getEmployeeName(log.employee_id)}</div>
                      <div className="text-[9px] font-bold text-primary-600 uppercase tracking-widest mt-0.5">{getEmployeeID(log.employee_id) || "SYSTEM"}</div>
                    </td>
                    <td className="px-8 py-5">
                      <div className="flex flex-col gap-1">
                        <div className="flex items-center gap-2">
                          <span className="px-2.5 py-1 bg-slate-100 text-slate-600 rounded-lg text-[9px] font-bold uppercase tracking-widest border border-slate-200">{log.source}</span>
                          {log.source === "Device" && getDeviceName(log.device_id) && (
                            <span className="text-[10px] font-bold text-slate-700 uppercase">{getDeviceName(log.device_id)}</span>
                          )}
                        </div>
                        {log.device_id && (
                          <div className="text-[8px] font-bold text-slate-400 uppercase tracking-tighter flex items-center gap-1 ml-0.5">
                            <HardDrive size={10} /> {log.device_id}
                          </div>
                        )}
                      </div>
                    </td>
                    <td className="px-8 py-5 text-right">
                      <span className="px-3 py-1 bg-emerald-50 text-emerald-600 rounded-full text-[9px] font-bold uppercase tracking-widest border border-emerald-100 shadow-sm">{t("attendance_logs.status.recorded")}</span>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>

        <div className="p-8 bg-slate-50/50 border-t border-slate-100 flex flex-col md:flex-row md:items-center justify-between gap-6">
          <div className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.15em]">{t("common.pagination.showing")} <span className="text-slate-700 mx-1">{data?.data.length || 0}</span> {t("common.pagination.out_of")} <span className="text-slate-700 mx-1">{data?.total || 0}</span> {t("common.pagination.records")}</div>
          <div className="flex items-center gap-4">
            <button disabled={page === 1} onClick={() => setPage(p => p - 1)} className="flex items-center gap-2 bg-white border border-slate-200 text-slate-600 px-4 py-2 rounded-xl hover:bg-slate-50 transition-all font-bold text-[10px] uppercase tracking-widest disabled:opacity-30 shadow-sm active:scale-95"><ChevronLeft size={14} /> {t("common.pagination.prev")}</button>
            <div className="text-xs font-bold text-slate-700 uppercase tracking-tighter">Page {page} / {totalPages || 1}</div>
            <button disabled={page >= totalPages} onClick={() => setPage(p => p + 1)} className="flex items-center gap-2 bg-white border border-slate-200 text-slate-600 px-4 py-2 rounded-xl hover:bg-slate-50 transition-all font-bold text-[10px] uppercase tracking-widest disabled:opacity-30 shadow-sm active:scale-95">{t("common.pagination.next")} <ChevronRight size={14} /></button>
          </div>
        </div>
      </div>

      <AttendanceImportDialog isOpen={isImportDialogOpen} onClose={() => setIsImportDialogOpen(false)} title={t("attendance_logs.modal.import_title")} endpoint="/attendance/logs/upload" targetType="log" />
    </div>
  );
}
