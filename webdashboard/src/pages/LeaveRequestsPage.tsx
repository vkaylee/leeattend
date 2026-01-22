import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  CalendarDays, Calendar, CheckCircle2, XCircle, Send, Plus, X, Settings2, PlayCircle,
  UserPlus, FileText, ShieldCheck, UserCircle, Edit2, Trash2, Clock, MessageSquare,
  FileDown, Upload, Check, Info
} from "lucide-react";
import {
  fetchEmployees,
  submitLeaveRequest,
  fetchWorkspaceLeaveRequests,
  approveLeaveRequest,
  fetchEmployeeLeaveBalances,
  fetchLeavePolicies,
  createLeavePolicy,
  updateLeavePolicy,
  deleteLeavePolicy,
  getCurrentWorkspaceId,
  LeavePolicy
} from "../lib/api";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Combobox } from "@/components/ui/Combobox";
import Modal from "@/components/ui/Modal";
import { useToast } from "@/components/Toast";
import { AssignPolicyModal } from "@/components/leave/AssignPolicyModal";
import { EmployeeSearch } from "@/components/common/EmployeeSearch";
import { cn } from "@/lib/utils";

type TabType = "pending" | "approved" | "rejected" | "policies";

export default function LeaveRequestsPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const { formatDate } = useDateTimeFormatter();
  
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isPolicyModalOpen, setIsPolicyModalOpen] = useState(false);
  const [editingPolicy, setEditingPolicy] = useState<LeavePolicy | null>(null);
  const [isAssignModalOpen, setIsAssignModalOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<TabType>("pending");
  const [accrualStrategy, setAccrualStrategy] = useState<string>("monthly");
  const [selectedEmpId, setSelectedEmpId] = useState<string>("");
  const [leaveType, setLeaveType] = useState<string>("vacation");
  const [policyType, setPolicyType] = useState<string>("vacation");

  const { data: employees = [] } = useQuery({ queryKey: ["employees", wsId], queryFn: () => fetchEmployees(wsId!), enabled: !!wsId });
  const { data: balances = [] } = useQuery({ queryKey: ["leave-balances", wsId, selectedEmpId], queryFn: () => fetchEmployeeLeaveBalances(wsId!, selectedEmpId), enabled: !!wsId && !!selectedEmpId });
  const { data: requests = [], isLoading: requestsLoading } = useQuery({ queryKey: ["leave-requests", wsId, activeTab], queryFn: () => fetchWorkspaceLeaveRequests(wsId!, activeTab), enabled: !!wsId && activeTab !== "policies" });
  const { data: policies = [], isLoading: policiesLoading } = useQuery({ queryKey: ["leave-policies", wsId], queryFn: () => fetchLeavePolicies(wsId!), enabled: !!wsId });

  const calculateAnnualEntitlement = (policy: LeavePolicy) => {
    const config = policy.rule_config; const amount = config.accrual_amount;
    switch (config.frequency) { case 'monthly': return amount * 12; case 'quarterly': return amount * 4; case 'yearly': return amount; default: return amount; }
  };

  const getAccrualStrategy = (policy: LeavePolicy) => policy.rule_config.frequency;

  const submitMutation = useMutation({
    mutationFn: (data: { employee_id: string; leave_type: string; start_date: string; end_date: string; reason?: string }) => submitLeaveRequest(wsId!, data),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["leave-requests"] }); setIsModalOpen(false); toast.success(t("common.success")); },
  });

  const approveMutation = useMutation({
    mutationFn: (data: { id: string; status: string }) => approveLeaveRequest(wsId!, data.id, { status: data.status }),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["leave-requests"] }); toast.success(t("common.success")); },
  });

  const policyMutation = useMutation({
    mutationFn: (data: Partial<LeavePolicy>) => editingPolicy ? updateLeavePolicy(wsId!, editingPolicy.id, data) : createLeavePolicy(wsId!, data),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["leave-policies"] }); setIsPolicyModalOpen(false); setEditingPolicy(null); toast.success(t("common.success")); },
  });

  const deletePolicyMutation = useMutation({
    mutationFn: (policyId: string) => deleteLeavePolicy(wsId!, policyId),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["leave-policies"] }); toast.success(t("common.success")); },
  });

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault(); const formData = new FormData(e.currentTarget);
    submitMutation.mutate({ employee_id: selectedEmpId, leave_type: leaveType, start_date: formData.get("start_date") as string, end_date: formData.get("end_date") as string, reason: formData.get("reason") as string });
  };

  const handleCreatePolicy = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault(); const formData = new FormData(e.currentTarget);
    const rule_config = { 
      frequency: accrualStrategy as any, 
      accrual_amount: Number(formData.get("annual_entitlement")) / (accrualStrategy === "monthly" ? 12 : accrualStrategy === "quarterly" ? 4 : 1), 
      accrual_timing: "start_of_period", 
      carry_over_limit: Number(formData.get("carry_over_limit")),
      min_work_days_threshold: formData.get("min_work_days_threshold") ? Number(formData.get("min_work_days_threshold")) : undefined
    };
    policyMutation.mutate({ name: formData.get("name") as string, leave_type: policyType, rule_config: rule_config as any });
  };

  if (!wsId) return <div className="p-12 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row justify-between items-center gap-8 px-2">
        <div className="space-y-1">
          <h1 className="text-4xl font-black tracking-tighter text-slate-900 uppercase leading-none">{t("leave_requests.title")}</h1>
          <p className="text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] opacity-60">{t("leave_requests.subtitle")}</p>
        </div>
        <button onClick={() => { setIsModalOpen(true); setSelectedEmpId(""); }} className="bg-primary-600 hover:bg-primary-700 text-white font-black px-8 py-3 rounded-2xl flex items-center gap-3 shadow-xl shadow-primary-100 transition-all active:scale-95 text-xs uppercase tracking-widest">
          <Plus size={18} /> {t("leave_requests.create_btn").toUpperCase()}
        </button>
      </div>

      <div className="flex bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50 shadow-inner">
        <button onClick={() => setActiveTab("pending")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'pending' ? "bg-white text-amber-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><Clock size={14} /> {t("common.status_pending")}</button>
        <button onClick={() => setActiveTab("approved")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'approved' ? "bg-white text-emerald-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><CheckCircle2 size={14} /> {t("common.status_approved")}</button>
        <button onClick={() => setActiveTab("rejected")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'rejected' ? "bg-white text-rose-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><XCircle size={14} /> {t("common.status_rejected")}</button>
        <button onClick={() => setActiveTab("policies")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'policies' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><ShieldCheck size={14} /> {t("leave_requests.tabs.policies")}</button>
      </div>

      <div className="animate-in slide-in-from-bottom-8 duration-700">
        {activeTab === "policies" ? (
          <div className="space-y-8">
            <div className="bg-primary-50/50 border border-primary-100 rounded-3xl p-6 flex items-start gap-4 mx-2">
              <Info className="w-5 h-5 text-primary-600 mt-1 shrink-0" />
              <p className="text-[11px] font-bold text-primary-900/70 leading-relaxed uppercase tracking-wider">
                {t("leave_requests.policy.policy_note")}
              </p>
            </div>
            <div className="flex justify-end gap-4 px-2">
              <button onClick={() => setIsAssignModalOpen(true)} className="flex items-center gap-3 bg-white border border-slate-200 text-slate-700 px-6 py-2.5 rounded-2xl hover:bg-slate-50 transition-all font-black text-[10px] uppercase tracking-widest shadow-md"><UserPlus size={16} /> {t("leave_requests.policy.assign_btn").toUpperCase()}</button>
              <button onClick={() => { setEditingPolicy(null); setPolicyType('vacation'); setAccrualStrategy('monthly'); setIsPolicyModalOpen(true); }} className="flex items-center gap-3 bg-slate-900 text-white px-8 py-2.5 rounded-2xl hover:bg-black transition-all font-black text-[10px] uppercase tracking-widest shadow-xl shadow-slate-200 active:scale-95"><Plus size={16} /> {t("leave_requests.policy.add_btn").toUpperCase()}</button>
            </div>
            <div className="bg-white border border-slate-100 rounded-[3rem] overflow-hidden shadow-sm">
              <table className="w-full text-left border-collapse">
                <thead>
                  <tr className="bg-slate-50/50 border-b border-slate-100">
                    <th className="px-10 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em]">{t("leave_requests.policy.table.name")}</th>
                    <th className="px-8 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em]">{t("leave_requests.policy.table.type")}</th>
                    <th className="px-8 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] text-right">{t("leave_requests.policy.table.entitlement")}</th>
                    <th className="px-8 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] text-right">{t("leave_requests.policy.table.strategy")}</th>
                    <th className="px-10 py-6 border-b border-slate-100"></th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-50">
                  {policiesLoading ? (
                    <tr><td colSpan={5} className="px-10 py-20 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse">{t("common.loading")}</td></tr>
                  ) : policies.length === 0 ? (
                    <tr><td colSpan={5} className="px-10 py-32 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("leave_requests.policy.messages.no_policies")}</td></tr>
                  ) : policies.map((policy) => (
                    <tr key={policy.id} className="hover:bg-slate-50/50 transition-all group">
                      <td className="px-10 py-6 font-black text-slate-900 text-sm tracking-tight uppercase leading-none">{policy.name}</td>
                      <td className="px-8 py-6"><span className="px-3 py-1 bg-slate-50 text-slate-500 rounded-lg text-[9px] font-black uppercase tracking-widest border border-slate-100 shadow-sm">{policy.leave_type?.toUpperCase() || 'VACATION'}</span></td>
                      <td className="px-8 py-6 text-right text-primary-600 font-black text-base font-mono tracking-tighter">{calculateAnnualEntitlement(policy)} {t("common.days_short")}</td>
                      <td className="px-8 py-6 text-right text-slate-400 font-black text-[10px] uppercase tracking-[0.2em] opacity-70">
                        {t(`leave_requests.policy.strategies.${getAccrualStrategy(policy)}`)}
                        {policy.rule_config.min_work_days_threshold && (
                          <div className="text-[8px] text-primary-500 mt-1 lowercase font-bold italic">
                            ({policy.rule_config.min_work_days_threshold} {t("leave_requests.policy.table.min_work_days").toLowerCase()})
                          </div>
                        )}
                      </td>
                      <td className="px-10 py-6 text-right">
                        <div className="flex justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300 translate-x-2 group-hover:translate-y-0 group-hover:opacity-100">
                          <button onClick={() => { setEditingPolicy(policy); setPolicyType(policy.leave_type || 'vacation'); setAccrualStrategy(getAccrualStrategy(policy)); setIsPolicyModalOpen(true); }} className="p-2.5 bg-white text-slate-400 hover:text-primary-600 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90" title={t("common.edit")}><Edit2 size={18} /></button>
                          <button onClick={() => { if (confirm(t("leave_requests.policy.messages.confirm_delete"))) deletePolicyMutation.mutate(policy.id); }} className="p-2.5 bg-white text-slate-400 hover:text-rose-600 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90" title={t("common.delete")}><Trash2 size={18} /></button>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        ) : (
          <div className="bg-white border border-slate-100 rounded-[3rem] overflow-hidden shadow-sm">
            <table className="w-full text-left border-collapse">
              <thead>
                <tr className="bg-slate-50/50 border-b border-slate-100">
                  <th className="px-10 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em]">{t("common.employee")}</th>
                  <th className="px-8 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em]">{t("common.type")}</th>
                  <th className="px-8 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em]">{t("common.time")}</th>
                  <th className="px-8 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] text-center">{t("common.status")}</th>
                  <th className="px-10 py-6 border-b border-slate-100"></th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-50">
                {requestsLoading ? (
                  <tr><td colSpan={5} className="px-10 py-20 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse">{t("common.loading")}</td></tr>
                ) : requests.length === 0 ? (
                  <tr><td colSpan={5} className="px-10 py-32 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("leave_requests.messages.no_requests")}</td></tr>
                ) : requests.map((req) => {
                  const emp = employees.find(e => e.id === req.employee_id);
                  return (
                    <tr key={req.id} className="hover:bg-slate-50/50 transition-all group">
                      <td className="px-10 py-6">
                        <div className="flex items-center gap-5">
                          <div className="w-14 h-14 bg-primary-50 rounded-2xl flex items-center justify-center text-primary-600 font-black text-xl shadow-inner border border-primary-100/50 uppercase group-hover:scale-110 transition-transform">{emp?.full_name[0]}</div>
                          <div>
                            <div className="font-black text-slate-900 text-sm tracking-tight uppercase leading-none mb-1.5">{emp ? emp.full_name : 'N/A'}</div>
                            <div className="text-[10px] text-slate-400 font-mono font-bold uppercase tracking-widest opacity-70">{emp?.employee_id_number}</div>
                          </div>
                        </div>
                      </td>
                      <td className="px-8 py-6"><span className="px-3 py-1 bg-slate-100 text-slate-600 rounded-lg text-[9px] font-black uppercase tracking-widest border border-slate-200 shadow-sm">{req.leave_type.toUpperCase()}</span></td>
                      <td className="px-8 py-6">
                        <div className="text-xs font-black text-slate-700 uppercase tracking-tight mb-1.5">{formatDate(req.start_date)} — {formatDate(req.end_date)}</div>
                        <div className="bg-slate-50/50 p-3 rounded-xl border border-slate-100 shadow-inner group-hover:bg-white transition-colors">
                          <p className="text-[10px] text-slate-500 font-bold leading-relaxed line-clamp-1 italic opacity-80 flex gap-2"><MessageSquare size={12} className="text-slate-300 mt-0.5 shrink-0" /> "{req.reason || t("common.no_description")}"</p>
                        </div>
                      </td>
                      <td className="px-8 py-6 text-center">
                        <span className={cn("px-4 py-1.5 rounded-full text-[9px] font-black uppercase tracking-widest border shadow-sm inline-block", 
                          req.status === 'pending' ? "bg-amber-50 text-amber-600 border-amber-100" : 
                          req.status === 'approved' ? "bg-emerald-50 text-emerald-600 border-emerald-100" : 
                          "bg-rose-50 text-rose-600 border-rose-100"
                        )}>
                          {req.status === 'pending' ? t("common.status_pending").toUpperCase() : req.status === 'approved' ? t("common.status_approved").toUpperCase() : t("common.status_rejected").toUpperCase()}
                        </span>
                      </td>
                      <td className="px-10 py-6 text-right">
                        {req.status === 'pending' && (
                          <div className="flex justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300 translate-x-2 group-hover:translate-x-0">
                            <button onClick={() => approveMutation.mutate({ id: req.id, status: 'approved' })} className="p-3 bg-white text-emerald-400 hover:text-emerald-600 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90" title={t("common.approve")}><CheckCircle2 size={20} /></button>
                            <button onClick={() => approveMutation.mutate({ id: req.id, status: 'rejected' })} className="p-3 bg-white text-rose-400 hover:text-rose-600 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90" title={t("common.reject")}><XCircle size={20} /></button>
                          </div>
                        )}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>

      <Modal isOpen={isModalOpen} onClose={() => setIsModalOpen(false)} title={t("leave_requests.modal.submit_title")} subtitle={t("leave_requests.modal.submit_subtitle")} icon={<CalendarDays size={24} />} maxWidth="md">
        <form onSubmit={handleSubmit} className="space-y-8 pt-4 animate-in fade-in duration-500">
          <div className="space-y-3 px-1">
            <label htmlFor="leave-emp" className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.employee")}</label>
            <EmployeeSearch id="leave-emp" workspaceId={wsId!} value={selectedEmpId} onChange={setSelectedEmpId} className="h-14 rounded-2xl bg-slate-50 border-slate-100 shadow-sm" />
          </div>
          {selectedEmpId && balances.length > 0 && (
            <div className="bg-primary-50/50 p-8 rounded-[2.5rem] border border-primary-100/50 grid grid-cols-2 gap-8 shadow-inner animate-in zoom-in-95">
              {balances.map(b => (
                <div key={b.leave_type} className="space-y-2">
                  <p className="text-[9px] font-black uppercase tracking-[0.25em] text-primary-600 opacity-70 leading-none">{b.leave_type === 'vacation' ? t("leave_requests.types.vacation").toUpperCase() : b.leave_type === 'sick' ? t("leave_requests.types.sick").toUpperCase() : b.leave_type.toUpperCase()}</p>
                  <p className="text-base font-black text-slate-700 tracking-tighter uppercase">{t("leave_requests.remaining")}: <span className="text-primary-600 text-2xl font-mono ml-1">{b.remaining}</span><span className="text-[10px] opacity-40 ml-1">/ {b.entitlement + b.carried_over}</span></p>
                </div>
              ))}
            </div>
          )}
          <div className="space-y-3 px-1">
            <label htmlFor="leave-type" className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.type")}</label>
            <Combobox id="leave-type" options={[{ value: "vacation", label: t("leave_requests.types.vacation_full").toUpperCase(), icon: <Calendar size={14} /> }, { value: "sick", label: t("leave_requests.types.sick_full").toUpperCase(), icon: <FileText size={14} /> }, { value: "personal", label: t("leave_requests.types.personal").toUpperCase(), icon: <CalendarDays size={14} /> }]} value={leaveType} onChange={setLeaveType} className="bg-slate-50 border-slate-100 h-14" />
          </div>
          <div className="grid grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label htmlFor="start-date" className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.from_date")}</label>
              <input id="start-date" type="date" name="start_date" required className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight shadow-sm" />
            </div>
            <div className="space-y-3 px-1">
              <label htmlFor="end-date" className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.to_date")}</label>
              <input id="end-date" type="date" name="end_date" required className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight shadow-sm" />
            </div>
          </div>
          <div className="space-y-3 px-1">
            <label htmlFor="leave-reason" className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.reason")}</label>
            <textarea id="leave-reason" name="reason" rows={3} className="w-full px-6 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none font-medium text-sm resize-none shadow-sm" placeholder={t("leave_requests.modal.reason_placeholder")}></textarea>
          </div>
          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50">
            <button type="button" onClick={() => setIsModalOpen(false)} className="px-8 py-4 bg-white border border-slate-200 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all active:scale-95">{t("common.cancel").toUpperCase()}</button>
            <button type="submit" className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all shadow-xl shadow-slate-200 active:scale-95">{t("leave_requests.modal.submit_btn").toUpperCase()}</button>
          </div>
        </form>
      </Modal>

      <Modal isOpen={isPolicyModalOpen} onClose={() => { setIsPolicyModalOpen(false); setEditingPolicy(null); }} title={editingPolicy ? t("common.leave_requests.policy.edit_title") : t("common.leave_requests.policy.title")} subtitle={t("leave_requests.policy.subtitle")} icon={<ShieldCheck size={24} />} maxWidth="md">
        <form onSubmit={handleCreatePolicy} className="space-y-8 pt-4 animate-in fade-in duration-500">
          <div className="space-y-3 px-1"><label htmlFor="policy-name" className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("leave_requests.policy.table.name")}</label><input id="policy-name" name="name" required defaultValue={editingPolicy?.name} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight shadow-sm" placeholder={t("leave_requests.policy.name_placeholder")} /></div>
          <div className="space-y-3 px-1"><label htmlFor="policy-type" className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("leave_requests.policy.table.type")}</label><Combobox id="policy-type" options={[{ value: "vacation", label: t("leave_requests.types.vacation").toUpperCase(), icon: <Calendar size={14} /> }, { value: "sick", label: t("leave_requests.types.sick").toUpperCase(), icon: <FileText size={14} /> }]} value={policyType} onChange={setPolicyType} className="bg-slate-50 border-slate-100 h-14" /></div>
          <div className="grid grid-cols-2 gap-8">
            <div className="space-y-3 px-1"><label htmlFor="annual-entitlement" className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("leave_requests.policy.table.entitlement")}</label><input id="annual-entitlement" type="number" name="annual_entitlement" required defaultValue={editingPolicy ? calculateAnnualEntitlement(editingPolicy) : 12} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm text-center font-mono shadow-sm" /></div>
            <div className="space-y-3 px-1"><label htmlFor="carry-over" className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("leave_requests.policy.carry_over_limit")}</label><input id="carry-over" type="number" name="carry_over_limit" required defaultValue={editingPolicy?.rule_config.carry_over_limit ?? 5} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm text-center font-mono shadow-sm" /></div>
          </div>
          <div className="space-y-3 px-1"><label htmlFor="accrual-strategy" className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("leave_requests.policy.table.strategy")}</label><Combobox id="accrual-strategy" options={[{ value: "monthly", label: t("leave_requests.policy.strategies.monthly").toUpperCase(), description: t("leave_requests.policy.strategies.monthly_desc") }, { value: "quarterly", label: t("leave_requests.policy.strategies.quarterly").toUpperCase(), description: t("leave_requests.policy.strategies.quarterly_desc") }, { value: "yearly", label: t("leave_requests.policy.strategies.yearly").toUpperCase(), description: t("leave_requests.policy.strategies.yearly_desc") }]} value={accrualStrategy} onChange={setAccrualStrategy} className="bg-slate-50 border-slate-100 h-14" /></div>
          
          <div className="space-y-3 px-1">
            <label htmlFor="min-work-days" className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-2">
              {t("leave_requests.policy.table.min_work_days")}
              <div className="group relative">
                <Info size={12} className="text-slate-300 cursor-help" />
                <div className="absolute bottom-full left-0 mb-2 w-64 p-3 bg-slate-900 text-[10px] text-white rounded-xl shadow-2xl opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50 font-medium normal-case tracking-normal leading-relaxed">
                  {t("leave_requests.policy.min_work_days_desc")}
                </div>
              </div>
            </label>
            <input id="min-work-days" type="number" name="min_work_days_threshold" step="0.5" defaultValue={editingPolicy?.rule_config.min_work_days_threshold} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm font-mono shadow-sm" placeholder="Ex: 14" />
          </div>

          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50">
            <button type="button" onClick={() => { setIsPolicyModalOpen(false); setEditingPolicy(null); }} className="px-8 py-4 bg-white border border-slate-200 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all active:scale-95">{t("common.cancel").toUpperCase()}</button>
            <button type="submit" data-testid="save-policy-btn" className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all shadow-xl shadow-slate-200 active:scale-95">{editingPolicy ? t("common.save_changes").toUpperCase() : t("common.leave_requests.policy.save_btn").toUpperCase()}</button>
          </div>
        </form>
      </Modal>

      <AssignPolicyModal isOpen={isAssignModalOpen} onClose={() => setIsAssignModalOpen(false)} wsId={wsId!} />
    </div>
  );
}
