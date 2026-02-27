import React, { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Download, RefreshCw, Check, CheckCircle } from "lucide-react";
import { format, endOfMonth, getDaysInMonth } from "date-fns";
import {
  fetchAttendanceReport,
  getCurrentWorkspaceId,
  fetchWorkspace,
  DailyAttendanceReport,
  reprocessAttendance,
  AttendanceReportSort,
  fetchWorkspaceAttendanceAdjustments,
  adminConfirmAttendance,
  AttendanceAdjustmentRequest,
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
  const [confirmingRow, setConfirmingRow] = useState<string | null>(null);
  const [confirmModal, setConfirmModal] = useState<{
    row: DailyAttendanceReport;
    existingRequest?: AttendanceAdjustmentRequest;
    punchIn: string;
    punchOut: string;
    reason: string;
  } | null>(null);
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

  const { data: pendingAdjustments } = useQuery({
    queryKey: ["attendance-adjustments", wsId, "pending"],
    queryFn: () => fetchWorkspaceAttendanceAdjustments(wsId!, "pending"),
    enabled: !!wsId && reportType === "daily",
  });

  const adjustmentMap = useMemo(() => {
    const map: Record<string, AttendanceAdjustmentRequest> = {};
    pendingAdjustments?.forEach((req) => {
      const key = `${req.employee_id}-${req.requested_date}`;
      map[key] = req;
    });
    return map;
  }, [pendingAdjustments]);

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

  const toast = useToast();

  const reprocessMutation = useMutation({
    mutationFn: ({ employeeId, date }: { employeeId: string; date: string }) => reprocessAttendance(wsId!, employeeId, date, date),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
      setReprocessingRow(null); setConfirmDialog(null);
      toast.success(t("common.success"));
    },
  });

  const confirmMutation = useMutation({
    mutationFn: ({ row, existingRequestId, reason }: { row: DailyAttendanceReport; existingRequestId?: string; reason?: string }) =>
      adminConfirmAttendance(wsId!, row, existingRequestId, reason),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
      queryClient.invalidateQueries({ queryKey: ["attendance-adjustments"] });
      setConfirmingRow(null);
      setConfirmModal(null);
      toast.success(t("reports.confirm.success"));
    },
    onError: () => {
      setConfirmingRow(null);
      toast.error(t("reports.confirm.error"));
    },
  });

  const handleReprocessClick = (row: DailyAttendanceReport) => {
    setConfirmDialog({ show: true, employeeId: row.employee_id, employeeName: row.employee_name, date: row.date });
  };

  const handleConfirmReprocess = () => {
    if (!confirmDialog) return;
    setReprocessingRow(`${confirmDialog.employeeId}-${confirmDialog.date}`);
    reprocessMutation.mutate({ employeeId: confirmDialog.employeeId, date: confirmDialog.date });
  };

  const handleConfirmClick = (row: DailyAttendanceReport, existing?: AttendanceAdjustmentRequest) => {
    // Helper: extract local HH:mm from a UTC ISO string (e.g. "2026-01-15T01:00:00Z" → "08:00" in UTC+7)
    const utcToLocalHHmm = (iso: string | null | undefined): string => {
      if (!iso) return "";
      const d = new Date(iso);
      if (isNaN(d.getTime())) {
        // Might be plain "HH:MM:SS" (scheduled_start) — not UTC, use directly
        return iso.substring(0, 5);
      }
      return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
    };

    // Pre-fill times from: existing request > actual punch > scheduled > empty
    const punchIn = utcToLocalHHmm(existing?.requested_punch_in_time)
      || utcToLocalHHmm(row.actual_punch_in)
      || row.scheduled_start?.substring(0, 5)
      || "";
    const punchOut = utcToLocalHHmm(existing?.requested_punch_out_time)
      || utcToLocalHHmm(row.actual_punch_out)
      || row.scheduled_end?.substring(0, 5)
      || "";
    const reason = existing?.reason || "";
    setConfirmModal({ row, existingRequest: existing, punchIn, punchOut, reason });
  };

  const handleConfirmSubmit = () => {
    if (!confirmModal) return;
    const { row, existingRequest } = confirmModal;
    const rowKey = `${row.employee_id}-${row.date}`;
    setConfirmingRow(rowKey);

    if (existingRequest) {
      // Approve existing request directly
      confirmMutation.mutate({ row, existingRequestId: existingRequest.id });
    } else {
      // Convert local date and HH:mm to a full UTC ISO string
      const createIsoFromLocal = (dateStr: string, hhmm: string): string | null => {
        if (!hhmm) return null;
        const [y, m, d] = dateStr.split('-').map(Number);
        const [hours, mins] = hhmm.split(':').map(Number);
        // new Date(Y, M, D, h, m, s) creates a LOCAL date object
        return new Date(y, m - 1, d, hours, mins, 0).toISOString();
      };

      const modifiedRow = {
        ...row,
        actual_punch_in: createIsoFromLocal(row.date, confirmModal.punchIn),
        actual_punch_out: createIsoFromLocal(row.date, confirmModal.punchOut),
      };
      confirmMutation.mutate({ row: modifiedRow as DailyAttendanceReport, reason: confirmModal.reason });
    }
  };

  const filteredData = Array.isArray(reportData) ? reportData : [];

  if (!wsId) return <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;



  return (

    <div className="space-y-10 pb-20 animate-in fade-in duration-700">

      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">

        <div className="space-y-1">

          <h1 className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none">{t("reports.title")}</h1>

          <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60">{t("reports.subtitle")}</p>

        </div>

        <button
          onClick={() => setIsExportDialogOpen(true)}
          className="flex items-center gap-3 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 px-8 py-3 rounded-2xl hover:bg-slate-50 dark:hover:bg-slate-800 transition-all font-bold text-xs uppercase tracking-widest shadow-xl active:scale-95"
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



      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">



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
            confirmingRow={confirmingRow}
            adjustmentMap={adjustmentMap}
            t={t}
            formatDate={formatDate}
            formatTime={formatTime}
            onToggleExpandRow={toggleExpandRow}
            onReprocessClick={handleReprocessClick}
            onConfirmClick={handleConfirmClick}
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

          <div className="p-8 bg-slate-50 dark:bg-slate-800 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-inner">

            <p className="text-base font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight leading-relaxed text-center italic opacity-80">"{confirmDialog && t("reports.reprocess.confirm_message", { employee: confirmDialog.employeeName, date: formatDate(confirmDialog.date) })}"</p>

          </div>

          <div className="flex gap-6 border-t border-slate-50 dark:border-slate-800 pt-2">

            <button onClick={() => setConfirmDialog(null)} className="flex-1 px-8 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95" disabled={reprocessMutation.isPending}>{t("common.cancel")}</button>

            <button onClick={handleConfirmReprocess} disabled={reprocessMutation.isPending} className="flex-[1.5] bg-slate-900 hover:bg-primary-700 text-white font-bold px-8 py-4 rounded-2xl flex items-center justify-center gap-4 transition-all active:scale-95 disabled:opacity-50 shadow-2xl shadow-slate-400 dark:shadow-black/40 text-xs uppercase tracking-widest">

              {reprocessMutation.isPending ? <RefreshCw className="w-5 h-5 animate-spin" /> : <Check size={20} />} {t("reports.reprocess.confirm").toUpperCase()}

            </button>

          </div>

        </div>

      </Modal>

      {/* Confirm Attendance Modal */}
      <Modal
        isOpen={!!confirmModal}
        onClose={() => !confirmMutation.isPending && setConfirmModal(null)}
        title={t("reports.confirm.modal_title")}
        subtitle={t("reports.confirm.modal_subtitle")}
        icon={<CheckCircle size={24} />}
        maxWidth="md"
      >
        {confirmModal && (
          <div className="space-y-6 pt-2 animate-in fade-in duration-500">
            {/* Employee & Date info */}
            <div className="grid grid-cols-2 gap-4 p-5 bg-slate-50 dark:bg-slate-800 rounded-2xl border border-slate-100 dark:border-slate-800">
              <div>
                <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("reports.confirm.employee")}</label>
                <p className="text-sm font-bold text-slate-700 dark:text-slate-200 mt-1">{confirmModal.row.employee_name}</p>
              </div>
              <div>
                <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("reports.confirm.date")}</label>
                <p className="text-sm font-bold text-slate-700 dark:text-slate-200 mt-1">{formatDate(confirmModal.row.date)}</p>
              </div>
            </div>

            {/* Existing request banner */}
            {confirmModal.existingRequest && (
              <div className="p-4 bg-orange-50 dark:bg-orange-950 rounded-2xl border border-orange-200 dark:border-orange-800">
                <p className="text-xs font-bold text-orange-600 dark:text-orange-300 uppercase tracking-wider">
                  {t("reports.confirm.existing_request")}
                </p>
                {confirmModal.existingRequest.reason && (
                  <p className="text-sm text-orange-700 dark:text-orange-200 mt-2 italic">
                    &ldquo;{confirmModal.existingRequest.reason}&rdquo;
                  </p>
                )}
              </div>
            )}

            {/* Punch In / Out fields — editable only when creating new */}
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2 block">{t("reports.confirm.punch_in")}</label>
                <input
                  type="time"
                  value={confirmModal.punchIn}
                  onChange={(e) => setConfirmModal({ ...confirmModal, punchIn: e.target.value })}
                  disabled={!!confirmModal.existingRequest}
                  className="w-full px-4 py-3 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-sm font-mono font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed"
                />
              </div>
              <div>
                <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2 block">{t("reports.confirm.punch_out")}</label>
                <input
                  type="time"
                  value={confirmModal.punchOut}
                  onChange={(e) => setConfirmModal({ ...confirmModal, punchOut: e.target.value })}
                  disabled={!!confirmModal.existingRequest}
                  className="w-full px-4 py-3 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-sm font-mono font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed"
                />
              </div>
            </div>

            {/* Reason field — only for new requests */}
            {!confirmModal.existingRequest && (
              <div>
                <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2 block">{t("reports.confirm.reason")}</label>
                <input
                  type="text"
                  value={confirmModal.reason}
                  onChange={(e) => setConfirmModal({ ...confirmModal, reason: e.target.value })}
                  placeholder={t("reports.confirm.reason_placeholder")}
                  className="w-full px-4 py-3 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-sm text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                />
              </div>
            )}

            {/* Buttons */}
            <div className="flex gap-4 border-t border-slate-100 dark:border-slate-800 pt-4">
              <button
                onClick={() => setConfirmModal(null)}
                className="flex-1 px-6 py-3.5 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95"
                disabled={confirmMutation.isPending}
              >
                {t("common.cancel")}
              </button>
              <button
                onClick={handleConfirmSubmit}
                disabled={confirmMutation.isPending || (!confirmModal.existingRequest && !confirmModal.punchIn)}
                className="flex-[1.5] bg-emerald-600 hover:bg-emerald-700 text-white font-bold px-6 py-3.5 rounded-2xl flex items-center justify-center gap-3 transition-all active:scale-95 disabled:opacity-50 shadow-xl text-xs uppercase tracking-widest"
              >
                {confirmMutation.isPending
                  ? <RefreshCw className="w-4 h-4 animate-spin" />
                  : <CheckCircle size={18} />
                }
                {confirmModal.existingRequest
                  ? t("reports.confirm.approve")
                  : t("reports.confirm.submit")
                }
              </button>
            </div>
          </div>
        )}
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
