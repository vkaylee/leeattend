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
  Employee,
  LeaveRequest,
  EmployeeLeaveBalance,
  LeavePolicy
} from "../lib/api";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Combobox } from "@/components/ui/Combobox";
import Modal from "@/components/ui/Modal";
import { useToast } from "@/components/Toast";

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
  const [activeTab, setActiveTab] = useState<string>("pending");
  const [selectedEmpId, setSelectedEmpId] = useState<string>("");
  const [leaveType, setLeaveType] = useState<string>("vacation");
  const [policyType, setPolicyType] = useState<string>("vacation");
  const [accrualStrategy, setAccrualStrategy] = useState<string>("monthly");
  const [assignEmpId, setAssignEmpId] = useState<string>("");
  const [assignPolicyId, setAssignPolicyId] = useState<string>("");

  const { data: employees = [] } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId!),
    enabled: !!wsId,
  });

  const { data: balances = [], isLoading: balancesLoading } = useQuery({
    queryKey: ["leave-balances", wsId, selectedEmpId],
    queryFn: () => fetchEmployeeLeaveBalances(wsId!, selectedEmpId),
    enabled: !!wsId && !!selectedEmpId,
  });

  const { data: requests = [], isLoading } = useQuery({
    queryKey: ["leave-requests", wsId, activeTab],
    queryFn: () => fetchWorkspaceLeaveRequests(wsId!, activeTab === "all" ? undefined : activeTab),
    enabled: !!wsId && activeTab !== "policies",
  });

  const { data: policies = [] } = useQuery({
    queryKey: ["leave-policies", wsId],
    queryFn: () => fetchLeavePolicies(wsId!),
    enabled: !!wsId && activeTab === "policies",
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

  const assignMutation = useMutation({
    mutationFn: (data: { employee_id: string; leave_policy_id: string; effective_date: string }) => 
      assignLeavePolicy(wsId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-balances", wsId] });
      setIsAssignModalOpen(false);
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
      rule_config: rule_config as any,
    });
  };

  const handleAssignPolicy = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    assignMutation.mutate({
      employee_id: assignEmpId,
      leave_policy_id: assignPolicyId,
      effective_date: formData.get("effective_date") as string,
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
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-extrabold tracking-tight text-slate-900">{t("leave_requests.title")}</h1>
          <p className="text-slate-500 mt-1">{t("leave_requests.subtitle")}</p>
        </div>
        <button 
          onClick={() => setIsModalOpen(true)}
          className="flex items-center gap-2 bg-slate-900 text-white px-6 py-3 rounded-2xl hover:bg-slate-800 transition-all shadow-xl shadow-slate-200 font-bold"
        >
          <Plus className="w-5 h-5" />
          {t("leave_requests.create_btn")}
        </button>
      </div>


      {/* Tabs */}
      <div className="flex gap-2 p-1 bg-slate-100 w-fit rounded-2xl">
        {['pending', 'approved', 'rejected', 'all', 'policies'].map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-6 py-2 rounded-xl font-bold text-sm transition-all ${activeTab === tab ? 'bg-white text-primary-600 shadow-sm' : 'text-slate-500 hover:text-slate-700'}`}
          >
            {tab === 'pending' ? t("common.status_pending") : 
             tab === 'approved' ? t("common.status_approved") : 
             tab === 'rejected' ? t("common.status_rejected") : 
             tab === 'all' ? t("common.all") : t("leave_requests.tabs.policies")}
          </button>
        ))}
      </div>

      {activeTab === "policies" ? (
        <div className="space-y-6">
          <div className="flex justify-end gap-3">
            <button 
              onClick={() => setIsAssignModalOpen(true)}
              className="flex items-center gap-2 bg-white border border-slate-200 text-slate-700 px-4 py-2 rounded-xl hover:bg-slate-50 transition-all font-bold text-sm"
            >
              <UserPlus className="w-4 h-4" />
              {t("leave_requests.policy.assign_btn")}
            </button>
            <button 
              onClick={() => setIsPolicyModalOpen(true)}
              className="flex items-center gap-2 bg-slate-900 text-white px-4 py-2 rounded-xl hover:bg-slate-800 transition-all font-bold text-sm shadow-lg shadow-slate-100"
            >
              <Plus className="w-4 h-4" />
              {t("leave_requests.policy.add_btn")}
            </button>
          </div>
          <div className="bg-white border border-slate-200 rounded-3xl overflow-hidden shadow-sm">
            <table className="w-full text-left">
              <thead className="bg-slate-50 border-b border-slate-200">
                <tr>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600">{t("leave_requests.policy.table.name")}</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600">{t("leave_requests.policy.table.type")}</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600">{t("leave_requests.policy.table.entitlement")}</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600">{t("leave_requests.policy.table.strategy")}</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600">Carry-over</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600 text-right">{t("common.actions")}</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {policies.length === 0 ? (
                  <tr><td colSpan={6} className="px-6 py-8 text-center text-slate-500 italic">{t("leave_requests.policy.messages.no_policies")}</td></tr>
                ) : policies.map((policy) => (
                  <tr key={policy.id} className="hover:bg-slate-50/50 transition-colors">
                    <td className="px-6 py-4 font-medium text-slate-900">{policy.name}</td>
                    <td className="px-6 py-4 capitalize text-slate-600">{policy.leave_type || 'vacation'}</td>
                    <td className="px-6 py-4 text-slate-600 font-bold">{calculateAnnualEntitlement(policy)} {t("common.days")}</td>
                    <td className="px-6 py-4 text-sm text-slate-600">
                      {t(`leave_requests.policy.strategies.${getAccrualStrategy(policy)}`)}
                    </td>
                    <td className="px-6 py-4 text-slate-600">{policy.rule_config.carry_over_limit || 0} {t("common.days")}</td>
                    <td className="px-6 py-4 text-right">
                      <div className="flex justify-end gap-2">
                        <button 
                          onClick={() => {
                            setEditingPolicy(policy);
                            setPolicyType(policy.leave_type || 'vacation');
                            setAccrualStrategy(getAccrualStrategy(policy));
                            setIsPolicyModalOpen(true);
                          }}
                          className="p-2 text-slate-400 hover:text-primary-600 hover:bg-primary-50 rounded-lg transition-all"
                        >
                          <Edit2 className="w-4 h-4" />
                        </button>
                        <button 
                          onClick={() => {
                            if (confirm(t("leave_requests.policy.messages.confirm_delete"))) {
                              deletePolicyMutation.mutate(policy.id);
                            }
                          }}
                          className="p-2 text-slate-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-all"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      ) : (
        <div className="bg-white border border-slate-200 rounded-3xl overflow-hidden shadow-sm">
          <table className="w-full text-left">
            <thead className="bg-slate-50 border-b border-slate-200">
              <tr>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600">{t("common.employee")}</th>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600">{t("common.type")}</th>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600">{t("common.time")}</th>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600">{t("common.reason")}</th>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600">{t("common.status")}</th>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600 text-right">{t("common.actions")}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {isLoading ? (
                <tr><td colSpan={6} className="px-6 py-8 text-center text-slate-500">{t("common.loading")}</td></tr>
              ) : requests.length === 0 ? (
                <tr><td colSpan={6} className="px-6 py-8 text-center text-slate-500 italic">{t("leave_requests.messages.no_requests")}</td></tr>
              ) : requests.map((req) => {
                const emp = employees.find(e => e.id === req.employee_id);
                return (
                  <tr key={req.id} className="hover:bg-slate-50/50 transition-colors">
                    <td className="px-6 py-4">
                      <div className="font-medium text-slate-900">{emp ? `${emp.first_name} ${emp.last_name}` : 'N/A'}</div>
                    </td>
                    <td className="px-6 py-4 capitalize text-slate-600">{req.leave_type}</td>
                    <td className="px-6 py-4 text-sm text-slate-600">
                      {formatDate(req.start_date)} - {formatDate(req.end_date)}
                    </td>
                    <td className="px-6 py-4 text-sm text-slate-500 max-w-xs truncate">{req.reason || '-'}</td>
                    <td className="px-6 py-4">
                      <span className={`px-2 py-1 rounded-full text-xs font-bold ${getStatusStyle(req.status)}`}>
                        {req.status === 'pending' ? t("common.status_pending") : req.status === 'approved' ? t("common.status_approved") : t("common.status_rejected")}
                      </span>
                    </td>
                    <td className="px-6 py-4 text-right">
                      {req.status === 'pending' && (
                        <div className="flex justify-end gap-2">
                          <button 
                            onClick={() => approveMutation.mutate({ id: req.id, status: 'approved' })}
                            className="p-2 text-green-600 hover:bg-green-50 rounded-lg transition-all"
                            title={t("common.approve")}
                          >
                            <CheckCircle2 className="w-5 h-5" />
                          </button>
                          <button 
                            onClick={() => approveMutation.mutate({ id: req.id, status: 'rejected' })}
                            className="p-2 text-red-600 hover:bg-red-50 rounded-lg transition-all"
                            title={t("common.reject")}
                          >
                            <XCircle className="w-5 h-5" />
                          </button>
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
            <Combobox
              id="leave-emp"
              options={employees.map((e) => ({
                value: e.id,
                label: `${e.first_name} ${e.last_name}`,
                description: `${t("employees.table.id")}: ${e.employee_id_number}`,
                icon: (
                  <div className="bg-primary-100 p-2 rounded-xl text-primary-600 shadow-inner">
                    <UserCircle className="w-4 h-4" />
                  </div>
                ),
              }))}
              value={selectedEmpId}
              onChange={setSelectedEmpId}
              placeholder={t("common.select_employee")}
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
        title={editingPolicy ? t("leave_requests.policy.edit_title") : t("leave_requests.policy.title")}
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
              <label htmlFor="carry-over" className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">Carry-over limit</label>
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
            <button type="submit" className="flex-1 px-6 py-3 bg-slate-900 text-white rounded-xl font-bold hover:bg-slate-800 transition-colors shadow-lg shadow-slate-200">
              {editingPolicy ? t("common.save_changes") : t("leave_requests.policy.save_btn")}
            </button>
          </div>
        </form>
      </Modal>

      {/* Assign Modal */}
      <Modal
        isOpen={isAssignModalOpen}
        onClose={() => setIsAssignModalOpen(false)}
        title={t("leave_requests.policy.assign_title")}
        subtitle={t("leave_requests.policy.assign_subtitle")}
        icon={<UserCircle className="w-6 h-6" />}
      >
        <form onSubmit={handleAssignPolicy} className="p-2 space-y-6">
          <div className="space-y-2">
            <label 
              htmlFor="assign-emp"
              className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1"
            >
              {t("common.employee")}
            </label>
            <Combobox
              id="assign-emp"
              options={employees.map(e => ({
                value: e.id,
                label: `${e.first_name} ${e.last_name}`,
                icon: <UserCircle className="w-4 h-4" />
              }))}
              value={assignEmpId}
              onChange={setAssignEmpId}
              placeholder={t("common.select_employee")}
              className="bg-slate-50 border-slate-200"
            />
          </div>
          <div className="space-y-2">
            <label 
              htmlFor="assign-policy"
              className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1"
            >
              {t("leave_requests.policy.title_single")}
            </label>
            <Combobox
              id="assign-policy"
              options={policies.map(p => ({
                value: p.id,
                label: p.name,
                description: p.leave_type === 'vacation' ? t("leave_requests.types.vacation") : t("leave_requests.types.sick"),
                icon: <ShieldCheck className="w-4 h-4" />
              }))}
              value={assignPolicyId}
              onChange={setAssignPolicyId}
              placeholder={t("leave_requests.policy.select_placeholder")}
              className="bg-slate-50 border-slate-200"
            />
          </div>
          <div className="space-y-1">
            <label htmlFor="assign-date" className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.effective_from")}</label>
            <input id="assign-date" type="date" name="effective_date" required defaultValue={new Date().toISOString().split('T')[0]} className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-xl outline-none font-bold" />
          </div>
          <div className="pt-4 flex gap-3">
            <button type="button" onClick={() => setIsAssignModalOpen(false)} className="flex-1 px-6 py-3 border border-slate-200 rounded-xl font-bold text-slate-500 hover:bg-slate-50 transition-colors">{t("common.cancel")}</button>
            <button type="submit" className="flex-1 px-6 py-3 bg-slate-900 text-white rounded-xl font-bold hover:bg-slate-800 transition-colors shadow-lg shadow-slate-200">{t("leave_requests.policy.assign_now_btn")}</button>
          </div>
        </form>
      </Modal>
    </div>
  );
}