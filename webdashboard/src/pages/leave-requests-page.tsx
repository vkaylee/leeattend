import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  CalendarDays, Calendar, CheckCircle2, XCircle, Clock, MessageSquare, Plus, Info, PlayCircle, FileText, ShieldCheck, Sun, Moon, Timer,
  Upload, Download, Loader2, AlertTriangle, X
} from "lucide-react";
import React, { useState, useEffect, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";

import type { LeaveImportJob } from "@/lib/api/leaves";
import type { LeaveDurationType, Employee, LeaveType } from "@/lib/api/types";
import type { TFunction } from "i18next";

import { EmployeeSearch } from "@/components/common/employee-search";
import LeaveImportDialog from "@/components/features/leave/leave-import-dialog";
import ApprovalListLayout, { TabConfig } from "@/components/templates/approval-list-layout";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useAllEmployees } from "@/hooks/use-employees";
import {
  submitLeaveRequest,
  fetchWorkspaceLeaveRequests,
  approveLeaveRequest,
  fetchEmployeeLeaveBalances,
  fetchLeaveTypes,
  getCurrentWorkspaceId,
  fetchLatestLeaveImport,
  downloadLeaveImportErrors,
} from "@/lib/api";
import { cn } from "@/lib/utils";

type TabType = "pending" | "approved" | "rejected";
type LeaveRequestItem = Awaited<ReturnType<typeof fetchWorkspaceLeaveRequests>>[0];

// ============================================================================
// 1. Leave Import Banner Component
// ============================================================================
function getImportErrorInfo(job: LeaveImportJob) {
  if (!job.error_log) return null;
  try {
    return JSON.parse(job.error_log) as { success_count?: number; error_count?: number; error_file_path?: string };
  } catch { return null; }
}

interface LeaveImportBannerProps {
  importBannerDismissed: boolean;
  importJob: LeaveImportJob | null;
  setImportBannerDismissed: (val: boolean) => void;
  t: TFunction;
  wsId: string;
}

function LeaveImportBanner({ importJob, importBannerDismissed, setImportBannerDismissed, t, wsId }: LeaveImportBannerProps) {
  if (!importJob || importBannerDismissed) return null;

  return (
    <div className={cn(
      "rounded-2xl px-6 py-4 flex items-center justify-between gap-4 animate-in slide-in-from-top-4 duration-500 border",
      importJob.status === 'processing' || importJob.status === 'pending'
        ? "bg-blue-50 dark:bg-blue-950/30 border-blue-200 dark:border-blue-800/50"
        : importJob.status === 'completed' && !importJob.error_log
          ? "bg-emerald-50 dark:bg-emerald-950/30 border-emerald-200 dark:border-emerald-800/50"
          : importJob.status === 'completed' && importJob.error_log
            ? "bg-amber-50 dark:bg-amber-950/30 border-amber-200 dark:border-amber-800/50"
            : "bg-red-50 dark:bg-red-950/30 border-red-200 dark:border-red-800/50"
    )}>
      <div className="flex items-center gap-3 flex-1 min-w-0">
        {(importJob.status === 'processing' || importJob.status === 'pending') && (
          <>
            <Loader2 className="animate-spin text-blue-500 flex-shrink-0" size={20} />
            <div>
              <p className="text-sm font-semibold text-blue-700 dark:text-blue-300">
                {t("leave_requests.upload_processing", "Đang xử lý {{processed}}/{{total}} dòng...").replace('{{processed}}', String(importJob.processed_rows)).replace('{{total}}', String(importJob.total_rows || '?'))}
              </p>
              {importJob.total_rows > 0 && (
                <div className="mt-2 h-2 rounded-full bg-blue-100 dark:bg-blue-900 overflow-hidden w-48">
                  <div
                    className="h-full rounded-full bg-blue-500 transition-all duration-500"
                    style={{ width: `${Math.min(100, (importJob.processed_rows / importJob.total_rows) * 100).toString()}%` }}
                  />
                </div>
              )}
            </div>
          </>
        )}
        {importJob.status === 'completed' && !importJob.error_log && (
          <>
            <CheckCircle2 className="text-emerald-500 flex-shrink-0" size={20} />
            <p className="text-sm font-semibold text-emerald-700 dark:text-emerald-300">
              {t("leave_requests.upload_success", "Import thành công {{success}} đơn nghỉ").replace('{{success}}', String(importJob.processed_rows))}
            </p>
          </>
        )}
        {importJob.status === 'completed' && !!importJob.error_log && (() => {
          const info = getImportErrorInfo(importJob);
          return (
            <>
              <AlertTriangle className="text-amber-500 flex-shrink-0" size={20} />
              <p className="text-sm font-semibold text-amber-700 dark:text-amber-300">
                {t("leave_requests.upload_partial", "Thành công {{success}}/{{total}}. {{errors}} dòng lỗi.")
                  .replace('{{success}}', String(info?.success_count ?? importJob.processed_rows))
                  .replace('{{total}}', String((info?.success_count ?? 0) + (info?.error_count ?? 0)))
                  .replace('{{errors}}', String(info?.error_count ?? 0))}
              </p>
              {!!info?.error_file_path && (
                <button
                  className="ml-2 flex items-center gap-1.5 text-xs font-bold text-amber-600 hover:text-amber-800 dark:text-amber-400 dark:hover:text-amber-200 underline underline-offset-2"
                  onClick={() => { void downloadLeaveImportErrors(wsId, importJob.id).catch(() => { /* empty */ }); }}
                >
                  <Download size={14} /> {t("leave_requests.download_errors", "Tải file lỗi")}
                </button>
              )}
            </>
          );
        })()}
        {importJob.status === 'failed' && (
          <>
            <XCircle className="text-red-500 flex-shrink-0" size={20} />
            <p className="text-sm font-semibold text-red-700 dark:text-red-300">
              {t("leave_requests.upload_failed", "Import thất bại")}
              {!!importJob.error_log && <span className="font-normal ml-2 text-xs opacity-75">— {importJob.error_log.substring(0, 100)}</span>}
            </p>
          </>
        )}
      </div>
      {importJob.status !== 'processing' && importJob.status !== 'pending' && (
        <button className="text-primary-400 hover:text-primary-600 transition-colors flex-shrink-0" onClick={() => { setImportBannerDismissed(true); }}>
          <X size={18} />
        </button>
      )}
    </div>
  );
}

// ============================================================================
// 2. Create Leave Request Modal
// ============================================================================
interface CreateLeaveRequestModalProps {
  isOpen: boolean;
  leaveTypes: LeaveType[];
  onClose: () => void;
  t: TFunction;
  wsId: string;
}

const getLeaveTypeIcon = (code: string) => {
  const c = code.toLowerCase();
  if (c.includes("vacation") || c.includes("annual")) return <Calendar size={14} />;
  if (c.includes("sick")) return <FileText size={14} />;
  if (c.includes("personal")) return <CalendarDays size={14} />;
  return <PlayCircle size={14} />;
};

function CreateLeaveRequestModal({ isOpen, onClose, wsId, leaveTypes, t }: CreateLeaveRequestModalProps) {
  const toast = useToast();
  const queryClient = useQueryClient();

  const [selectedEmpId, setSelectedEmpId] = useState<string>("");
  const [leaveTypeId, setLeaveTypeId] = useState<string>("");
  const [durationType, setDurationType] = useState<LeaveDurationType>("full_day");
  const [startTime, setStartTime] = useState<string>("");
  const [endTime, setEndTime] = useState<string>("");
  const [startDate, setStartDate] = useState<string>("");
  const [endDate, setEndDate] = useState<string>("");

  const { data: balances = [] } = useQuery({ 
    queryKey: ["leave-balances", wsId, selectedEmpId], 
    queryFn: () => fetchEmployeeLeaveBalances(wsId, selectedEmpId), 
    enabled: !!wsId && !!selectedEmpId && isOpen 
  });

  useEffect(() => {
    if (leaveTypes.length > 0 && !leaveTypeId) {
      setLeaveTypeId(leaveTypes[0].id);
    }
  }, [leaveTypes, leaveTypeId]);

  const submitMutation = useMutation({
    mutationFn: (data: { employee_id: string; leave_type_id: string; start_date: string; end_date: string; reason?: string; duration_type?: LeaveDurationType; start_time?: string; end_time?: string }) => submitLeaveRequest(wsId, data),
    onSuccess: () => { 
      void queryClient.invalidateQueries({ queryKey: ["leave-requests"] }); 
      onClose(); 
      setSelectedEmpId("");
      setDurationType("full_day"); 
      setStartTime(""); 
      setEndTime(""); 
      setStartDate(""); 
      setEndDate(""); 
      toast.success(t("common.success")); 
    },
  });

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault(); 
    const formData = new FormData(e.currentTarget);
    if (!leaveTypeId) { toast.error(t("leave_requests.invalid_leave_type")); return; }
 
    const payload: { employee_id: string; leave_type_id: string; start_date: string; end_date: string; reason?: string; duration_type?: LeaveDurationType; start_time?: string; end_time?: string } = {
      employee_id: selectedEmpId,
      leave_type_id: leaveTypeId,
      start_date: startDate || (formData.get("start_date") as string),
      end_date: endDate || (formData.get("end_date") as string),
      reason: formData.get("reason") as string,
      duration_type: durationType,
    };
    if (durationType === "custom_hours" && startTime && endTime) {
      payload.start_time = startTime;
      payload.end_time = endTime;
    }
    submitMutation.mutate(payload);
  };

  return (
    <Modal icon={<CalendarDays size={24} />} isOpen={isOpen} maxWidth="md" onClose={onClose} subtitle={t("leave_requests.modal.submit_subtitle")} title={t("leave_requests.modal.submit_title")}>
      <form className="space-y-8 pt-4" onSubmit={handleSubmit}>
        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest ml-1" htmlFor="leave-emp">{t("common.employee")}</label>
          <EmployeeSearch className="h-14 rounded-2xl bg-slate-50 dark:bg-slate-800 border-primary-100 dark:border-primary-800/50 shadow-sm" id="leave-emp" onChange={(value) => { setSelectedEmpId(Array.isArray(value) ? value[0] || "" : value); }} value={selectedEmpId} workspaceId={wsId} />
        </div>
        {!!selectedEmpId && balances.length > 0 && (
          <div className="bg-primary-50 dark:bg-primary-950/50 p-8 rounded-[2.5rem] border border-primary-100 dark:border-primary-800/50 grid grid-cols-2 gap-8 shadow-inner">
            {balances.map(b => (
              <div className="space-y-2" key={b.leave_type_id || b.leave_type_code}>
                <p className="text-[9px] font-bold uppercase tracking-[0.25em] text-primary-600 dark:text-primary-400 opacity-70 leading-none">{(b.leave_type_name || b.leave_type_code || "Unknown").toUpperCase()}</p>
                <p className="text-base font-bold text-primary-700 dark:text-primary-300 tracking-tighter uppercase">{t("leave_requests.remaining")}: <span className="text-primary-600 dark:text-primary-400 text-2xl font-mono ml-1">{b.remaining}</span><span className="text-[10px] opacity-40 ml-1">/ {b.entitlement + b.carried_over}</span></p>
              </div>
            ))}
          </div>
        )}
        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest ml-1" htmlFor="leave-type">{t("common.type")}</label>
          <Combobox 
            className="bg-slate-50 dark:bg-slate-800 border-primary-100 dark:border-primary-800/50 h-14" 
            id="leave-type" 
            onChange={setLeaveTypeId} 
            options={leaveTypes.map(tOption => ({ value: tOption.id, label: tOption.name.toUpperCase(), icon: getLeaveTypeIcon(tOption.code) }))} 
            value={leaveTypeId} 
          />
        </div>
        <div className="grid grid-cols-2 gap-8">
          <FormField id="start-date" inputClassName="text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/50" label={t("common.from_date")} name="start_date" onChange={e => { setStartDate(e.target.value); if (durationType !== "full_day") setEndDate(e.target.value); }} required type="date" value={startDate} />
          <FormField disabled={durationType !== "full_day"} id="end-date" inputClassName={cn("text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/50", durationType !== "full_day" && "opacity-50 cursor-not-allowed")} label={t("common.to_date")} name="end_date" onChange={e => { setEndDate(e.target.value); }} required type="date" value={endDate} />
        </div>

        {/* Duration Type Segmented Buttons */}
        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest ml-1">{t("leave_requests.duration.label")}</label>
          <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1 rounded-2xl border border-primary-100 dark:border-primary-800/50 shadow-inner">
            {([
              { value: "full_day" as LeaveDurationType, label: t("leave_requests.duration.full_day"), icon: <Calendar size={12} /> },
              { value: "first_half" as LeaveDurationType, label: t("leave_requests.duration.first_half"), icon: <Sun size={12} /> },
              { value: "second_half" as LeaveDurationType, label: t("leave_requests.duration.second_half"), icon: <Moon size={12} /> },
              { value: "custom_hours" as LeaveDurationType, label: t("leave_requests.duration.custom_hours"), icon: <Timer size={12} /> },
            ]).map(opt => (
              <button
                className={cn(
                  "flex-1 flex items-center justify-center gap-1.5 px-3 py-2.5 rounded-xl text-[9px] font-bold uppercase tracking-wider transition-all",
                  durationType === opt.value
                    ? "bg-white dark:bg-slate-900 text-primary-700 dark:text-primary-300 shadow-sm border border-primary-100 dark:border-primary-800/50"
                    : "text-primary-400 hover:text-primary-600 dark:text-primary-500 dark:hover:text-primary-300"
                )}
                key={opt.value}
                onClick={() => {
                  setDurationType(opt.value);
                  if (opt.value !== "full_day" && startDate) {
                    setEndDate(startDate);
                  }
                }}
                type="button"
              >
                {opt.icon} {opt.label}
              </button>
            ))}
          </div>
          {durationType !== "full_day" && durationType !== "custom_hours" && (
            <p className="text-[10px] text-primary-400 italic ml-1 flex items-center gap-1"><Info size={10} /> {t("leave_requests.duration.half_day_note")}</p>
          )}
        </div>

        {/* Time Pickers for Custom Hours */}
        {durationType === "custom_hours" && (
          <div className="grid grid-cols-2 gap-8 animate-in fade-in slide-in-from-top-2 duration-300">
            <FormField id="start-time" inputClassName="text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/50" label={t("leave_requests.duration.time_from")} onChange={e => { setStartTime(e.target.value); }} required type="time" value={startTime} />
            <FormField id="end-time" inputClassName="text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/50" label={t("leave_requests.duration.time_to")} onChange={e => { setEndTime(e.target.value); }} required type="time" value={endTime} />
          </div>
        )}

        <FormField id="leave-reason" inputClassName="text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/50" label={t("common.reason")} name="reason" placeholder={t("leave_requests.modal.reason_placeholder")} rows={3} type="textarea" />
        <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
          <Button onClick={onClose} size="lg" type="button" variant="outline">{t("common.cancel").toUpperCase()}</Button>
          <Button size="lg" type="submit" variant="primary">{t("leave_requests.modal.submit_btn").toUpperCase()}</Button>
        </div>
      </form>
    </Modal>
  );
}

// ============================================================================
// 3. Leave Request Table Row Component
// ============================================================================
interface LeaveRequestRowProps {
  emp?: Employee;
  formatDate: (d: string) => string;
  leaveTypes: LeaveType[];
  onApprove: (id: string, status: string) => void;
  request: LeaveRequestItem;
  t: TFunction;
}

function LeaveRequestRow({ request, emp, leaveTypes, formatDate, t, onApprove }: LeaveRequestRowProps) {
  return (
    <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group">
      <td className="px-10 py-6">
        <div className="flex items-center gap-5">
          <div className="w-14 h-14 bg-primary-50 dark:bg-primary-950 rounded-2xl flex items-center justify-center text-primary-600 dark:text-primary-400 font-bold text-xl shadow-inner border border-primary-100 dark:border-primary-800/50 uppercase group-hover:scale-110 transition-transform">
            {emp?.full_name[0] ?? "?"}
          </div>
          <div>
            <div className="font-bold text-primary-700 dark:text-primary-300 text-sm tracking-tight uppercase leading-none mb-1.5">{emp?.full_name ?? "N/A"}</div>
            <div className="text-[10px] text-primary-400 font-mono font-bold uppercase tracking-widest opacity-70">{emp?.employee_id_number}</div>
          </div>
        </div>
      </td>
      <td className="px-8 py-6"><span className="px-3 py-1 bg-slate-100 dark:bg-slate-700 text-primary-700 dark:text-primary-300 rounded-lg text-[9px] font-bold uppercase tracking-widest border border-primary-200 dark:border-primary-800/50 shadow-sm">{(leaveTypes.find(tOption => tOption.id === request.leave_type_id)?.name ?? "Unknown").toUpperCase()}</span></td>
      <td className="px-8 py-6">
        <div className="text-xs font-bold text-primary-700 dark:text-primary-300 uppercase tracking-tight mb-1.5">
          {formatDate(request.start_date)} — {formatDate(request.end_date)}
          {!!request.duration_type && request.duration_type !== "full_day" && (
            <span className={cn("ml-2 px-2 py-0.5 rounded-md text-[8px] font-bold uppercase tracking-wider inline-flex items-center gap-1 border",
              request.duration_type === "first_half" ? "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-200 dark:border-amber-800" :
              request.duration_type === "second_half" ? "bg-indigo-50 dark:bg-indigo-950 text-indigo-600 dark:text-indigo-300 border-indigo-200 dark:border-indigo-800" :
              "bg-cyan-50 dark:bg-cyan-950 text-cyan-600 dark:text-cyan-300 border-cyan-200 dark:border-cyan-800"
            )}>
              {request.duration_type === "first_half" && <><Sun size={8} /> {t("leave_requests.duration.first_half")}</>}
              {request.duration_type === "second_half" && <><Moon size={8} /> {t("leave_requests.duration.second_half")}</>}
              {request.duration_type === "custom_hours" && <><Timer size={8} /> {request.start_time?.slice(0,5) ?? ""}–{request.end_time?.slice(0,5) ?? ""}</>}
            </span>
          )}
        </div>
        <div className="bg-slate-50 dark:bg-slate-800/50 p-3 rounded-xl border border-primary-100 dark:border-primary-800/50 shadow-inner group-hover:bg-white dark:hover:bg-slate-900 transition-colors">
          <p className="text-[10px] text-primary-400 font-bold leading-relaxed line-clamp-1 italic opacity-80 flex gap-2"><MessageSquare className="text-slate-300 dark:text-slate-600 mt-0.5 shrink-0" size={12} /> "{request.reason ?? t("common.no_description")}"</p>
        </div>
      </td>
      <td className="px-8 py-6 text-center">
        <span className={cn("px-4 py-1.5 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm inline-block", 
          request.status === "pending" ? "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800" : 
          request.status === "approved" ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" :
          request.status === "pre_approved" ? "bg-teal-50 dark:bg-teal-950 text-teal-600 dark:text-teal-300 border-teal-100 dark:border-teal-800" :
          "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800"
        )}>
          {request.status === "pending" ? t("common.status_pending").toUpperCase() : request.status === "approved" ? t("common.status_approved").toUpperCase() : request.status === "pre_approved" ? <span className="flex items-center gap-1"><ShieldCheck size={10} />{t("common.status_pre_approved").toUpperCase()}</span> : t("common.status_rejected").toUpperCase()}
        </span>
      </td>
      <td className="px-10 py-6 text-right">
        {request.status === "pending" && (
          <div className="flex justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300 translate-x-2 group-hover:translate-x-0">
            <button className="p-3 bg-white dark:bg-slate-900 text-emerald-400 hover:text-emerald-600 dark:text-emerald-300 rounded-xl shadow-md border border-primary-100 dark:border-primary-800/50 transition-all active:scale-90" onClick={() => { onApprove(request.id, "approved"); }} title={t("common.approve")}><CheckCircle2 size={20} /></button>
            <button className="p-3 bg-white dark:bg-slate-900 text-rose-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-primary-100 dark:border-primary-800/50 transition-all active:scale-90" onClick={() => { onApprove(request.id, "rejected"); }} title={t("common.reject")}><XCircle size={20} /></button>
          </div>
        )}
      </td>
    </tr>
  );
}

// ============================================================================
// 4. Main Page Component
// ============================================================================
export default function LeaveRequestsPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const { formatDate } = useDateTimeFormatter();
  
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isImportModalOpen, setIsImportModalOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<TabType>("pending");

  const { employees } = useAllEmployees(wsId);
  const { data: requests = [], isLoading: requestsLoading } = useQuery({ queryKey: ["leave-requests", wsId, activeTab], queryFn: () => fetchWorkspaceLeaveRequests(wsId!, activeTab), enabled: !!wsId });
  const { data: leaveTypes = [] } = useQuery({ queryKey: ["leave-types", wsId], queryFn: () => fetchLeaveTypes(wsId!), enabled: !!wsId });

  const approveMutation = useMutation({
    mutationFn: (data: { id: string; status: string }) => approveLeaveRequest(wsId!, data.id, { status: data.status }),
    onSuccess: () => { void queryClient.invalidateQueries({ queryKey: ["leave-requests"] }); toast.success(t("common.success")); },
  });

  const [importJob, setImportJob] = useState<LeaveImportJob | null>(null);
  const [importBannerDismissed, setImportBannerDismissed] = useState(false);
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    if (!wsId) return;
    void fetchLatestLeaveImport(wsId).then((job) => {
      if (job) {
        const created = new Date(job.created_at);
        const now = new Date();
        if (now.getTime() - created.getTime() < 24 * 60 * 60 * 1000) {
          setImportJob(job);
          setImportBannerDismissed(false);
        }
      }
    }).catch(() => { /* empty */ });
  }, [wsId]);

  const startPolling = useCallback((wsIdValue: string) => {
    if (pollingRef.current) clearInterval(pollingRef.current);
    pollingRef.current = setInterval(async () => {
      try {
        const job = await fetchLatestLeaveImport(wsIdValue);
        if (job) {
          setImportJob(job);
          if (job.status !== 'processing' && job.status !== 'pending') {
            if (pollingRef.current) clearInterval(pollingRef.current);
            pollingRef.current = null;
            void queryClient.invalidateQueries({ queryKey: ["leave-requests"] });
          }
        }
      } catch {
        // Network error — keep polling
      }
    }, 3000);
  }, [queryClient]);

  useEffect(() => {
    if (importJob?.status === 'processing' || importJob?.status === 'pending') {
      if (wsId) startPolling(wsId);
    }
    return () => {
      if (pollingRef.current) clearInterval(pollingRef.current);
    };
  }, [importJob?.status, wsId, startPolling]);

  const handleImportComplete = useCallback(() => {
    if (wsId) {
      void fetchLatestLeaveImport(wsId).then((job) => {
        if (job) {
          setImportJob(job);
          setImportBannerDismissed(false);
        }
      }).catch(() => { /* empty */ });
      void queryClient.invalidateQueries({ queryKey: ["leave-requests"] });
    }
  }, [wsId, queryClient]);

  if (!wsId) return <div className="p-12 text-center text-primary-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;

  const tabs: TabConfig[] = [
    { id: "pending", label: t("common.status_pending"), icon: <Clock size={14} />, colorScheme: "amber" },
    { id: "approved", label: t("common.status_approved"), icon: <CheckCircle2 size={14} />, colorScheme: "emerald" },
    { id: "rejected", label: t("common.status_rejected"), icon: <XCircle size={14} />, colorScheme: "rose" },
  ];

  const tableHeaders = [
    <th className="px-10 py-6 text-[10px] font-bold text-primary-400 uppercase tracking-[0.25em]" key="emp">{t("common.employee")}</th>,
    <th className="px-8 py-6 text-[10px] font-bold text-primary-400 uppercase tracking-[0.25em]" key="type">{t("common.type")}</th>,
    <th className="px-8 py-6 text-[10px] font-bold text-primary-400 uppercase tracking-[0.25em]" key="time">{t("common.time")}</th>,
    <th className="px-8 py-6 text-[10px] font-bold text-primary-400 uppercase tracking-[0.25em] text-center" key="status">{t("common.status")}</th>,
    <th className="px-10 py-6 border-b border-primary-100 dark:border-primary-800/50" key="actions"></th>,
  ];

  return (
    <ApprovalListLayout
      activeTab={activeTab}
      banner={<LeaveImportBanner importBannerDismissed={importBannerDismissed} importJob={importJob} setImportBannerDismissed={setImportBannerDismissed} t={t} wsId={wsId} />}
      emptyMessage={t("leave_requests.messages.no_requests")}
      headerActions={
        <>
          <Button
            className="bg-emerald-600 hover:bg-emerald-700 shadow-emerald-100 dark:shadow-black/30 text-[10px] px-5 py-3"
            disabled={importJob?.status === 'processing'}
            onClick={() => { setIsImportModalOpen(true); }}
          >
            <Upload size={16} />
            {t("leave_requests.upload_btn", "Upload Excel")}
          </Button>
          <Button onClick={() => { setIsModalOpen(true); }}>
            <Plus size={18} /> {t("leave_requests.create_btn").toUpperCase()}
          </Button>
        </>
      }
      isEmpty={requests.length === 0}
      isLoading={requestsLoading}
      modals={
        <>
          <CreateLeaveRequestModal isOpen={isModalOpen} leaveTypes={leaveTypes} onClose={() => { setIsModalOpen(false); }} t={t} wsId={wsId} />
          <LeaveImportDialog
            isOpen={isImportModalOpen}
            onClose={() => { setIsImportModalOpen(false); }}
            onImportComplete={handleImportComplete}
          />
        </>
      }
      onTabChange={(id) => { setActiveTab(id as TabType); }}
      subtitle={t("leave_requests.subtitle")}
      tableHeaders={tableHeaders}
      tabs={tabs}
      title={t("leave_requests.title")}
    >
      {requests.map((request) => (
        <LeaveRequestRow
          emp={employees.find(e => e.id === request.employee_id)}
          formatDate={formatDate}
          key={request.id}
          leaveTypes={leaveTypes}
          onApprove={(id, status) => { approveMutation.mutate({ id, status }); }}
          request={request}
          t={t}
        />
      ))}
    </ApprovalListLayout>
  );
}
