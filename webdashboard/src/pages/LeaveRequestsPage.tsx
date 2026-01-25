import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  CalendarDays, Calendar, CheckCircle2, XCircle, Clock, MessageSquare, Plus, Info, PlayCircle, FileText
} from "lucide-react";
import {
  fetchEmployees,
  submitLeaveRequest,
  fetchWorkspaceLeaveRequests,
  approveLeaveRequest,
  fetchEmployeeLeaveBalances,
  fetchLeaveTypes,
  getCurrentWorkspaceId
} from "@/lib/api";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Combobox } from "@/components/ui/Combobox";
import Modal from "@/components/ui/Modal";
import { useToast } from "@/components/Toast";
import { EmployeeSearch } from "@/components/common/EmployeeSearch";
import { cn } from "@/lib/utils";

type TabType = "pending" | "approved" | "rejected";

export default function LeaveRequestsPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const { formatDate } = useDateTimeFormatter();
  
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<TabType>("pending");
  const [selectedEmpId, setSelectedEmpId] = useState<string>("");
  const [leaveTypeId, setLeaveTypeId] = useState<string>("");

  const { data: employeesData } = useQuery({ queryKey: ["employees", wsId], queryFn: () => fetchEmployees(wsId!, 1, 3000), enabled: !!wsId });
  const employees = employeesData?.data || [];
  const { data: balances = [] } = useQuery({ queryKey: ["leave-balances", wsId, selectedEmpId], queryFn: () => fetchEmployeeLeaveBalances(wsId!, selectedEmpId), enabled: !!wsId && !!selectedEmpId });
  const { data: requests = [], isLoading: requestsLoading } = useQuery({ queryKey: ["leave-requests", wsId, activeTab], queryFn: () => fetchWorkspaceLeaveRequests(wsId!, activeTab), enabled: !!wsId });
  const { data: leaveTypes = [] } = useQuery({ queryKey: ["leave-types", wsId], queryFn: () => fetchLeaveTypes(wsId!), enabled: !!wsId });

  React.useEffect(() => {
    if (leaveTypes.length > 0 && !leaveTypeId) {
      setLeaveTypeId(leaveTypes[0].id);
    }
  }, [leaveTypes, leaveTypeId]);

  const submitMutation = useMutation({
    mutationFn: (data: { employee_id: string; leave_type_id: string; start_date: string; end_date: string; reason?: string }) => submitLeaveRequest(wsId!, data),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["leave-requests"] }); setIsModalOpen(false); toast.success(t("common.success")); },
  });

  const approveMutation = useMutation({
    mutationFn: (data: { id: string; status: string }) => approveLeaveRequest(wsId!, data.id, { status: data.status }),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["leave-requests"] }); toast.success(t("common.success")); },
  });

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault(); const formData = new FormData(e.currentTarget);
    if (!leaveTypeId) { toast.error(t("leave_requests.invalid_leave_type")); return; }
    submitMutation.mutate({ employee_id: selectedEmpId, leave_type_id: leaveTypeId, start_date: formData.get("start_date") as string, end_date: formData.get("end_date") as string, reason: formData.get("reason") as string });
  };

  const getLeaveTypeIcon = (code: string) => {
    const c = code.toLowerCase();
    if (c.includes("vacation") || c.includes("annual")) return <Calendar size={14} />;
    if (c.includes("sick")) return <FileText size={14} />;
    if (c.includes("personal")) return <CalendarDays size={14} />;
    return <PlayCircle size={14} />;
  };

  if (!wsId) return <div className="p-12 text-center text-primary-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row justify-between items-center gap-8 px-2">
        <div className="space-y-1">
          <h1 className="text-5xl font-bold tracking-tighter text-primary-700 uppercase leading-none">{t("leave_requests.title")}</h1>
          <p className="text-[10px] font-bold text-primary-400 uppercase tracking-[0.25em] opacity-60">{t("leave_requests.subtitle")}</p>
        </div>
        <button onClick={() => { setIsModalOpen(true); setSelectedEmpId(""); }} className="bg-primary-600 hover:bg-primary-700 text-white font-bold px-8 py-3 rounded-2xl flex items-center gap-3 shadow-xl shadow-primary-100 transition-all active:scale-95 text-xs uppercase tracking-widest">
          <Plus size={18} /> {t("leave_requests.create_btn").toUpperCase()}
        </button>
      </div>

      <div className="flex bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-primary-200/50/50 shadow-inner">
        <button onClick={() => setActiveTab("pending")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === "pending" ? "bg-white text-amber-600 shadow-sm border border-primary-100/50" : "text-primary-400 hover:text-primary-700")}><Clock size={14} /> {t("common.status_pending")}</button>
        <button onClick={() => setActiveTab("approved")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === "approved" ? "bg-white text-emerald-600 shadow-sm border border-primary-100/50" : "text-primary-400 hover:text-primary-700")}><CheckCircle2 size={14} /> {t("common.status_approved")}</button>
        <button onClick={() => setActiveTab("rejected")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === "rejected" ? "bg-white text-rose-600 shadow-sm border border-primary-100/50" : "text-primary-400 hover:text-primary-700")}><XCircle size={14} /> {t("common.status_rejected")}</button>
      </div>

      <div className="bg-white border border-primary-100/50 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">
        <table className="w-full text-left border-collapse">
          <thead>
            <tr className="bg-slate-50/50 border-b border-primary-100/50">
              <th className="px-10 py-6 text-[10px] font-bold text-primary-400 uppercase tracking-[0.25em]">{t("common.employee")}</th>
              <th className="px-8 py-6 text-[10px] font-bold text-primary-400 uppercase tracking-[0.25em]">{t("common.type")}</th>
              <th className="px-8 py-6 text-[10px] font-bold text-primary-400 uppercase tracking-[0.25em]">{t("common.time")}</th>
              <th className="px-8 py-6 text-[10px] font-bold text-primary-400 uppercase tracking-[0.25em] text-center">{t("common.status")}</th>
              <th className="px-10 py-6 border-b border-primary-100/50"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50">
            {requestsLoading ? (
              <tr><td colSpan={5} className="px-10 py-20 text-center text-primary-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse">{t("common.loading")}</td></tr>
            ) : requests.length === 0 ? (
              <tr><td colSpan={5} className="px-10 py-32 text-center text-primary-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("leave_requests.messages.no_requests")}</td></tr>
            ) : requests.map((req) => {
              const emp = employees.find(e => e.id === req.employee_id);
              return (
                <tr key={req.id} className="hover:bg-slate-50/50 transition-all group">
                  <td className="px-10 py-6">
                    <div className="flex items-center gap-5">
                      <div className="w-14 h-14 bg-primary-50 rounded-2xl flex items-center justify-center text-primary-600 font-bold text-xl shadow-inner border border-primary-100/50 uppercase group-hover:scale-110 transition-transform">{emp?.full_name[0]}</div>
                      <div>
                        <div className="font-bold text-primary-700 text-sm tracking-tight uppercase leading-none mb-1.5">{emp ? emp.full_name : "N/A"}</div>
                        <div className="text-[10px] text-primary-400 font-mono font-bold uppercase tracking-widest opacity-70">{emp?.employee_id_number}</div>
                      </div>
                    </div>
                  </td>
                  <td className="px-8 py-6"><span className="px-3 py-1 bg-slate-100 text-primary-700 rounded-lg text-[9px] font-bold uppercase tracking-widest border border-primary-200/50 shadow-sm">{(leaveTypes.find(t => t.id === req.leave_type_id)?.name || "Unknown").toUpperCase()}</span></td>
                  <td className="px-8 py-6">
                    <div className="text-xs font-bold text-primary-700 uppercase tracking-tight mb-1.5">{formatDate(req.start_date)} — {formatDate(req.end_date)}</div>
                    <div className="bg-slate-50/50 p-3 rounded-xl border border-primary-100/50 shadow-inner group-hover:bg-white transition-colors">
                      <p className="text-[10px] text-primary-400 font-bold leading-relaxed line-clamp-1 italic opacity-80 flex gap-2"><MessageSquare size={12} className="text-slate-300 mt-0.5 shrink-0" /> "{req.reason || t("common.no_description")}"</p>
                    </div>
                  </td>
                  <td className="px-8 py-6 text-center">
                    <span className={cn("px-4 py-1.5 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm inline-block", 
                      req.status === "pending" ? "bg-amber-50 text-amber-600 border-amber-100" : 
                      req.status === "approved" ? "bg-emerald-50 text-emerald-600 border-emerald-100" : 
                      "bg-rose-50 text-rose-600 border-rose-100"
                    )}>
                      {req.status === "pending" ? t("common.status_pending").toUpperCase() : req.status === "approved" ? t("common.status_approved").toUpperCase() : t("common.status_rejected").toUpperCase()}
                    </span>
                  </td>
                  <td className="px-10 py-6 text-right">
                    {req.status === "pending" && (
                      <div className="flex justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300 translate-x-2 group-hover:translate-x-0">
                        <button onClick={() => approveMutation.mutate({ id: req.id, status: "approved" })} className="p-3 bg-white text-emerald-400 hover:text-emerald-600 rounded-xl shadow-md border border-primary-100/50 transition-all active:scale-90" title={t("common.approve")}><CheckCircle2 size={20} /></button>
                        <button onClick={() => approveMutation.mutate({ id: req.id, status: "rejected" })} className="p-3 bg-white text-rose-400 hover:text-rose-600 rounded-xl shadow-md border border-primary-100/50 transition-all active:scale-90" title={t("common.reject")}><XCircle size={20} /></button>
                      </div>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      <Modal isOpen={isModalOpen} onClose={() => setIsModalOpen(false)} title={t("leave_requests.modal.submit_title")} subtitle={t("leave_requests.modal.submit_subtitle")} icon={<CalendarDays size={24} />} maxWidth="md">
        <form onSubmit={handleSubmit} className="space-y-8 pt-4">
          <div className="space-y-3 px-1">
            <label htmlFor="leave-emp" className="text-[10px] font-bold text-primary-600 uppercase tracking-widest ml-1">{t("common.employee")}</label>
            <EmployeeSearch id="leave-emp" workspaceId={wsId!} value={selectedEmpId} onChange={setSelectedEmpId} className="h-14 rounded-2xl bg-slate-50 border-primary-100/50 shadow-sm" />
          </div>
          {selectedEmpId && balances.length > 0 && (
            <div className="bg-primary-50/50 p-8 rounded-[2.5rem] border border-primary-100/50 grid grid-cols-2 gap-8 shadow-inner">
              {balances.map(b => (
                <div key={b.leave_type_id || b.leave_type_code} className="space-y-2">
                  <p className="text-[9px] font-bold uppercase tracking-[0.25em] text-primary-600 opacity-70 leading-none">{(b.leave_type_name || b.leave_type_code || "Unknown").toUpperCase()}</p>
                  <p className="text-base font-bold text-primary-700 tracking-tighter uppercase">{t("leave_requests.remaining")}: <span className="text-primary-600 text-2xl font-mono ml-1">{b.remaining}</span><span className="text-[10px] opacity-40 ml-1">/ {b.entitlement + b.carried_over}</span></p>
                </div>
              ))}
            </div>
          )}
          <div className="space-y-3 px-1">
            <label htmlFor="leave-type" className="text-[10px] font-bold text-primary-600 uppercase tracking-widest ml-1">{t("common.type")}</label>
            <Combobox 
              id="leave-type" 
              options={leaveTypes.map(t => ({ value: t.id, label: t.name.toUpperCase(), icon: getLeaveTypeIcon(t.code) }))} 
              value={leaveTypeId} 
              onChange={setLeaveTypeId} 
              className="bg-slate-50 border-primary-100/50 h-14" 
            />
          </div>
          <div className="grid grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label htmlFor="start-date" className="text-[10px] font-bold text-primary-600 uppercase tracking-widest ml-1">{t("common.from_date")}</label>
              <input id="start-date" type="date" name="start_date" required className="w-full px-5 py-4 text-primary-700 bg-slate-50 border border-primary-100/50 rounded-2xl outline-none transition-all font-medium text-sm tracking-tight shadow-sm" />
            </div>
            <div className="space-y-3 px-1">
              <label htmlFor="end-date" className="text-[10px] font-bold text-primary-600 uppercase tracking-widest ml-1">{t("common.to_date")}</label>
              <input id="end-date" type="date" name="end_date" required className="w-full px-5 py-4 text-primary-700 bg-slate-50 border border-primary-100/50 rounded-2xl outline-none transition-all font-medium text-sm tracking-tight shadow-sm" />
            </div>
          </div>
          <div className="space-y-3 px-1">
            <label htmlFor="leave-reason" className="text-[10px] font-bold text-primary-600 uppercase tracking-widest ml-1">{t("common.reason")}</label>
            <textarea id="leave-reason" name="reason" rows={3} className="w-full px-6 py-4 text-primary-700 bg-slate-50 border border-primary-100/50 rounded-2xl outline-none font-medium text-sm resize-none shadow-sm" placeholder={t("leave_requests.modal.reason_placeholder")}></textarea>
          </div>
          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50">
            <button type="button" onClick={() => setIsModalOpen(false)} className="px-8 py-4 bg-white border border-primary-200/50 rounded-2xl font-bold text-xs uppercase tracking-widest text-primary-400 hover:bg-slate-50">{t("common.cancel").toUpperCase()}</button>
            <button type="submit" className="px-8 py-4 bg-primary-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 shadow-xl shadow-indigo-100">{t("leave_requests.modal.submit_btn").toUpperCase()}</button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
