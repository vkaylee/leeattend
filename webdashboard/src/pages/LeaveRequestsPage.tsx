import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  CalendarDays, Calendar, CheckCircle2, XCircle, Send, Plus, X, Settings2, PlayCircle,
  UserPlus, FileText, ShieldCheck, UserCircle, Edit2, Trash2
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
  assignLeavePolicy,
  triggerAccrualRun,
  getCurrentWorkspaceId,
  fetchDepartments,
  Employee,
  LeaveRequest,
  EmployeeLeaveBalance,
  LeavePolicy
} from "../lib/api";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Combobox } from "@/components/ui/Combobox";
import Modal from "@/components/ui/Modal";
import { useToast } from "@/components/Toast";
import { AssignPolicyModal } from "@/components/leave/AssignPolicyModal";
import { EmployeeSearch } from "@/components/common/EmployeeSearch";

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

  const { data: employees = [], isLoading: employeesLoading } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId!),
    enabled: !!wsId,
  });

  const { data: departments = [], isLoading: departmentsLoading } = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId!),
    enabled: !!wsId,
  });

  const { data: balances = [], isLoading: balancesLoading } = useQuery({
    queryKey: ["leave-balances", wsId, selectedEmpId],
    queryFn: () => fetchEmployeeLeaveBalances(wsId!, selectedEmpId),
    enabled: !!wsId && !!selectedEmpId,
  });

  const { data: requests = [], isLoading: requestsLoading } = useQuery({
    queryKey: ["leave-requests", wsId, activeTab],
    queryFn: () => fetchWorkspaceLeaveRequests(wsId!, activeTab === "all" ? undefined : activeTab),
    enabled: !!wsId && activeTab !== "policies",
  });

  const { data: policies = [], isLoading: policiesLoading } = useQuery({
    queryKey: ["leave-policies", wsId],
    queryFn: () => fetchLeavePolicies(wsId!),
    enabled: !!wsId,
  });

  const calculateAnnualEntitlement = (policy: LeavePolicy) => {
    const config = policy.rule_config;
    const amount = config.accrual_amount;
    switch (config.frequency) {
      case 'monthly': return amount * 12;
      case 'quarterly': return amount * 4;
      case 'yearly': return amount;
      default: return amount;
    }
  };

  const getAccrualStrategy = (policy: LeavePolicy) => {
    return policy.rule_config.frequency;
  };

  const submitMutation = useMutation({
    mutationFn: (data: { employee_id: string; leave_type: string; start_date: string; end_date: string; reason?: string }) => 
      submitLeaveRequest(wsId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-requests", wsId] });
      setIsModalOpen(false);
    },
  });

  const approveMutation = useMutation({
    mutationFn: (data: { id: string; status: string }) => 
      approveLeaveRequest(wsId!, data.id, { status: data.status }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-requests", wsId] });
    },
  });

  const policyMutation = useMutation({
    mutationFn: (data: Partial<LeavePolicy>) => 
      editingPolicy 
        ? updateLeavePolicy(wsId!, editingPolicy.id, data) 
        : createLeavePolicy(wsId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-policies", wsId] });
      setIsPolicyModalOpen(false);
      setEditingPolicy(null);
      toast.success(editingPolicy ? t("leave_requests.policy.messages.update_success") : t("leave_requests.policy.messages.create_success"));
    },
  });

  const deletePolicyMutation = useMutation({
    mutationFn: (policyId: string) => deleteLeavePolicy(wsId!, policyId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-policies", wsId] });
      toast.success(t("leave_requests.policy.messages.delete_success"));
    },
  });

  const accrualMutation = useMutation({
    mutationFn: (date: string) => triggerAccrualRun(wsId!, { date }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-balances", wsId] });
      toast.success(t("leave_requests.messages.accrual_success"));
    },
  });

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const data = {
      employee_id: selectedEmpId,
      leave_type: leaveType,
      start_date: formData.get("start_date") as string,
      end_date: formData.get("end_date") as string,
      reason: formData.get("reason") as string,
    };
    submitMutation.mutate(data);
  };

  const handleCreatePolicy = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    
    // Construct the rule_config object expected by the backend
    const rule_config = {
      frequency: accrualStrategy as any, // "monthly" | "quarterly" | "yearly"
      accrual_amount: Number(formData.get("annual_entitlement")) / (accrualStrategy === "monthly" ? 12 : accrualStrategy === "quarterly" ? 4 : 1),
      accrual_timing: "start_of_period",
      carry_over_limit: Number(formData.get("carry_over_limit")),
    };

    policyMutation.mutate({
      name: formData.get("name") as string,
      leave_type: policyType,
      rule_config: rule_config as any,
    });
  };

  const getStatusStyle = (status: string) => {
    switch (status) {
      case "pending": return "bg-orange-100 text-orange-700";
      case "approved": return "bg-green-100 text-green-700";
      case "rejected": return "bg-red-100 text-red-700";
      default: return "bg-slate-100 text-slate-700";
    }
  };

  if (!wsId) return <div className="p-8 text-center">{t("common.select_workspace")}</div>;

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
        <div>
          <h1 className="text-3xl font-bold text-slate-900 tracking-tight">{t("leave_requests.title")}</h1>
          <p className="text-slate-500 mt-1">{t("leave_requests.subtitle")}</p>
        </div>
        <div className="flex items-center gap-3">
          <button onClick={() => setIsModalOpen(true)} className="bg-primary-600 hover:bg-primary-700 text-white font-black px-6 py-2.5 rounded-2xl flex items-center gap-2 shadow-lg shadow-primary-100 transition-all active:scale-95">
            <Plus className="w-5 h-5" />
            <span className="text-xs uppercase tracking-widest">{t("leave_requests.create_btn")}</span>
          </button>
        </div>
      </div>

      {/* Tabs Navigation */}
      <div className="flex items-center gap-1 bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50">
        <button onClick={() => setActiveTab("pending")} className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${activeTab === 'pending' ? 'bg-white text-amber-600 shadow-sm border border-slate-100' : 'text-slate-500 hover:text-slate-700'}`}>
          <Clock size={14} /> {t("common.status_pending")}
        </button>
        <button onClick={() => setActiveTab("approved")} className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${activeTab === 'approved' ? 'bg-white text-emerald-600 shadow-sm border border-slate-100' : 'text-slate-500 hover:text-slate-700'}`}>
          <CheckCircle2 size={14} /> {t("common.status_approved")}
        </button>
        <button onClick={() => setActiveTab("rejected")} className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${activeTab === 'rejected' ? 'bg-white text-rose-600 shadow-sm border border-slate-100' : 'text-slate-500 hover:text-slate-700'}`}>
          <XCircle size={14} /> {t("common.status_rejected")}
        </button>
        <button onClick={() => setActiveTab("policies")} className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${activeTab === 'policies' ? 'bg-white text-primary-600 shadow-sm border border-slate-100' : 'text-slate-500 hover:text-slate-700'}`}>
          <ShieldCheck size={14} /> {t("leave_requests.tabs.policies")}
        </button>
      </div>

            <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">

              {activeTab === "policies" ? (

                <div className="space-y-6">

                  <div className="flex justify-end gap-3">

                    <button onClick={() => setIsAssignModalOpen(true)} className="flex items-center gap-2 bg-white border border-slate-200 text-slate-700 px-5 py-2.5 rounded-2xl hover:bg-slate-50 transition-all font-black text-[10px] uppercase tracking-widest shadow-sm">

                      <UserPlus className="w-4 h-4" /> {t("leave_requests.policy.assign_btn")}

                    </button>

                    <button onClick={() => { setEditingPolicy(null); setIsPolicyModalOpen(true); }} className="flex items-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl hover:bg-primary-700 transition-all font-black text-[10px] uppercase tracking-widest shadow-lg shadow-primary-100">

                      <Plus className="w-4 h-4" /> {t("leave_requests.policy.add_btn")}

                    </button>

                  </div>

                  <div className="bg-white border border-slate-100 rounded-[32px] overflow-hidden shadow-sm">

                    <table className="w-full text-left">

                      <thead>

                        <tr className="bg-slate-50/50 border-b border-slate-100">

                          <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("leave_requests.policy.table.name")}</th>

                          <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("leave_requests.policy.table.type")}</th>

                          <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("leave_requests.policy.table.entitlement")}</th>

                          <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("leave_requests.policy.table.strategy")}</th>

                          <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.actions")}</th>

                        </tr>

                      </thead>

                      <tbody className="divide-y divide-slate-50">

                        {policiesLoading ? (

                          <tr><td colSpan={5} className="px-6 py-12 text-center text-slate-400 font-bold italic">{t("common.loading")}</td></tr>

                        ) : policies.length === 0 ? (

                          <tr><td colSpan={5} className="px-6 py-12 text-center text-slate-400 font-bold italic">{t("leave_requests.policy.messages.no_policies")}</td></tr>

                        ) : policies.map((policy) => (

                          <tr key={policy.id} className="hover:bg-slate-50/50 transition-colors group">

                            <td className="px-6 py-5 font-black text-slate-900 text-sm tracking-tight">{policy.name}</td>

                            <td className="px-6 py-5 capitalize text-slate-500 font-bold text-xs">{policy.leave_type || 'vacation'}</td>

                            <td className="px-6 py-5 text-primary-600 font-black text-sm font-mono">{calculateAnnualEntitlement(policy)} {t("common.days_short")}</td>

                            <td className="px-6 py-5 text-slate-500 font-bold text-[10px] uppercase tracking-widest">{t(`leave_requests.policy.strategies.${getAccrualStrategy(policy)}`)}</td>

                            <td className="px-6 py-5 text-right">

                              <div className="flex justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">

                                <button onClick={() => { setEditingPolicy(policy); setPolicyType(policy.leave_type || 'vacation'); setAccrualStrategy(getAccrualStrategy(policy)); setIsPolicyModalOpen(true); }} className="p-2.5 bg-white text-slate-400 hover:text-primary-600 rounded-xl shadow-md border border-slate-100"><Edit2 className="w-4 h-4" /></button>

                                <button onClick={() => { if (confirm(t("leave_requests.policy.messages.confirm_delete"))) deletePolicyMutation.mutate(policy.id); }} className="p-2.5 bg-white text-slate-400 hover:text-rose-600 rounded-xl shadow-md border border-slate-100"><Trash2 className="w-4 h-4" /></button>

                              </div>

                            </td>

                          </tr>

                        ))}

                      </tbody>

                    </table>

                  </div>

                </div>

                      ) : (

                        /* REQUESTS TABLE */

                        <div className="bg-white border border-slate-100 rounded-[32px] overflow-hidden shadow-sm">

                          <table className="w-full text-left">

                            <thead>

                              <tr className="bg-slate-50/50 border-b border-slate-100">

                                <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.employee")}</th>

                                <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.type")}</th>

                                <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.time")}</th>

                                <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.status")}</th>

                                <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.actions")}</th>

                              </tr>

                            </thead>

                            <tbody className="divide-y divide-slate-50">

                              {requestsLoading ? (

                                <tr><td colSpan={5} className="px-6 py-12 text-center text-slate-400 font-bold italic">{t("common.loading")}</td></tr>

                              ) : requests.length === 0 ? (

                                <tr><td colSpan={5} className="px-6 py-12 text-center text-slate-400 font-bold italic">{t("leave_requests.messages.no_requests")}</td></tr>

                              ) : requests.map((req) => {

                                const emp = employees.find(e => e.id === req.employee_id);

                                return (

                                  <tr key={req.id} className="hover:bg-slate-50/50 transition-colors group">

                                    <td className="px-6 py-5">

                                      <div className="flex items-center gap-3">

                                        <div className="w-8 h-8 rounded-lg bg-slate-100 flex items-center justify-center text-slate-500 font-black text-xs">{emp?.last_name[0]}</div>

                                        <div>

                                          <div className="font-black text-slate-900 text-sm tracking-tight">{emp ? `${emp.first_name} ${emp.last_name}` : 'N/A'}</div>

                                          <div className="text-[10px] text-slate-400 font-bold uppercase tracking-tighter">{emp?.employee_id_number}</div>

                                        </div>

                                      </div>

                                    </td>

                                    <td className="px-6 py-5">

                                      <span className="bg-slate-100 text-slate-600 px-2 py-1 rounded-lg text-[10px] font-black uppercase tracking-tighter border border-slate-200">{req.leave_type}</span>

                                    </td>

                                    <td className="px-6 py-5">

                                      <div className="flex flex-col">

                                        <div className="text-xs font-black text-slate-700">{formatDate(req.start_date)} - {formatDate(req.end_date)}</div>

                                        <div className="text-[10px] text-slate-400 font-bold line-clamp-1 max-w-[200px]">{req.reason || t("common.no_description")}</div>

                                      </div>

                                    </td>

                                    <td className="px-6 py-5">

                                      <span className={cn("px-2.5 py-1 rounded-full text-[10px] font-black uppercase tracking-widest", 

                                        req.status === 'pending' ? "bg-amber-50 text-amber-600 border border-amber-100" : 

                                        req.status === 'approved' ? "bg-emerald-50 text-emerald-600 border border-emerald-100" : 

                                        "bg-rose-50 text-rose-600 border border-rose-100"

                                      )}>

                                        {req.status === 'pending' ? t("common.status_pending") : req.status === 'approved' ? t("common.status_approved") : t("common.status_rejected")}

                                      </span>

                                    </td>

                                    <td className="px-6 py-5 text-right">

                                      {req.status === 'pending' && (

                                        <div className="flex justify-end gap-2">

                                          <button onClick={() => approveMutation.mutate({ id: req.id, status: 'approved' })} className="p-2.5 bg-white text-emerald-600 hover:bg-emerald-50 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90" title={t("common.approve")}><CheckCircle2 className="w-5 h-5" /></button>

                                          <button onClick={() => approveMutation.mutate({ id: req.id, status: 'rejected' })} className="p-2.5 bg-white text-rose-600 hover:bg-rose-50 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90" title={t("common.reject")}><XCircle className="w-5 h-5" /></button>

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

              

      {/* Modals remain below */}
      {/* Submit Modal */}
      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title={t("leave_requests.modal.submit_title")}
        subtitle={t("leave_requests.modal.submit_subtitle")}
        icon={<CalendarDays className="w-6 h-6" />}
      >
        <form onSubmit={handleSubmit} className="space-y-6">
          <input type="hidden" name="employee_id" value={selectedEmpId} />
          <div className="space-y-2">
            <label 
              htmlFor="leave-emp"
              className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1"
            >
              {t("common.employee")}
            </label>
            <EmployeeSearch
              id="leave-emp"
              workspaceId={wsId!}
              value={selectedEmpId}
              onChange={setSelectedEmpId}
              className="bg-slate-50 border-slate-200"
            />
          </div>

          {selectedEmpId && balances.length > 0 && (
            <div className="bg-primary-50/50 p-4 rounded-2xl border border-primary-100/50 grid grid-cols-2 gap-3">
              {balances.map(b => (
                <div key={b.leave_type} className="space-y-0.5">
                  <p className="text-[10px] font-black uppercase tracking-widest text-primary-600/60">
                    {b.leave_type === 'vacation' ? t("leave_requests.types.vacation") : b.leave_type === 'sick' ? t("leave_requests.types.sick") : b.leave_type}
                  </p>
                  <p className="text-sm font-bold text-slate-700">
                    {t("leave_requests.remaining")}: <span className="text-primary-600 text-lg">{b.remaining}</span> / {b.entitlement + b.carried_over} {t("common.days")}
                  </p>
                </div>
              ))}
            </div>
          )}

          <div className="space-y-2">
            <input type="hidden" name="leave_type" value={leaveType} />
            <label 
              htmlFor="leave-type"
              className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1"
            >
              {t("common.type")}
            </label>
            <Combobox
              id="leave-type"
              options={[
                { value: "vacation", label: t("leave_requests.types.vacation_full"), icon: <Calendar className="w-4 h-4" /> },
                { value: "sick", label: t("leave_requests.types.sick_full"), icon: <FileText className="w-4 h-4" /> },
                { value: "personal", label: t("leave_requests.types.personal"), icon: <CalendarDays className="w-4 h-4" /> },
              ]}
              value={leaveType}
              onChange={setLeaveType}
              className="bg-slate-50 border-slate-200"
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1">
              <label htmlFor="start-date" className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.from_date")}</label>
              <input id="start-date" type="date" name="start_date" required className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-xl outline-none font-bold" />
            </div>
            <div className="space-y-1">
              <label htmlFor="end-date" className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.to_date")}</label>
              <input id="end-date" type="date" name="end_date" required className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-xl outline-none font-bold" />
            </div>
          </div>
          <div className="space-y-1">
            <label htmlFor="leave-reason" className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.reason")}</label>
            <textarea id="leave-reason" name="reason" rows={3} className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-xl outline-none font-medium resize-none" placeholder={t("leave_requests.modal.reason_placeholder")}></textarea>
          </div>
          <div className="pt-4 flex gap-3">
            <button type="button" onClick={() => setIsModalOpen(false)} className="flex-1 px-6 py-3 border border-slate-200 rounded-xl font-bold text-slate-500 hover:bg-slate-50 transition-all">{t("common.cancel")}</button>
            <button type="submit" className="flex-1 px-6 py-3 bg-slate-900 text-white rounded-xl font-bold hover:bg-slate-800 transition-all shadow-lg shadow-slate-200">{t("leave_requests.modal.submit_btn")}</button>
          </div>
        </form>
      </Modal>

      {/* Policy Modal */}
      <Modal
        isOpen={isPolicyModalOpen}
        onClose={() => {
          setIsPolicyModalOpen(false);
          setEditingPolicy(null);
        }}
        title={editingPolicy ? t("common.leave_requests.policy.edit_title") : t("common.leave_requests.policy.title")}
        subtitle={t("leave_requests.policy.subtitle")}
        icon={<ShieldCheck className="w-6 h-6" />}
      >
        <form onSubmit={handleCreatePolicy} className="p-2 space-y-6">
          <div className="space-y-1">
            <label htmlFor="policy-name" className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">{t("leave_requests.policy.table.name")}</label>
            <input id="policy-name" name="name" required defaultValue={editingPolicy?.name} className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-xl outline-none font-bold" placeholder={t("leave_requests.policy.name_placeholder")} />
          </div>
          <div className="space-y-2">
            <label htmlFor="policy-type" className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">{t("leave_requests.policy.table.type")}</label>
            <Combobox
              id="policy-type"
              options={[
                { value: "vacation", label: t("leave_requests.types.vacation"), icon: <Calendar className="w-4 h-4" /> },
                { value: "sick", label: t("leave_requests.types.sick"), icon: <FileText className="w-4 h-4" /> },
              ]}
              value={policyType}
              onChange={setPolicyType}
              className="bg-slate-50 border-slate-200"
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1">
              <label htmlFor="annual-entitlement" className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">{t("leave_requests.policy.table.entitlement")}</label>
              <input id="annual-entitlement" type="number" name="annual_entitlement" required defaultValue={editingPolicy ? calculateAnnualEntitlement(editingPolicy) : 12} className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-xl outline-none font-bold" />
            </div>
            <div className="space-y-1">
              <label htmlFor="carry-over" className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">{t("leave_requests.policy.carry_over_limit")}</label>
              <input id="carry-over" type="number" name="carry_over_limit" required defaultValue={editingPolicy?.rule_config.carry_over_limit ?? 5} className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-xl outline-none font-bold" />
            </div>
          </div>
          <div className="space-y-2">
            <label htmlFor="accrual-strategy" className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">{t("leave_requests.policy.table.strategy")}</label>
            <Combobox
              id="accrual-strategy"
              options={[
                { value: "monthly", label: t("leave_requests.policy.strategies.monthly"), description: t("leave_requests.policy.strategies.monthly_desc") },
                { value: "quarterly", label: t("leave_requests.policy.strategies.quarterly"), description: t("leave_requests.policy.strategies.quarterly_desc") },
                { value: "yearly", label: t("leave_requests.policy.strategies.yearly"), description: t("leave_requests.policy.strategies.yearly_desc") },
              ]}
              value={accrualStrategy}
              onChange={setAccrualStrategy}
              className="bg-slate-50 border-slate-200"
            />
          </div>
          <div className="pt-4 flex gap-3">
            <button type="button" onClick={() => { setIsPolicyModalOpen(false); setEditingPolicy(null); }} className="flex-1 px-6 py-3 border border-slate-200 rounded-xl font-bold text-slate-500 hover:bg-slate-50 transition-colors">{t("common.cancel")}</button>
            <button type="submit" data-testid="save-policy-btn" className="flex-1 px-6 py-3 bg-slate-900 text-white rounded-xl font-bold hover:bg-slate-800 transition-colors shadow-lg shadow-slate-100">
              {editingPolicy ? t("common.save_changes") : t("common.leave_requests.policy.save_btn")}
            </button>
          </div>
                </form>
              </Modal>
        
              <AssignPolicyModal 
                isOpen={isAssignModalOpen}
                onClose={() => setIsAssignModalOpen(false)}
                wsId={wsId}
              />
            </div>
          );
        }
        