import React, { useState } from "react";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Users, Building, _Building2, Plus, Trash2, Calendar,
  Globe, _ChevronDown, Shield, Clock, Sliders,
} from "lucide-react";
import { useTranslation } from "react-i18next";

import { EmployeeSearch } from "@/components/common/EmployeeSearch";
import Modal from "@/components/ui/Modal";
import { useToast } from "@/components/ui/Toast";

import { fetchDepartments } from "@/lib/api/departments";
import {
  fetchPayrollRuleAssignments,
  createPayrollRuleAssignment,
  deletePayrollRuleAssignment,
} from "@/lib/api/payroll";
import { PayrollRuleAssignment, Department } from "@/lib/api/types";
import { cn } from "@/lib/utils";


interface Props {
  wsId: string;
  ruleId: string;
  /** calculation_method of parent rule — drives override config form */
  calculationMethod?: "percentage" | "fixed" | "tiered";
}

const SCOPE_ICONS = {
  workspace: Globe,
  department: Building,
  employee: Users,
} as const;

const SCOPE_COLORS = {
  workspace: { bg: "bg-amber-50 dark:bg-amber-950/40", text: "text-amber-600 dark:text-amber-400", border: "border-amber-100 dark:border-amber-800", badge: "bg-amber-100 dark:bg-amber-900/50 text-amber-700 dark:text-amber-300" },
  department: { bg: "bg-sky-50 dark:bg-sky-950/40", text: "text-sky-600 dark:text-sky-400", border: "border-sky-100 dark:border-sky-800", badge: "bg-sky-100 dark:bg-sky-900/50 text-sky-700 dark:text-sky-300" },
  employee: { bg: "bg-teal-50 dark:bg-teal-950/40", text: "text-teal-600 dark:text-teal-400", border: "border-teal-100 dark:border-teal-800", badge: "bg-teal-100 dark:bg-teal-900/50 text-teal-700 dark:text-teal-300" },
} as const;

function formatDate(d: string) {
  return new Date(d).toLocaleDateString("vi-VN", { day: "2-digit", month: "short", year: "numeric" });
}

export default function RuleAssignmentManager({ wsId, ruleId, calculationMethod }: Props) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const [isModalOpen, setIsModalOpen] = useState(false);
  const [deletingId, setDeletingId] = useState<string | null>(null);

  // --- Form state ---
  const [assigneeType, setAssigneeType] = useState<"workspace" | "department" | "employee">("employee");
  const [assigneeId, setAssigneeId] = useState<string>("");
  const [effectiveFrom, setEffectiveFrom] = useState(() => new Date().toISOString().split("T")[0]);
  const [effectiveTo, setEffectiveTo] = useState<string>("");
  const [hasOverride, setHasOverride] = useState(false);
  const [overrideRate, setOverrideRate] = useState<string>("");
  const [overrideAmount, setOverrideAmount] = useState<string>("");
  const [overrideBase, setOverrideBase] = useState<string>("gross");

  // --- Data ---
  const { data: assignments, isLoading } = useQuery({
    queryKey: ["payroll-rule-assignments", ruleId],
    queryFn: () => fetchPayrollRuleAssignments(wsId, ruleId),
    enabled: !!ruleId,
  });

  const { data: departments } = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId),
    enabled: !!wsId,
  });

  const deptMap = React.useMemo(() => {
    const m: Record<string, Department> = {};
    departments?.forEach((d) => { m[d.id] = d; });
    return m;
  }, [departments]);

  // --- Mutations ---
  const createMutation = useMutation({
    mutationFn: (data: Partial<PayrollRuleAssignment>) => createPayrollRuleAssignment(wsId, ruleId, data),
    onSuccess: () => {
      toast.success(t("common.success"));
      queryClient.invalidateQueries({ queryKey: ["payroll-rule-assignments", ruleId] });
      closeModal();
    },
    onError: (err: Error) => toast.error(err.message || t("common.error")),
  });

  const deleteMutation = useMutation({
    mutationFn: (assignmentId: string) => deletePayrollRuleAssignment(wsId, ruleId, assignmentId),
    onSuccess: () => {
      toast.success(t("common.success"));
      queryClient.invalidateQueries({ queryKey: ["payroll-rule-assignments", ruleId] });
      setDeletingId(null);
    },
    onError: (err: Error) => toast.error(err.message || t("common.error")),
  });

  // --- Handlers ---
  const closeModal = () => {
    setIsModalOpen(false);
    setAssigneeType("employee");
    setAssigneeId("");
    setEffectiveFrom(new Date().toISOString().split("T")[0]);
    setEffectiveTo("");
    setHasOverride(false);
    setOverrideRate("");
    setOverrideAmount("");
    setOverrideBase("gross");
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    let overrideConfig: Record<string, any> | undefined;
    if (hasOverride) {
      if (calculationMethod === "percentage" && overrideRate) {
        overrideConfig = { rate: parseFloat(overrideRate), base: overrideBase };
      } else if (calculationMethod === "fixed" && overrideAmount) {
        overrideConfig = { amount: parseFloat(overrideAmount) };
      }
    }
    createMutation.mutate({
      assignee_type: assigneeType,
      assignee_id: assigneeType === "workspace" ? undefined : assigneeId || undefined,
      effective_from: effectiveFrom,
      effective_to: effectiveTo || undefined,
      override_config: overrideConfig,
    } as any);
  };

  const getAssigneeName = (a: PayrollRuleAssignment) => {
    if (a.assignee_type === "workspace") return t("payroll_rules.scope_workspace", "Entire Workspace");
    if (a.assignee_type === "department") return deptMap[a.assignee_id!]?.name || a.assignee_id?.slice(0, 8) + "…";
    return a.assignee_id?.slice(0, 8) + "…";
  };

  const grouped = React.useMemo(() => {
    const g = { workspace: [] as PayrollRuleAssignment[], department: [] as PayrollRuleAssignment[], employee: [] as PayrollRuleAssignment[] };
    assignments?.forEach((a) => g[a.assignee_type]?.push(a));
    return g;
  }, [assignments]);

  const totalCount = assignments?.length ?? 0;

  return (
    <div className="space-y-5 mt-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-xl bg-teal-50 dark:bg-teal-950/40 text-teal-600 dark:text-teal-400 flex items-center justify-center shadow-sm border border-teal-100 dark:border-teal-800">
            <Shield size={16} />
          </div>
          <div>
            <h3 className="font-bold text-sm uppercase tracking-widest text-slate-700 dark:text-slate-200 leading-none">
              Rule Assignments
            </h3>
            <p className="text-[10px] text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1 opacity-70">
              {totalCount} active {totalCount === 1 ? "assignment" : "assignments"} across all scopes
            </p>
          </div>
        </div>
        <button
          onClick={() => setIsModalOpen(true)}
          className="flex items-center gap-2 bg-teal-600 hover:bg-teal-700 text-white px-5 py-2.5 rounded-xl font-bold text-[10px] uppercase tracking-widest transition-all shadow-lg shadow-teal-100 dark:shadow-black/30 active:scale-95"
        >
          <Plus size={14} /> Assign
        </button>
      </div>

      {/* Loading state */}
      {isLoading && (
        <div className="p-12 flex justify-center">
          <div className="w-8 h-8 border-[3px] border-teal-600 border-t-transparent rounded-full animate-spin" />
        </div>
      )}

      {/* Empty state */}
      {!isLoading && totalCount === 0 && (
        <div className="p-12 bg-slate-50 dark:bg-slate-800/30 rounded-2xl border-2 border-dashed border-slate-100 dark:border-slate-800 flex flex-col items-center gap-3">
          <div className="w-14 h-14 bg-white dark:bg-slate-900 rounded-2xl flex items-center justify-center shadow-sm border border-slate-100 dark:border-slate-800">
            <Users size={24} className="text-slate-200 dark:text-slate-700" />
          </div>
          <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">
            No assignments yet — click "Assign" to get started
          </p>
        </div>
      )}

      {/* Assignment cards by scope */}
      {!isLoading && totalCount > 0 && (
        <div className="space-y-3">
          {(["workspace", "department", "employee"] as const).map((scope) => {
            const items = grouped[scope];
            if (items.length === 0) return null;
            const colors = SCOPE_COLORS[scope];
            const Icon = SCOPE_ICONS[scope];
            return (
              <div key={scope} className="space-y-2">
                <div className="flex items-center gap-2 px-1">
                  <Icon size={12} className={colors.text + " opacity-60"} />
                  <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
                    {scope} level ({items.length})
                  </span>
                </div>
                {items.map((a) => (
                  <div
                    key={a.id}
                    className={cn(
                      "flex items-center justify-between p-4 rounded-xl border shadow-sm transition-all group hover:shadow-md",
                      colors.bg, colors.border,
                      deletingId === a.id && "ring-2 ring-rose-300 dark:ring-rose-700"
                    )}
                  >
                    <div className="flex items-center gap-4 min-w-0">
                      <div className={cn("w-10 h-10 rounded-xl flex items-center justify-center shrink-0", colors.badge)}>
                        <Icon size={18} />
                      </div>
                      <div className="min-w-0">
                        <p className="font-bold text-sm text-slate-700 dark:text-slate-200 uppercase tracking-tight truncate leading-none">
                          {getAssigneeName(a)}
                        </p>
                        <div className="flex items-center gap-3 mt-1.5 flex-wrap">
                          <span className="flex items-center gap-1 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                            <Calendar size={10} className="opacity-50" />
                            {formatDate(a.effective_from)}
                            {a.effective_to ? ` → ${formatDate(a.effective_to)}` : " → ∞"}
                          </span>
                          {a.override_config && (
                            <span className="flex items-center gap-1 text-[9px] font-bold bg-orange-100 dark:bg-orange-900/40 text-orange-700 dark:text-orange-300 px-2 py-0.5 rounded-md border border-orange-200 dark:border-orange-800">
                              <Sliders size={9} /> OVERRIDE
                              {a.override_config.rate != null && ` ${a.override_config.rate}%`}
                              {a.override_config.amount != null && ` ₫${Number(a.override_config.amount).toLocaleString()}`}
                            </span>
                          )}
                        </div>
                      </div>
                    </div>

                    {/* Delete button */}
                    {deletingId === a.id ? (
                      <div className="flex items-center gap-2 animate-in fade-in duration-200">
                        <button
                          onClick={() => deleteMutation.mutate(a.id)}
                          disabled={deleteMutation.isPending}
                          className="px-3 py-1.5 bg-rose-600 text-white rounded-lg text-[10px] font-bold uppercase tracking-widest hover:bg-rose-700 transition-all disabled:opacity-50"
                        >
                          {deleteMutation.isPending ? "…" : "Confirm"}
                        </button>
                        <button
                          onClick={() => setDeletingId(null)}
                          className="px-3 py-1.5 bg-slate-200 dark:bg-slate-700 text-slate-600 dark:text-slate-300 rounded-lg text-[10px] font-bold uppercase tracking-widest hover:bg-slate-300 dark:hover:bg-slate-600 transition-all"
                        >
                          Cancel
                        </button>
                      </div>
                    ) : (
                      <button
                        onClick={() => setDeletingId(a.id)}
                        className="p-2.5 text-slate-300 dark:text-slate-600 hover:text-rose-500 dark:hover:text-rose-400 rounded-xl hover:bg-white dark:hover:bg-slate-900 transition-all opacity-0 group-hover:opacity-100 active:scale-90 shadow-none hover:shadow-md border border-transparent hover:border-slate-100 dark:hover:border-slate-800"
                        aria-label="Delete assignment"
                      >
                        <Trash2 size={16} />
                      </button>
                    )}
                  </div>
                ))}
              </div>
            );
          })}
        </div>
      )}

      {/* Create Assignment Modal */}
      <Modal
        isOpen={isModalOpen}
        onClose={closeModal}
        title="Assign Rule"
        subtitle="Define who this rule applies to and when"
        icon={<Shield size={24} />}
        maxWidth="lg"
      >
        <form onSubmit={handleSubmit} className="space-y-8 pt-2 animate-in fade-in duration-300">
          {/* Scope selector */}
          <div className="space-y-3">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
              Assignment Scope
            </label>
            <div className="grid grid-cols-3 gap-3">
              {(["employee", "department", "workspace"] as const).map((scope) => {
                const Icon = SCOPE_ICONS[scope];
                const colors = SCOPE_COLORS[scope];
                const isActive = assigneeType === scope;
                return (
                  <button
                    key={scope}
                    type="button"
                    onClick={() => { setAssigneeType(scope); setAssigneeId(""); }}
                    className={cn(
                      "flex flex-col items-center gap-2.5 p-5 rounded-2xl border-2 transition-all active:scale-[0.97]",
                      isActive
                        ? `${colors.bg} ${colors.border} shadow-lg`
                        : "bg-slate-50 dark:bg-slate-800/50 border-transparent hover:border-slate-200 dark:hover:border-slate-700 hover:bg-white dark:hover:bg-slate-900"
                    )}
                  >
                    <Icon size={22} className={isActive ? colors.text : "text-slate-400 dark:text-slate-500"} />
                    <span className={cn("text-[10px] font-bold uppercase tracking-widest", isActive ? "text-slate-700 dark:text-slate-200" : "text-slate-500 dark:text-slate-400")}>
                      {scope}
                    </span>
                  </button>
                );
              })}
            </div>
          </div>

          {/* Assignee picker (employee or department) */}
          {assigneeType === "employee" && (
            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
                Select Employee
              </label>
              <EmployeeSearch
                workspaceId={wsId}
                value={assigneeId}
                onChange={(val: string) => setAssigneeId(val)}
                placeholder="Search employees…"
              />
            </div>
          )}

          {assigneeType === "department" && (
            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
                Select Department
              </label>
              <select
                required
                value={assigneeId}
                onChange={(e) => setAssigneeId(e.target.value)}
                className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-3.5 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-sky-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all uppercase tracking-tight shadow-sm"
              >
                <option value="">— Select department —</option>
                {departments?.map((d) => (
                  <option key={d.id} value={d.id}>{d.name}</option>
                ))}
              </select>
            </div>
          )}

          {assigneeType === "workspace" && (
            <div className="bg-amber-50 dark:bg-amber-950/30 border border-amber-100 dark:border-amber-800 rounded-2xl p-5 flex items-center gap-4">
              <Globe size={20} className="text-amber-600 dark:text-amber-400 shrink-0" />
              <p className="text-xs font-bold text-amber-800 dark:text-amber-200 uppercase tracking-tight">
                This rule will apply to all employees in the workspace
              </p>
            </div>
          )}

          {/* Date range */}
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-3">
              <label htmlFor="effective-from" className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-1.5">
                <Clock size={10} /> Effective From
              </label>
              <input
                id="effective-from"
                type="date"
                required
                value={effectiveFrom}
                onChange={(e) => setEffectiveFrom(e.target.value)}
                className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-3.5 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm"
              />
            </div>
            <div className="space-y-3">
              <label htmlFor="effective-to" className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-1.5">
                <Clock size={10} /> Effective To
                <span className="opacity-40 normal-case tracking-normal">(optional)</span>
              </label>
              <input
                id="effective-to"
                type="date"
                value={effectiveTo}
                onChange={(e) => setEffectiveTo(e.target.value)}
                className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-3.5 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm"
              />
            </div>
          </div>

          {/* Override config toggle */}
          {(calculationMethod === "percentage" || calculationMethod === "fixed") && (
            <div className="space-y-4">
              <label
                className={cn(
                  "flex items-center gap-4 p-5 rounded-2xl cursor-pointer transition-all border-2 active:scale-[0.98]",
                  hasOverride
                    ? "bg-orange-50 dark:bg-orange-950/30 border-orange-100 dark:border-orange-800 shadow-lg"
                    : "bg-slate-50 dark:bg-slate-800/50 border-transparent hover:border-slate-200 dark:hover:border-slate-700"
                )}
              >
                <input
                  type="checkbox"
                  checked={hasOverride}
                  onChange={(e) => setHasOverride(e.target.checked)}
                  className="w-5 h-5 rounded-lg text-orange-600 focus:ring-orange-500 border-slate-300 dark:border-slate-600 cursor-pointer"
                />
                <div>
                  <p className="font-bold text-sm text-slate-700 dark:text-slate-200 uppercase tracking-tight flex items-center gap-2">
                    <Sliders size={14} className="opacity-50" /> Override Default Config
                  </p>
                  <p className="text-[10px] text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1 opacity-70">
                    Apply a different rate or amount for this specific assignment
                  </p>
                </div>
              </label>

              {hasOverride && (
                <div className="bg-orange-50 dark:bg-orange-950/20 rounded-2xl p-5 border border-orange-100 dark:border-orange-800 space-y-4 animate-in slide-in-from-top-2 duration-200">
                  {calculationMethod === "percentage" && (
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-2">
                        <label className="text-[10px] font-bold text-orange-700 dark:text-orange-300 uppercase tracking-widest">Override Rate (%)</label>
                        <input
                          type="number"
                          step="0.01"
                          value={overrideRate}
                          onChange={(e) => setOverrideRate(e.target.value)}
                          placeholder="e.g. 10.5"
                          className="w-full bg-white dark:bg-slate-900 border border-orange-200 dark:border-orange-800 rounded-xl px-4 py-3 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-orange-500/20 outline-none"
                        />
                      </div>
                      <div className="space-y-2">
                        <label className="text-[10px] font-bold text-orange-700 dark:text-orange-300 uppercase tracking-widest">Override Base</label>
                        <select
                          value={overrideBase}
                          onChange={(e) => setOverrideBase(e.target.value)}
                          className="w-full bg-white dark:bg-slate-900 border border-orange-200 dark:border-orange-800 rounded-xl px-4 py-3 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-orange-500/20 outline-none uppercase tracking-tight"
                        >
                          <option value="gross">Gross Salary</option>
                          <option value="gross_salary">Gross Salary (alt)</option>
                          <option value="taxable_income">Taxable Income</option>
                        </select>
                      </div>
                    </div>
                  )}
                  {calculationMethod === "fixed" && (
                    <div className="space-y-2">
                      <label className="text-[10px] font-bold text-orange-700 dark:text-orange-300 uppercase tracking-widest">Override Amount (₫)</label>
                      <input
                        type="number"
                        value={overrideAmount}
                        onChange={(e) => setOverrideAmount(e.target.value)}
                        placeholder="e.g. 500000"
                        className="w-full bg-white dark:bg-slate-900 border border-orange-200 dark:border-orange-800 rounded-xl px-4 py-3 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-orange-500/20 outline-none"
                      />
                    </div>
                  )}
                </div>
              )}
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-4 pt-2 border-t border-slate-50 dark:border-slate-800">
            <button
              type="button"
              onClick={closeModal}
              className="flex-1 px-6 py-3.5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-500 dark:text-slate-400 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95"
            >
              {t("common.cancel")}
            </button>
            <button
              type="submit"
              disabled={createMutation.isPending || (assigneeType !== "workspace" && !assigneeId)}
              className="flex-[1.5] bg-teal-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-teal-700 transition-all shadow-xl shadow-teal-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed py-3.5"
            >
              {createMutation.isPending ? t("common.saving") + "…" : "Assign Rule"}
            </button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
