import React, { useState, useEffect } from "react";
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
  CalendarRange,
  Users2,
  Check,
  ArrowUpDown,
  ArrowUp,
  ArrowDown
} from "lucide-react";
import { format, startOfMonth, endOfMonth, getDaysInMonth } from "date-fns";
import {
  fetchAttendanceReport,
  getCurrentWorkspaceId,
  fetchWorkspace,
  AttendanceSummary,
  DailyAttendanceReport,
  reprocessAttendance,
  AttendanceReportSort
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
import { useToast } from "@/components/Toast";
import { cn } from "@/lib/utils";
import PaginationControls from "@/components/common/PaginationControls";

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
    const d = new Date(); d.setMonth(d.getMonth() + offset);
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

  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [sortBy, setSortBy] = useState<AttendanceReportSort | undefined>(undefined);

  const updatePageSizeForDateRange = (start: string, end: string) => {
    const startDate = new Date(start);
    const endDate = new Date(end);
    if (startDate.getDate() === 1 && format(endDate, 'yyyy-MM-dd') === format(endOfMonth(startDate), 'yyyy-MM-dd')) {
      setPageSize(getDaysInMonth(startDate));
    }
    setPage(1);
  };

  const handleFilterModeChange = (mode: "this_month" | "last_month" | "custom" | "specific_month") => {
    setFilterMode(mode);
    let range;
    if (mode === "this_month") range = getMonthRange(0);
    else if (mode === "last_month") range = getMonthRange(-1);
    
    if (range) {
      setDateRange(range);
      updatePageSizeForDateRange(range.start, range.end);
    }
  };

  const handleSpecificMonthChange = (m: number, y: number) => {
    setSpecificMonth({ month: m, year: y }); setFilterMode("specific_month");
    const start = format(new Date(y, m - 1, 1), 'yyyy-MM-dd');
    const end = format(endOfMonth(new Date(y, m - 1, 1)), 'yyyy-MM-dd');
    setDateRange({ start, end });
    updatePageSizeForDateRange(start, end);
  };

  const handleReportTypeChange = (type: "summary" | "daily") => {
    setReportType(type);
    setPage(1);
  };

  const toggleExpandRow = (rowKey: string) => {
    const next = new Set(expandedDebugRows);
    if (next.has(rowKey)) next.delete(rowKey); else next.add(rowKey);
    setExpandedDebugRows(next);
  };

  const { data: reportResponse, isLoading } = useQuery({
    queryKey: ["attendance-report", wsId, dateRange, reportType, page, pageSize, sortBy, selectedEmployeeIds],
    queryFn: () => fetchAttendanceReport(wsId!, dateRange.start, dateRange.end, reportType, page, pageSize, sortBy, selectedEmployeeIds),
    enabled: !!wsId,
  });

  const reportData = reportResponse?.data || (Array.isArray(reportResponse) ? reportResponse : []);
  const totalItems = reportResponse?.total || reportData.length;
  const totalPages = reportResponse?.total ? Math.ceil(reportResponse.total / pageSize) : 1;

  const handleSort = (field: 'name' | 'work_hours' | 'leave_days') => {
    let nextSort: AttendanceReportSort;
    if (field === 'name') {
      nextSort = sortBy === 'name_asc' ? 'name_desc' : 'name_asc';
    } else if (field === 'work_hours') {
      nextSort = sortBy === 'work_hours_desc' ? 'work_hours_asc' : 'work_hours_desc';
    } else {
      nextSort = sortBy === 'leave_days_desc' ? 'leave_days_asc' : 'leave_days_desc';
    }
    setSortBy(nextSort);
    setPage(1);
  };

  const reprocessMutation = useMutation({
    mutationFn: ({ employeeId, date }: { employeeId: string; date: string }) => reprocessAttendance(wsId!, employeeId, date, date),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
      setReprocessingRow(null); setConfirmDialog(null);
      toast.success(t("common.success"));
    },
  });

  const toast = useToast();

  const handleReprocessClick = (row: DailyAttendanceReport) => {
    setConfirmDialog({ show: true, employeeId: row.employee_id, employeeName: row.employee_name, date: row.date });
  };

  const handleConfirmReprocess = () => {
    if (!confirmDialog) return;
    setReprocessingRow(`${confirmDialog.employeeId}-${confirmDialog.date}`);
    reprocessMutation.mutate({ employeeId: confirmDialog.employeeId, date: confirmDialog.date });
  };

  const filteredData = Array.isArray(reportData) ? reportData : [];

  const getSortIcon = (field: 'name' | 'work_hours' | 'leave_days') => {
    if (field === 'name') {
      if (sortBy === 'name_asc') return <ArrowUp size={12} className="ml-1 text-primary-600" />;
      if (sortBy === 'name_desc') return <ArrowDown size={12} className="ml-1 text-primary-600" />;
    }
    if (field === 'work_hours') {
      if (sortBy === 'work_hours_asc') return <ArrowUp size={12} className="ml-1 text-primary-600" />;
      if (sortBy === 'work_hours_desc') return <ArrowDown size={12} className="ml-1 text-primary-600" />;
    }
    if (field === 'leave_days') {
      if (sortBy === 'leave_days_asc') return <ArrowUp size={12} className="ml-1 text-primary-600" />;
      if (sortBy === 'leave_days_desc') return <ArrowDown size={12} className="ml-1 text-primary-600" />;
    }
    return <ArrowUpDown size={12} className="ml-1 opacity-20 group-hover:opacity-100 transition-opacity" />;
  };



  if (!wsId) return <div className="p-12 text-center text-slate-500 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;



  const getStatusBadge = (status: string) => {

    const styles: Record<string, string> = {

      present: "bg-emerald-50 text-emerald-600 border-emerald-100",

      late: "bg-amber-50 text-amber-600 border-amber-100",

      absent: "bg-rose-50 text-rose-600 border-rose-100",

      off: "bg-slate-50 text-slate-500 border-slate-100",

    };

    const labels: Record<string, string> = {

      present: t("reports.status.present"), late: t("reports.status.late"),

      absent: t("reports.status.absent"), off: t("reports.status.off"),

    };

    return <span className={cn("px-2.5 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm", styles[status] || styles.off)}>{labels[status] || status}</span>;

  };



  return (

    <div className="space-y-10 pb-20 animate-in fade-in duration-700">

      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">

        <div className="space-y-1">

          <h1 className="text-4xl font-bold tracking-tighter text-slate-700 uppercase leading-none">{t("reports.title")}</h1>

          <p className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] opacity-60">{t("reports.subtitle")}</p>

        </div>

        <button className="flex items-center gap-3 bg-white border border-slate-200 text-slate-700 px-8 py-3 rounded-2xl hover:bg-slate-50 transition-all font-bold text-xs uppercase tracking-widest shadow-xl active:scale-95"><Download size={18} /> {t("reports.export_excel").toUpperCase()}</button>

      </div>



            <div className="flex flex-col xl:flex-row xl:items-center justify-between gap-8">



              <div className="flex bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50 shadow-inner">



                <button onClick={() => handleReportTypeChange("summary")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", reportType === 'summary' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><List size={14} /> {t("reports.tabs.summary")}</button>



                <button onClick={() => handleReportTypeChange("daily")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", reportType === 'daily' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><TableProperties size={14} /> {t("reports.tabs.daily")}</button>



              </div>



        <div className="flex flex-col md:flex-row md:items-center gap-6 bg-white p-3 rounded-[2.5rem] border border-slate-100 shadow-xl shadow-slate-900/5 pr-6">

          <div className="flex items-center bg-slate-100/50 rounded-[1.5rem] p-1.5 shadow-inner border border-slate-200/50">

            {["this_month", "last_month"].map(mode => (

              <button key={mode} onClick={() => handleFilterModeChange(mode as any)} className={cn("px-5 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", filterMode === mode ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}>{t(`common.filters.${mode}`).toUpperCase()}</button>

            ))}

            <div className="h-6 w-px bg-slate-200 mx-2 opacity-50" />

                        <DropdownMenu>

                          <DropdownMenuTrigger asChild>

                            <button className={cn("flex items-center gap-2 px-5 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", filterMode === "specific_month" ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}>

                              <CalendarRange size={14} /> {filterMode === "specific_month" ? `${specificMonth.month}/${specificMonth.year}` : t("common.filters.specific_month").toUpperCase()}

                            </button>

                          </DropdownMenuTrigger>

            

              <DropdownMenuContent className="w-80 p-4 rounded-[2.5rem] shadow-premium-2xl border-slate-100 bg-white/95 backdrop-blur-xl" align="end" sideOffset={12}>

                 <MonthYearPicker month={specificMonth.month} year={specificMonth.year} onChange={handleSpecificMonthChange} />

              </DropdownMenuContent>

            </DropdownMenu>

            <button onClick={() => handleFilterModeChange("custom")} className={cn("px-5 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", filterMode === "custom" ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}>{t("common.filters.custom").toUpperCase()}</button>

          </div>

          <div className="h-10 w-px bg-slate-100 hidden md:block opacity-50" />

          <div className="w-full md:w-80"><EmployeeSearch workspaceId={wsId!} value={selectedEmployeeIds} onChange={setSelectedEmployeeIds} multiple={true} autoFocus={true} placeholder={t("reports.search_placeholder")} /></div>

        </div>

      </div>



      {filterMode === "custom" && (

        <div className="bg-white p-8 rounded-[2.5rem] border border-slate-100 shadow-xl shadow-primary-900/5 grid grid-cols-1 md:grid-cols-2 gap-10 animate-in slide-in-from-top-4 duration-500 max-w-2xl">

                    <div className="space-y-3 px-1">

                      <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1 flex items-center gap-2"><Calendar size={12} /> {t("common.from_date")}</label>

                      <input 

                        type="date" 

                        value={dateRange.start} 

                        onChange={(e) => {

                          const newStart = e.target.value;

                          setDateRange(prev => ({ ...prev, start: newStart }));

                          updatePageSizeForDateRange(newStart, dateRange.end);

                        }} 

                        className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 outline-none focus:ring-4 focus:ring-primary-500/10 focus:bg-white transition-all font-bold text-sm shadow-inner" 

                      />

                    </div>

                    <div className="space-y-3 px-1">

                      <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1 flex items-center gap-2"><Calendar size={12} /> {t("common.to_date")}</label>

                      <input 

                        type="date" 

                        value={dateRange.end} 

                        onChange={(e) => {

                          const newEnd = e.target.value;

                          setDateRange(prev => ({ ...prev, end: newEnd }));

                          updatePageSizeForDateRange(dateRange.start, newEnd);

                        }} 

                        className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 outline-none focus:ring-4 focus:ring-primary-500/10 focus:bg-white transition-all font-bold text-sm shadow-inner" 

                      />

                    </div>

        </div>

      )}



                        <div className="bg-white border border-slate-100 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">



                          <PaginationControls 



                            currentPage={page} 



                            totalPages={totalPages} 



                            onPageChange={setPage} 



                            pageSize={pageSize} 



                            totalItems={totalItems} 



                            onPageSizeChange={setPageSize}



                          />



                          {reportType === "summary" ? (



                <table className="w-full text-left border-collapse">



                  <thead>



                    <tr className="bg-slate-50/50 border-b border-slate-100">



                      <th 



                        className="px-10 py-6 text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] cursor-pointer hover:text-primary-600 transition-colors group"



                        onClick={() => handleSort('name')}



                      >



                        <div className="flex items-center">



                          {t("common.employee")}



                          {getSortIcon('name')}



                        </div>



                      </th>



                      <th 



                        className="px-10 py-6 text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] text-right cursor-pointer hover:text-primary-600 transition-colors group"



                        onClick={() => handleSort('work_hours')}



                      >



                        <div className="flex items-center justify-end">



                          {t("reports.table.total_hours")}



                          {getSortIcon('work_hours')}



                        </div>



                      </th>



                      <th 



                        className="px-10 py-6 text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] text-right cursor-pointer hover:text-primary-600 transition-colors group"



                        onClick={() => handleSort('leave_days')}



                      >



                        <div className="flex items-center justify-end">



                          {t("reports.table.leave_days")}



                          {getSortIcon('leave_days')}



                        </div>



                      </th>



                      <th className="px-10 py-6 text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] text-right">{t("common.notes")}</th>



                    </tr>



                  </thead>



                  <tbody className="divide-y divide-slate-50">



                    {isLoading ? (



                      <tr><td colSpan={4} className="px-10 py-20 text-center text-slate-500 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse">{t("reports.messages.generating")}</td></tr>



                    ) : filteredData.length === 0 ? (



                      <tr><td colSpan={4} className="px-10 py-32 text-center text-slate-500 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("reports.messages.no_data")}</td></tr>



                    ) : filteredData.map((row: AttendanceSummary) => (



                      <tr key={row.employee_id} className="hover:bg-slate-50/50 transition-all group">



                        <td className="px-10 py-6 font-bold text-slate-700 text-sm tracking-tight uppercase leading-none">{row.employee_name}</td>



                        <td className="px-10 py-6 text-right">



                          <div className="font-mono font-bold text-primary-600 text-base tracking-tighter">{formatUnitsToHours(row.total_work_units, workUnitMinutes)}h</div>



                          <span className="text-[9px] font-bold text-slate-500 uppercase tracking-widest opacity-60">{row.total_work_units} UNITS</span>



                        </td>



                        <td className="px-10 py-6 text-right font-mono font-bold text-orange-600 text-base tracking-tighter">{row.total_leave_days} {t("common.days_short")}</td>



                        <td className="px-10 py-6 text-right text-slate-200 font-bold tracking-widest">—</td>



                      </tr>



                    ))}



                  </tbody>



                </table>



              ) : (



                <div className="overflow-x-auto">



                  <table className="w-full text-left min-w-[1200px] border-collapse">



                    <thead>



                      <tr className="bg-slate-50/50 border-b border-slate-100">



                        <th className="px-10 py-6 text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em]">{t("common.date")}</th>



                        <th className="px-8 py-6 text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em]">{t("common.employee")}</th>



                        <th className="px-8 py-6 text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em]">{t("common.shift")}</th>



                        <th className="px-8 py-6 text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] text-center">{t("common.in")}</th>



                        <th className="px-8 py-6 text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] text-center">{t("common.out")}</th>



                        <th className="px-8 py-6 text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] text-right">{t("reports.table.hours")}</th>



                        <th className="px-8 py-6 text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] text-center">{t("common.status")}</th>



                        {isDebugMode && <th className="px-8 py-6 text-[10px] font-bold text-amber-600 uppercase tracking-[0.25em] text-center">DEBUG</th>}



                        <th className="px-8 py-6"></th>



                      </tr>



                    </thead>



                    <tbody className="divide-y divide-slate-50">



                      {isLoading ? (



                        <tr><td colSpan={isDebugMode ? 9 : 8} className="px-10 py-20 text-center text-slate-500 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse">{t("common.loading")}</td></tr>



                      ) : filteredData.length === 0 ? (



                        <tr><td colSpan={isDebugMode ? 9 : 8} className="px-10 py-32 text-center text-slate-500 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("reports.messages.no_data")}</td></tr>



                      ) : filteredData.map((row: DailyAttendanceReport, idx: number) => {



                        const rowKey = `${row.employee_id}-${row.date}`;



                        const isReprocessing = reprocessingRow === rowKey;



                        const isExpanded = expandedDebugRows.has(rowKey);



                        return (



                        <React.Fragment key={`${rowKey}-${idx}`}>



                          <tr className={cn("transition-all group", isExpanded ? "bg-amber-50/20" : "hover:bg-slate-50/50")}>



                            <td className="px-10 py-6 font-mono text-[11px] font-bold text-slate-500 uppercase">{formatDate(row.date)}</td>



                            <td className="px-8 py-6 font-bold text-slate-700 text-sm tracking-tight uppercase leading-none">{row.employee_name}</td>



                            <td className="px-8 py-6">



                              {row.scheduled_start ? (



                                <div className="flex flex-col">



                                  <span className="font-bold text-slate-700 text-xs uppercase tracking-tighter shadow-sm bg-white px-2 py-0.5 rounded-lg border border-slate-100 w-fit">{row.scheduled_start.substring(0, 5)} — {row.scheduled_end?.substring(0, 5)}</span>



                                  {row.break_duration_minutes !== null && <span className="text-[8px] font-bold text-slate-500 uppercase tracking-[0.2em] mt-1.5 opacity-70 italic">{row.break_start ? `${t("common.break")}: ${row.break_start.substring(0, 5)}-${row.break_end?.substring(0, 5)}` : `${t("common.break")}: ${row.break_duration_minutes}m`}</span>}



                                </div>



                              ) : <span className="text-[9px] font-bold text-slate-300 uppercase tracking-widest italic opacity-50">{t("reports.no_shift")}</span>}



                            </td>



                            <td className="px-8 py-6 text-center font-mono font-bold text-xs text-slate-700 tracking-tighter shadow-inner bg-slate-50/50 rounded-xl m-2">{formatTime(row.actual_punch_in)}</td>



                            <td className="px-8 py-6 text-center font-mono font-bold text-xs text-slate-700 tracking-tighter shadow-inner bg-slate-50/50 rounded-xl m-2">{formatTime(row.actual_punch_out)}</td>



                            <td className="px-8 py-6 text-right">



                              <div className="font-mono font-bold text-primary-600 text-base tracking-tighter leading-none">{formatUnitsToHours(row.work_units, workUnitMinutes)}h</div>



                              <span className="text-[9px] font-bold text-slate-500 uppercase tracking-widest opacity-60">{row.work_units}U</span>



                            </td>



                            <td className="px-8 py-6 text-center">{getStatusBadge(row.status)}</td>



                            {isDebugMode && (



                              <td className="px-8 py-6 text-center">



                                <button onClick={() => toggleExpandRow(rowKey)} className={cn("p-2 rounded-xl transition-all shadow-md border active:scale-90", isExpanded ? "bg-amber-100 text-amber-700 border-amber-200" : "bg-white text-amber-600 border-slate-100 hover:bg-amber-50")}><Bug size={16} /></button>



                              </td>



                            )}



                            <td className="px-10 py-6 text-right">



                              <button onClick={() => handleReprocessClick(row)} disabled={isReprocessing} className="p-3 bg-white text-blue-600 hover:text-blue-700 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90 opacity-0 group-hover:opacity-100 disabled:opacity-50" title={t("reports.reprocess.tooltip")}><RefreshCw size={18} className={cn(isReprocessing && "animate-spin")} /></button>



                            </td>



                          </tr>



                          {isDebugMode && isExpanded && (



                            <tr className="bg-slate-900/5 animate-in slide-in-from-top-2 duration-500">



                              <td colSpan={9} className="px-10 py-10">



                                <div className="bg-slate-900 rounded-[3rem] p-10 shadow-2xl border-2 border-amber-200/20 overflow-hidden relative">



                                  <div className="flex items-center gap-4 mb-8 border-b border-white/10 pb-6 relative z-10">



                                    <div className="w-10 h-10 rounded-xl bg-amber-500/10 flex items-center justify-center text-amber-400 shadow-inner border border-amber-500/20"><Bug size={20} /></div>



                                    <div>



                                      <h4 className="text-[10px] font-bold text-amber-400 uppercase tracking-[0.3em]">Audit Trail Analysis</h4>



                                      <p className="text-[9px] text-white/40 uppercase tracking-widest mt-1 font-bold">BSON Traceability Metadata — Real-time Evaluator Logs</p>



                                    </div>



                                  </div>



                                  <pre className="text-[11px] font-mono text-emerald-400 leading-relaxed overflow-x-auto custom-scrollbar relative z-10 p-2 selection:bg-emerald-500/20 max-h-[400px]">



                                    {JSON.stringify(row.generation_metadata || { message: "No metadata available" }, null, 2)}



                                  </pre>



                                  <Bug className="absolute -right-16 -bottom-16 w-64 h-64 text-amber-500 opacity-5 -rotate-12 pointer-events-none" />



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



                              <PaginationControls 



                                currentPage={page} 



                                totalPages={totalPages} 



                                onPageChange={setPage} 



                                pageSize={pageSize} 



                                totalItems={totalItems} 



                                onPageSizeChange={setPageSize}



                              />



                            </div>



      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8 px-2">

        <div className="bg-slate-900 p-10 rounded-[3rem] border border-slate-800 shadow-2xl shadow-slate-900/20 overflow-hidden relative group hover:scale-[1.02] transition-all duration-500">

          <div className="bg-primary-500/20 w-16 h-16 rounded-[1.5rem] flex items-center justify-center mb-8 text-primary-400 shadow-inner border border-white/5 group-hover:scale-110 group-hover:rotate-3 transition-transform duration-500">

            <FileBarChart size={32} />

          </div>

          <p className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] mb-2 opacity-70">{t("reports.quick_stats.total_hours")}</p>

          <p className="text-5xl font-bold text-white tracking-tighter">

            {formatUnitsToHours(

              reportType === "summary" 

                ? filteredData.reduce((acc: number, r: any) => acc + r.total_work_units, 0)

                : filteredData.reduce((acc: number, r: any) => acc + r.work_units, 0),

              workUnitMinutes

            )} <span className="text-xl opacity-30 font-bold uppercase tracking-widest ml-1">HRS</span>

          </p>

          <FileBarChart className="absolute -right-8 -bottom-8 w-40 h-44 text-white opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />

        </div>

        <div className="bg-white p-10 rounded-[3rem] border border-slate-100 shadow-xl shadow-slate-900/5 overflow-hidden relative group hover:scale-[1.02] transition-all duration-500">

          <div className="bg-primary-50 w-16 h-16 rounded-[1.5rem] flex items-center justify-center mb-8 text-primary-600 shadow-inner border border-primary-100 group-hover:scale-110 group-hover:rotate-3 transition-transform duration-500">

            <Users2 size={32} />

          </div>

          <p className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] mb-2 opacity-70">{t("common.employees")}</p>

          <p className="text-5xl font-bold text-slate-700 tracking-tighter">

            {new Set(filteredData.map((r: any) => r.employee_id)).size} <span className="text-xl opacity-20 font-bold uppercase tracking-widest ml-1">STAFF</span>

          </p>

          <Users2 className="absolute -right-8 -bottom-8 w-40 h-44 text-slate-100 opacity-20 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />

        </div>

      </div>



      <Modal isOpen={!!confirmDialog} onClose={() => !reprocessMutation.isPending && setConfirmDialog(null)} title={t("reports.reprocess.confirm_title")} icon={<RefreshCw size={24} />} maxWidth="md">

        <div className="space-y-10 pt-4 animate-in fade-in duration-500">

          <div className="p-8 bg-slate-50 rounded-[2.5rem] border border-slate-100 shadow-inner">

            <p className="text-base font-bold text-slate-700 uppercase tracking-tight leading-relaxed text-center italic opacity-80">"{confirmDialog && t("reports.reprocess.confirm_message", { employee: confirmDialog.employeeName, date: formatDate(confirmDialog.date) })}"</p>

          </div>

          <div className="flex gap-6 border-t border-slate-50 pt-2">

            <button onClick={() => setConfirmDialog(null)} className="flex-1 px-8 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all active:scale-95" disabled={reprocessMutation.isPending}>{t("common.cancel")}</button>

            <button onClick={handleConfirmReprocess} disabled={reprocessMutation.isPending} className="flex-[1.5] bg-slate-900 hover:bg-primary-700 text-white font-bold px-8 py-4 rounded-2xl flex items-center justify-center gap-4 transition-all active:scale-95 disabled:opacity-50 shadow-2xl shadow-slate-400 text-xs uppercase tracking-widest">

              {reprocessMutation.isPending ? <RefreshCw className="w-5 h-5 animate-spin" /> : <Check size={20} />} {t("reports.reprocess.confirm").toUpperCase()}

            </button>

          </div>

        </div>

      </Modal>

    </div>

  );

}
