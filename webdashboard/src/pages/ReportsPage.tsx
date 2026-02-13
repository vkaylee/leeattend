import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Download, RefreshCw, Check } from "lucide-react";
import { format, endOfMonth, getDaysInMonth } from "date-fns";
import {
  fetchAttendanceReport,
  getCurrentWorkspaceId,
  fetchWorkspace,
  DailyAttendanceReport,
  reprocessAttendance,
  AttendanceReportSort
} from "../lib/api";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import Modal from "@/components/ui/Modal";
import { useDebug } from "@/context/DebugContext";
import { useToast } from "@/components/ui/Toast";
import PaginationControls from "@/components/common/PaginationControls";
import ReportExportDialog from "@/components/features/reports/ReportExportDialog";
import ReportFilters from "@/components/features/reports/ReportFilters";
import SummaryReportTable from "@/components/features/reports/SummaryReportTable";
import DailyReportTable from "@/components/features/reports/DailyReportTable";
import ReportStats from "@/components/features/reports/ReportStats";

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
  const [isExportDialogOpen, setIsExportDialogOpen] = useState(false);

  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(() => getDaysInMonth(new Date(getMonthRange(-1).start)));
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
    setSpecificMonth({ month: m, year: y });
    setFilterMode("specific_month");
    const start = format(new Date(y, m - 1, 1), 'yyyy-MM-dd');
    const end = format(endOfMonth(new Date(y, m - 1, 1)), 'yyyy-MM-dd');
    setDateRange({ start, end });
    updatePageSizeForDateRange(start, end);
  };

  const handleReportTypeChange = (type: "summary" | "daily") => {
    setReportType(type);
    setPage(1);
  };

  const handleDateRangeChange = (newRange: { start: string; end: string }) => {
    setDateRange(newRange);
    updatePageSizeForDateRange(newRange.start, newRange.end);
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

  if (!wsId) return <div className="p-12 text-center text-slate-500 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;



  return (

    <div className="space-y-10 pb-20 animate-in fade-in duration-700">

      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">

        <div className="space-y-1">

          <h1 className="text-4xl font-bold tracking-tighter text-slate-700 uppercase leading-none">{t("reports.title")}</h1>

          <p className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] opacity-60">{t("reports.subtitle")}</p>

        </div>

        <button
          onClick={() => setIsExportDialogOpen(true)}
          className="flex items-center gap-3 bg-white border border-slate-200 text-slate-700 px-8 py-3 rounded-2xl hover:bg-slate-50 transition-all font-bold text-xs uppercase tracking-widest shadow-xl active:scale-95"
        >
          <Download size={18} /> {t("reports.export_excel").toUpperCase()}
        </button>

      </div>

      <ReportFilters
        filterMode={filterMode}
        reportType={reportType}
        dateRange={dateRange}
        specificMonth={specificMonth}
        selectedEmployeeIds={selectedEmployeeIds}
        workspaceId={wsId}
        t={t}
        onFilterModeChange={handleFilterModeChange}
        onReportTypeChange={handleReportTypeChange}
        onDateRangeChange={handleDateRangeChange}
        onSpecificMonthChange={handleSpecificMonthChange}
        onEmployeeIdsChange={setSelectedEmployeeIds}
      />



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
                            <SummaryReportTable
                              data={filteredData}
                              isLoading={isLoading}
                              workUnitMinutes={workUnitMinutes}
                              sortBy={sortBy}
                              t={t}
                              onSort={handleSort}
                            />
                          ) : (
                            <DailyReportTable
                              data={filteredData}
                              isLoading={isLoading}
                              workUnitMinutes={workUnitMinutes}
                              isDebugMode={isDebugMode}
                              expandedDebugRows={expandedDebugRows}
                              reprocessingRow={reprocessingRow}
                              t={t}
                              formatDate={formatDate}
                              formatTime={formatTime}
                              onToggleExpandRow={toggleExpandRow}
                              onReprocessClick={handleReprocessClick}
                            />
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



      <ReportStats
        reportType={reportType}
        filteredData={filteredData}
        workUnitMinutes={workUnitMinutes}
        t={t}
      />



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

      <ReportExportDialog
        isOpen={isExportDialogOpen}
        onClose={() => setIsExportDialogOpen(false)}
        wsId={wsId || ""}
        startDate={dateRange.start}
        endDate={dateRange.end}
        reportType={reportType === "daily" ? "detail" : "summary"}
        employeeIds={selectedEmployeeIds.length > 0 ? selectedEmployeeIds : undefined}
      />

    </div>

  );

}
