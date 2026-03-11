import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Clock, 
  CheckCircle, 
  XCircle, 
  FileDown, 
  Upload, 
  User as UserIcon,
  Search,
  Check,
  X,
  AlertCircle
} from "lucide-react";
import { 
  fetchWorkspaceAttendanceAdjustments, 
  fetchEmployees, 
  approveAttendanceAdjustment, 
  getCurrentWorkspaceId 
} from "../lib/api";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useToast } from "@/components/ui/Toast";
import { cn } from "@/lib/utils";
import Button from "@/components/ui/Button";
import StatusBadge from "@/components/ui/StatusBadge";
import AttendanceImportDialog from "@/components/features/attendance/AttendanceImportDialog";

type TabType = "pending" | "approved" | "rejected";

export default function AttendanceAdjustmentsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const { formatDate, formatTime } = useDateTimeFormatter();
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<TabType>("pending");
  const toast = useToast();

  const { data: requests = [], isLoading } = useQuery({ 
    queryKey: ["attendance-adjustments", wsId, activeTab], 
    queryFn: () => fetchWorkspaceAttendanceAdjustments(wsId!, activeTab), 
    enabled: !!wsId 
  });
  const { data: employeesData } = useQuery({ 
    queryKey: ["employees", wsId], 
    queryFn: () => fetchEmployees(wsId!, 1, 3000), 
    enabled: !!wsId 
  });
  const employees = employeesData?.data || [];

  const approveMutation = useMutation({
    mutationFn: (data: { id: string; status: string }) => approveAttendanceAdjustment(wsId!, data.id, { status: data.status }),
    onSuccess: () => { 
      queryClient.invalidateQueries({ queryKey: ["attendance-adjustments"] }); 
      queryClient.invalidateQueries({ queryKey: ["time-history"] }); 
      toast.success(t("common.success")); 
    },
  });

  const downloadTemplate = () => {
    const headers = "empNo,empName,empDt"; const example = "EMP001,Nguyen Van A,11/01/2026 08:00";
    const blob = new Blob([`\uFEFF${headers}\n${example}`], { type: 'text/csv;charset=utf-8;' });
    const link = document.body.appendChild(document.createElement("a")); link.href = URL.createObjectURL(blob); link.download = "attendance_template.csv"; link.click(); document.body.removeChild(link);
  };

  const getEmployeeName = (id: string) => { const emp = employees.find(e => e.id === id); return emp ? emp.full_name : "Unknown"; };
  const getEmployeeID = (id: string) => { const emp = employees.find(e => e.id === id); return emp ? emp.employee_id_number : "N/A"; };

  if (!wsId) return <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div className="space-y-1">
          <h1 className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none">{t("attendance_confirm.title")}</h1>
          <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60">{t("attendance_confirm.subtitle")}</p>
        </div>
        <div className="flex items-center gap-4">
          <Button variant="outline" onClick={downloadTemplate} icon={<FileDown size={18} />}>{t("attendance_confirm.excel_template").toUpperCase()}</Button>
          <Button onClick={() => setIsImportDialogOpen(true)} icon={<Upload size={18} />}>{t("attendance_confirm.import_excel").toUpperCase()}</Button>
        </div>
      </div>

      <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
        <button onClick={() => setActiveTab("pending")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'pending' ? "bg-white dark:bg-slate-900 text-amber-600 dark:text-amber-300 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><Clock size={14} /> {t("common.status_pending")}</button>
        <button onClick={() => setActiveTab("approved")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'approved' ? "bg-white dark:bg-slate-900 text-emerald-600 dark:text-emerald-300 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><CheckCircle size={14} /> {t("common.status_approved")}</button>
        <button onClick={() => setActiveTab("rejected")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'rejected' ? "bg-white dark:bg-slate-900 text-rose-600 dark:text-rose-300 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><XCircle size={14} /> {t("common.status_rejected")}</button>
      </div>

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">
        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
                <th className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("common.employee")}</th>
                <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">{t("common.time")}</th>
                <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("common.notes")}</th>
                <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">{t("common.status")}</th>
                <th className="px-10 py-6 border-b border-slate-100 dark:border-slate-800"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
              {isLoading ? (
                <tr><td colSpan={5} className="px-10 py-20 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse">{t("common.loading")}</td></tr>
              ) : requests.length === 0 ? (
                <tr><td colSpan={5} className="px-10 py-32 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 flex flex-col items-center gap-4">
                  <Clock size={40} className="text-slate-200 dark:text-slate-700" />
                  {t("attendance_confirm.messages.no_data")}
                </td></tr>
              ) : requests.map((req) => (
                <tr key={req.id} className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group">
                  <td className="px-10 py-6">
                    <div className="flex items-center gap-4">
                      <div className="w-12 h-12 rounded-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 font-bold shadow-inner border border-primary-100 dark:border-primary-800/50 overflow-hidden">
                        <UserIcon size={20} />
                      </div>
                      <div>
                        <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none mb-1">{getEmployeeName(req.employee_id)}</div>
                        <div className="text-[10px] text-slate-500 dark:text-slate-400 font-mono font-bold uppercase opacity-70">ID: {getEmployeeID(req.employee_id)}</div>
                      </div>
                    </div>
                  </td>
                  <td className="px-8 py-6 text-center">
                    <div className="inline-flex flex-col items-center bg-slate-50 dark:bg-slate-800 px-4 py-2 rounded-xl border border-slate-100 dark:border-slate-800 shadow-inner">
                      <div className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tighter">{formatDate(req.requested_punch_in_time)}</div>
                      <div className="text-[10px] font-mono font-bold text-primary-600 dark:text-primary-400 mt-1">{formatTime(req.requested_punch_in_time)} — {req.requested_punch_out_time ? formatTime(req.requested_punch_out_time) : "???"}</div>
                    </div>
                  </td>
                  <td className="px-8 py-6">
                    <p className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-tight line-clamp-2 italic opacity-70 leading-relaxed">{req.reason || t("common.no_notes")}</p>
                  </td>
                  <td className="px-8 py-6 text-center">
                    <StatusBadge variant={req.status === "pending" ? "warning" : req.status === "approved" ? "success" : "danger"}>
                      {t(`common.status_${req.status}`).toUpperCase()}
                    </StatusBadge>
                  </td>
                  <td className="px-10 py-6 text-right">
                    {req.status === "pending" && (
                      <div className="flex items-center justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300">
                        <button onClick={() => approveMutation.mutate({ id: req.id, status: "approved" })} className="p-3 bg-emerald-600 text-white hover:bg-emerald-700 rounded-xl shadow-lg shadow-emerald-100 transition-all active:scale-90" title={t("common.approve")}><Check size={18} /></button>
                        <button onClick={() => approveMutation.mutate({ id: req.id, status: "rejected" })} className="p-3 bg-white dark:bg-slate-900 text-rose-600 dark:text-rose-300 hover:bg-rose-50 dark:bg-rose-950 rounded-xl shadow-md border border-rose-100 dark:border-rose-800 transition-all active:scale-90" title={t("common.reject")}><X size={18} /></button>
                      </div>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

            <AttendanceImportDialog 

              isOpen={isImportDialogOpen} 

              onClose={() => setIsImportDialogOpen(false)} 

              title={t("attendance_confirm.import_excel")}

              endpoint="/attendance-adjustments/upload"

              targetType="adjustment"

            />

          </div>

        );

      }

      