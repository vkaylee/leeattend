import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  FileBarChart, 
  Download, 
  Calendar, 
  Search, 
  List, 
  TableProperties, 
  Clock, 
  User as UserIcon, 
  CalendarDays, 
  Info, 
  RefreshCw, 
  Bug, 
  ChevronDown, 
  ChevronUp,
  CalendarRange
} from "lucide-react";
import { format, startOfMonth, endOfMonth } from "date-fns";
import {
  fetchAttendanceReport,
  getCurrentWorkspaceId,
  fetchWorkspace,
  AttendanceSummary,
  DailyAttendanceReport,
  reprocessAttendance
} from "../lib/api";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { formatUnitsToHours } from "@/lib/utils/attendance";
import { EmployeeSearch } from "@/components/common/EmployeeSearch";
import Modal from "@/components/ui/Modal";
import { useDebug } from "@/context/DebugContext";
import { MonthYearPicker } from "@/components/common/MonthYearPicker";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

export default function ReportsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const { formatDate, formatTime } = useDateTimeFormatter();
  const queryClient = useQueryClient();
  const { isEnabled: isDebugMode } = useDebug();

  const { data: workspace } = useQuery({
    queryKey: ["workspace", wsId],
    queryFn: () => fetchWorkspace(wsId!),
    enabled: !!wsId,
  });

  const workUnitMinutes = workspace?.work_unit_minutes || 60;

  const getMonthRange = (offset: number = 0) => {
    const d = new Date();
    d.setMonth(d.getMonth() + offset);
    const firstDay = format(new Date(d.getFullYear(), d.getMonth(), 1), 'yyyy-MM-dd');
    const lastDay = format(new Date(d.getFullYear(), d.getMonth() + 1, 0), 'yyyy-MM-dd');
    return { start: firstDay, end: lastDay };
  };

  const [filterMode, setFilterMode] = useState<"this_month" | "last_month" | "custom" | "specific_month">("last_month");
  const [dateRange, setDateRange] = useState(getMonthRange(-1));
  const [specificMonth, setSpecificMonth] = useState({ 
    month: new Date().getMonth() === 0 ? 12 : new Date().getMonth(), 
    year: new Date().getMonth() === 0 ? new Date().getFullYear() - 1 : new Date().getFullYear() 
  });
  
  const [selectedEmployeeIds, setSelectedEmployeeIds] = useState<string[]>([]);
  const [reportType, setReportType] = useState<"summary" | "daily">("summary");
  const [expandedDebugRows, setExpandedDebugRows] = useState<Set<string>>(new Set());
  const [reprocessingRow, setReprocessingRow] = useState<string | null>(null);
  const [confirmDialog, setConfirmDialog] = useState<{ show: boolean; employeeId: string; employeeName: string; date: string; } | null>(null);

  const handleFilterModeChange = (mode: "this_month" | "last_month" | "custom" | "specific_month") => {
    setFilterMode(mode);
    if (mode === "this_month") setDateRange(getMonthRange(0));
    else if (mode === "last_month") setDateRange(getMonthRange(-1));
  };

  const handleSpecificMonthChange = (m: number, y: number) => {
    setSpecificMonth({ month: m, year: y });
    setFilterMode("specific_month");
    const start = format(new Date(y, m - 1, 1), 'yyyy-MM-dd');
    const end = format(endOfMonth(new Date(y, m - 1, 1)), 'yyyy-MM-dd');
    setDateRange({ start, end });
  };

  const toggleExpandRow = (rowKey: string) => {
    const next = new Set(expandedDebugRows);
    if (next.has(rowKey)) next.delete(rowKey);
    else next.add(rowKey);
    setExpandedDebugRows(next);
  };

  const { data: reportData = [], isLoading } = useQuery({
    queryKey: ["attendance-report", wsId, dateRange, reportType],
    queryFn: () => fetchAttendanceReport(wsId!, dateRange.start, dateRange.end, reportType),
    enabled: !!wsId,
  });

  const reprocessMutation = useMutation({
    mutationFn: ({ employeeId, date }: { employeeId: string; date: string }) => reprocessAttendance(wsId!, employeeId, date, date),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
      setReprocessingRow(null); setConfirmDialog(null);
      toast.success(t("common.success"));
    },
  });

  const toast = useToast();

  const filteredData = Array.isArray(reportData) ? reportData.filter((row: any) =>
    selectedEmployeeIds.length === 0 || selectedEmployeeIds.includes(row.employee_id)
  ) : [];

  if (!wsId) return <div className="p-8 text-center">{t("common.select_workspace")}</div>;

  const getStatusBadge = (status: string) => {
    const styles: Record<string, string> = {
      present: "bg-emerald-50 text-emerald-600 border-emerald-100",
      late: "bg-amber-50 text-amber-600 border-amber-100",
      absent: "bg-rose-50 text-rose-600 border-rose-100",
      off: "bg-slate-50 text-slate-400 border-slate-100",
    };
    const labels: Record<string, string> = {
      present: t("reports.status.present"),
      late: t("reports.status.late"),
      absent: t("reports.status.absent"),
      off: t("reports.status.off"),
    };
    return <span className={cn("px-2.5 py-1 rounded-full text-[9px] font-black uppercase tracking-widest border", styles[status] || styles.off)}>{labels[status] || status}</span>;
  };

  return (
    <div className="space-y-8 pb-20 animate-in fade-in duration-500">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-6 px-2">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-slate-900">{t("reports.title")}</h1>
          <p className="text-slate-500 mt-1 font-medium">{t("reports.subtitle")}</p>
        </div>
        <div className="flex items-center gap-3">
          <button className="flex items-center gap-2 bg-white border border-slate-200 text-slate-700 px-5 py-2.5 rounded-2xl hover:bg-slate-50 transition-all font-black text-[10px] uppercase tracking-widest shadow-sm"><Download size={14} /> {t("reports.export_excel")}</button>
        </div>
      </div>

      {/* Tabs & Period Selector Container */}
      <div className="flex flex-col xl:flex-row xl:items-center justify-between gap-6">
        <div className="flex items-center gap-1 bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50">
          <button onClick={() => setReportType("summary")} className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${reportType === 'summary' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700"}`}>
            <List size={14} /> {t("reports.tabs.summary")}
          </button>
          <button onClick={() => setReportType("daily")} className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${reportType === 'daily' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700"}`}>
            <TableProperties size={14} /> {t("reports.tabs.daily")}
          </button>
        </div>

        <div className="flex items-center gap-4 bg-white p-2 rounded-[2rem] border border-slate-200 shadow-sm">
          <div className="flex items-center bg-slate-100/50 rounded-2xl p-1 shadow-inner border border-slate-200/50">
            {["this_month", "last_month"].map(mode => (
              <button key={mode} onClick={() => handleFilterModeChange(mode as any)} className={`px-4 py-2 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all ${filterMode === mode ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700"}`}>{t(`common.filters.${mode}`)}</button>
            ))}
            <div className="h-6 w-px bg-slate-200 mx-1" />
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button className={`flex items-center gap-2 px-4 py-2 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all ${filterMode === "specific_month" ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700"}`}>
                  <CalendarRange size={14} /> {filterMode === "specific_month" ? `T${specificMonth.month}/${specificMonth.year}` : t("common.filters.specific_month")}
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent className="w-80 p-4 rounded-[2rem] shadow-2xl border-slate-100" align="end" sideOffset={8}>
                 <MonthYearPicker month={specificMonth.month} year={specificMonth.year} onChange={handleSpecificMonthChange} />
              </DropdownMenuContent>
            </DropdownMenu>
            <button onClick={() => handleFilterModeChange("custom")} className={`px-4 py-2 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all ${filterMode === "custom" ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700"}`}>{t("common.filters.custom")}</button>
          </div>
          <div className="h-10 w-px bg-slate-100" />
          <div className="w-72">
            <EmployeeSearch workspaceId={wsId!} value={selectedEmployeeIds} onChange={setSelectedEmployeeIds} multiple={true} placeholder={t("reports.search_placeholder")} />
          </div>
        </div>
      </div>

      {filterMode === "custom" && (
        <div className="bg-white p-6 rounded-[2rem] border border-slate-200 shadow-sm grid grid-cols-1 md:grid-cols-2 gap-8 animate-in slide-in-from-top-2 duration-300 max-w-2xl">
          <div className="space-y-2">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-2"><Calendar size={12} /> {t("common.from_date")}</label>
            <input type="date" value={dateRange.start} onChange={(e) => setDateRange(prev => ({ ...prev, start: e.target.value }))} className="w-full bg-slate-50 border border-slate-100 rounded-xl px-4 py-3 outline-none focus:ring-4 focus:ring-primary-500/10 transition-all font-black text-sm" />
          </div>
          <div className="space-y-2">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-2"><Calendar size={12} /> {t("common.to_date")}</label>
            <input type="date" value={dateRange.end} onChange={(e) => setDateRange(prev => ({ ...prev, end: e.target.value }))} className="w-full bg-slate-50 border border-slate-100 rounded-xl px-4 py-3 outline-none focus:ring-4 focus:ring-primary-500/10 transition-all font-black text-sm" />
          </div>
        </div>
      )}

      <div className="bg-white border border-slate-200 rounded-[2.5rem] overflow-hidden shadow-sm">
        {reportType === "summary" ? (
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-slate-50/50 border-b border-slate-100">
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.employee")}</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("reports.table.total_hours")}</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("reports.table.leave_days")}</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.notes")}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {isLoading ? (
                <tr><td colSpan={4} className="px-8 py-12 text-center text-slate-400 font-bold italic">{t("reports.messages.generating")}</td></tr>
              ) : filteredData.length === 0 ? (
                <tr><td colSpan={4} className="px-8 py-20 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("reports.messages.no_data")}</td></tr>
              ) : filteredData.map((row: AttendanceSummary) => (
                <tr key={row.employee_id} className="hover:bg-slate-50/50 transition-colors">
                  <td className="px-8 py-5 font-black text-slate-900 text-sm tracking-tight">{row.employee_name}</td>
                  <td className="px-8 py-5 text-right">
                    <div className="font-mono font-black text-primary-600 text-sm tracking-tight">{formatUnitsToHours(row.total_work_units, workUnitMinutes)} {t("reports.table.hours")}</div>
                    <span className="text-[9px] font-black text-slate-400 uppercase tracking-tighter">{row.total_work_units} {t("reports.table.units")}</span>
                  </td>
                  <td className="px-8 py-5 text-right font-mono font-black text-orange-600 text-sm">{row.total_leave_days} {t("common.days_short")}</td>
                  <td className="px-8 py-5 text-right text-slate-300 font-bold">-</td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left min-w-[1000px] border-collapse">
              <thead>
                <tr className="bg-slate-50/50 border-b border-slate-100">
                  <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.date")}</th>
                  <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.employee")}</th>
                  <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.shift")}</th>
                  <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center">{t("common.in")}</th>
                  <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center">{t("common.out")}</th>
                  <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("reports.table.hours")}</th>
                  <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center">{t("common.status")}</th>
                  {isDebugMode && <th className="px-8 py-4 text-[10px] font-black text-amber-600 uppercase tracking-widest text-center">Debug</th>}
                  <th className="px-8 py-4 text-right"></th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-50">
                {isLoading ? (
                  <tr><td colSpan={isDebugMode ? 9 : 8} className="px-8 py-12 text-center text-slate-400 font-bold italic">{t("common.loading")}</td></tr>
                ) : filteredData.length === 0 ? (
                  <tr><td colSpan={isDebugMode ? 9 : 8} className="px-8 py-20 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("reports.messages.no_data")}</td></tr>
                ) : filteredData.map((row: DailyAttendanceReport, idx: number) => {
                  const rowKey = `${row.employee_id}-${row.date}`;
                  const isReprocessing = reprocessingRow === rowKey;
                  const isExpanded = expandedDebugRows.has(rowKey);
                  return (
                  <React.Fragment key={`${rowKey}-${idx}`}>
                    <tr className={cn("transition-all group", isExpanded ? "bg-amber-50/20" : "hover:bg-slate-50/50")}>
                      <td className="px-8 py-5 font-mono text-[11px] font-black text-slate-500 uppercase">{formatDate(row.date)}</td>
                      <td className="px-8 py-5 font-black text-slate-900 text-sm tracking-tight">{row.employee_name}</td>
                      <td className="px-8 py-5">
                        {row.scheduled_start ? (
                          <div className="flex flex-col">
                            <span className="font-black text-slate-700 text-xs">{row.scheduled_start.substring(0, 5)} - {row.scheduled_end?.substring(0, 5)}</span>
                            {row.break_duration_minutes !== null && <span className="text-[9px] font-black text-slate-400 uppercase tracking-tighter">{row.break_start ? `${t("common.break")}: ${row.break_start.substring(0, 5)}-${row.break_end?.substring(0, 5)}` : `${t("common.break")}: ${row.break_duration_minutes}m`}</span>}
                          </div>
                        ) : <span className="text-[9px] font-black text-slate-300 uppercase tracking-widest">{t("reports.no_shift")}</span>}
                      </td>
                      <td className="px-8 py-5 text-center font-mono font-black text-xs text-slate-900">{formatTime(row.actual_punch_in)}</td>
                      <td className="px-8 py-5 text-center font-mono font-black text-xs text-slate-900">{formatTime(row.actual_punch_out)}</td>
                      <td className="px-8 py-5 text-right">
                        <div className="font-mono font-black text-primary-600 text-sm">{formatUnitsToHours(row.work_units, workUnitMinutes)}h</div>
                        <span className="text-[9px] font-black text-slate-400 uppercase tracking-tighter">{row.work_units}u</span>
                      </td>
                      <td className="px-8 py-5 text-center">{getStatusBadge(row.status)}</td>
                      {isDebugMode && (
                        <td className="px-8 py-5 text-center">
                          <button onClick={() => toggleExpandRow(rowKey)} className={cn("p-1.5 rounded-xl transition-all shadow-sm border", isExpanded ? "bg-amber-100 text-amber-700 border-amber-200" : "bg-white text-amber-600 border-slate-100 hover:bg-amber-50")}><Bug size={14} /></button>
                        </td>
                      )}
                      <td className="px-8 py-5 text-right">
                        <button onClick={() => handleReprocessClick(row)} disabled={isReprocessing} className="p-2 bg-white text-blue-600 hover:bg-blue-50 rounded-xl shadow-sm border border-slate-100 transition-all active:scale-90 opacity-0 group-hover:opacity-100 disabled:opacity-50" title={t("reports.reprocess.tooltip")}><RefreshCw size={16} className={cn(isReprocessing && "animate-spin")} /></button>
                      </td>
                    </tr>
                    {isDebugMode && isExpanded && (
                      <tr className="bg-amber-50/10">
                        <td colSpan={9} className="px-8 py-6">
                          <div className="bg-slate-900 rounded-[2rem] p-6 shadow-2xl border-2 border-amber-200/30 overflow-auto max-h-[500px]">
                            <div className="flex items-center gap-3 mb-4 border-b border-white/5 pb-4">
                              <Bug className="w-5 h-5 text-amber-400" />
                              <span className="text-[10px] font-black text-amber-400 uppercase tracking-[0.2em]">BSON Generation Metadata (Audit Trail)</span>
                            </div>
                            <pre className="text-[11px] font-mono text-emerald-400 leading-relaxed overflow-x-auto">
                              {JSON.stringify(row.generation_metadata || { message: "No metadata available" }, null, 2)}
                            </pre>
                          </div>
                        </td>
                      </tr>
                    )}
                  </React.Fragment>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Quick Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 px-2">
        <div className="bg-slate-900 p-8 rounded-[2.5rem] border border-slate-800 shadow-2xl shadow-slate-200 overflow-hidden relative group">
          <div className="bg-primary-500/20 w-12 h-12 rounded-xl flex items-center justify-center mb-6 text-primary-400 shadow-inner">
            <FileBarChart size={20} />
          </div>
          <p className="text-[9px] font-black text-slate-400 uppercase tracking-widest">{t("reports.quick_stats.total_hours")}</p>
          <p className="text-3xl font-black text-white mt-2 truncate">
            {formatUnitsToHours(
              reportType === "summary" 
                ? filteredData.reduce((acc: number, r: any) => acc + r.total_work_units, 0)
                : filteredData.reduce((acc: number, r: any) => acc + r.work_units, 0),
              workUnitMinutes
            )} <span className="text-lg opacity-50 font-medium">h</span>
          </p>
          <FileBarChart className="absolute -right-4 -bottom-4 w-24 h-24 text-white opacity-[0.03] -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
        </div>
      </div>

      <Modal isOpen={!!confirmDialog} onClose={() => !reprocessMutation.isPending && setConfirmDialog(null)} title={t("reports.reprocess.confirm_title")} icon={<RefreshCw size={24} />} maxWidth="md">
        <div className="space-y-8 pt-4">
          <p className="text-sm font-bold text-slate-600 leading-relaxed">{confirmDialog && t("reports.reprocess.confirm_message", { employee: confirmDialog.employeeName, date: formatDate(confirmDialog.date) })}</p>
          <div className="flex gap-4">
            <button onClick={() => setConfirmDialog(null)} className="flex-1 px-6 py-4 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-100 transition-all" disabled={reprocessMutation.isPending}>{t("common.cancel")}</button>
            <button onClick={handleConfirmReprocess} disabled={reprocessMutation.isPending} className="flex-1 bg-primary-600 hover:bg-primary-700 text-white font-black px-6 py-4 rounded-2xl flex items-center justify-center gap-3 transition-all active:scale-95 disabled:opacity-50 shadow-lg shadow-primary-100 text-xs uppercase tracking-widest">
              {reprocessMutation.isPending ? <RefreshCw className="w-4 h-4 animate-spin" /> : <Check size={18} />} {t("reports.reprocess.confirm")}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-slate-900 rounded-3xl p-6 border border-slate-800 shadow-xl shadow-slate-200">
          <h3 className="text-white font-bold mb-2 flex items-center gap-2">
            <FileBarChart className="w-5 h-5 text-blue-400" /> {t("reports.quick_stats.title")}
          </h3>
          <div className="grid grid-cols-2 gap-4 mt-4">
            <div className="bg-slate-800/50 p-4 rounded-2xl">
              <p className="text-slate-400 text-xs font-black uppercase tracking-widest">{t("common.employees")}</p>
              <p className="text-2xl font-bold text-white mt-1">{new Set(filteredData.map((r: any) => r.employee_id)).size}</p>
            </div>
            <div className="bg-slate-800/50 p-4 rounded-2xl">
              <p className="text-slate-400 text-xs font-black uppercase tracking-widest">{t("reports.quick_stats.total_hours")}</p>
              <p className="text-2xl font-bold text-blue-400 mt-1">
                {formatUnitsToHours(
                  reportType === "summary" 
                    ? filteredData.reduce((acc: number, r: any) => acc + r.total_work_units, 0)
                    : filteredData.reduce((acc: number, r: any) => acc + r.work_units, 0),
                  workUnitMinutes
                )}
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Confirmation Dialog */}
      <Modal
        isOpen={!!confirmDialog}
        onClose={() => !reprocessMutation.isPending && setConfirmDialog(null)}
        title={t("reports.reprocess.confirm_title")}
        icon={<RefreshCw className="w-6 h-6" />}
        maxWidth="md"
      >
        <div className="space-y-6">
          <p className="text-sm text-slate-600">
            {confirmDialog && t("reports.reprocess.confirm_message", {
              employee: confirmDialog.employeeName,
              date: formatDate(confirmDialog.date),
            })}
          </p>
          <div className="flex gap-3 justify-end">
            <button
              onClick={() => setConfirmDialog(null)}
              className="px-4 py-2 text-sm font-semibold text-slate-600 hover:text-slate-900 hover:bg-slate-100 rounded-lg transition-colors"
              disabled={reprocessMutation.isPending}
            >
              {t("common.cancel")}
            </button>
            <button
              onClick={handleConfirmReprocess}
              disabled={reprocessMutation.isPending}
              className="px-4 py-2 text-sm font-semibold text-white bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
            >
              {reprocessMutation.isPending && <RefreshCw className="w-4 h-4 animate-spin" />}
              {t("reports.reprocess.confirm")}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}