import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Building2, 
  User, 
  Plus, 
  Settings2, 
  Clock, 
  ShieldAlert, 
  Calculator,
  Zap,
  Trash2,
  ChevronRight,
  Filter
} from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { LABEL_BASE } from "@/components/ui/form-field";
import Sheet from "@/components/ui/sheet";
import { useToast } from "@/components/ui/toast";
import { 
  fetchPolicyAssignments, 
  upsertPolicyAssignment, 
  deletePolicyAssignment, 
  fetchDepartments, 
  fetchEmployees,
  Employee, UpsertPolicyAssignmentRequest 
} from "@/lib/api";
import { cn } from "@/lib/utils";

// We'll build a slide-over later, but for now let's structure the table.

export function PolicyAssignmentsSection({ wsId }: { wsId: string }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const { data: assignments = [], isLoading: isLoadingAssignments } = useQuery({
    queryKey: ["attendance-policy-assignments", wsId],
    queryFn: () => fetchPolicyAssignments(wsId),
  });

  const { data: departments = [] } = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId),
  });

  const { data: employeesResponse } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId, 1, 1000),
  });

  const employees = employeesResponse?.data ?? [];

  const [activeFilter, setActiveFilter] = useState<"ALL" | "GROUP" | "USER">("ALL");

  const filteredAssignments = assignments.filter(a => activeFilter === "ALL" || a.target_type === activeFilter);

  // Helper to resolve name
  const getTargetName = (type: string, id: string) => {
    if (type === "GROUP") {
      const dept = departments.find(d => d.id === id);
      return dept ? dept.name : "Unknown Department";
    } else {
      const emp = employees.find((e: Employee) => e.id === id);
      return emp ? emp.full_name : "Unknown Employee";
    }
  };

  const workUnitLabel = (v: number) => {
    if (v === 1) return t("common.workspace_settings.work_unit_options.1min");
    if (v === 15) return t("common.workspace_settings.work_unit_options.15min");
    if (v === 30) return t("common.workspace_settings.work_unit_options.30min");
    if (v === 60) return t("common.workspace_settings.work_unit_options.60min");
    return `${v ?? ""} min`;
  };

  const roundingLabel = (v: string) => {
    const map: Record<string, string> = { 
      none: t("common.workspace_settings.rounding_options.none"), 
      floor: t("common.workspace_settings.rounding_options.floor"), 
      ceil: t("common.workspace_settings.rounding_options.ceil"), 
      round: t("common.workspace_settings.rounding_options.round") 
    };
    return map[v] || v;
  };

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deletePolicyAssignment(wsId, id),
    onSuccess: () => {
      toast.success(t("attendance_policy_overrides.messages.delete_success"));
      void queryClient.invalidateQueries({ queryKey: ["attendance-policy-assignments", wsId] });
    }
  });

  const [isDrawerOpen, setIsDrawerOpen] = useState(false);
  const [targetType, setTargetType] = useState<"USER" | "GROUP">("GROUP");
  const [targetId, setTargetId] = useState("");
  const [workUnitMinutes, setWorkUnitMinutes] = useState(1);
  const [roundingPolicy, setRoundingPolicy] = useState("none");
  const [earlyArrivalStrategy, setEarlyArrivalStrategy] = useState<"allow" | "block" | "clamp">("allow");
  const [validFrom, setValidFrom] = useState(() => new Date().toISOString().split("T")[0]);

  const upsertMutation = useMutation({
    mutationFn: (data: UpsertPolicyAssignmentRequest) => upsertPolicyAssignment(wsId, data),
    onSuccess: () => {
      toast.success(t("attendance_policy_overrides.messages.save_success"));
      void queryClient.invalidateQueries({ queryKey: ["attendance-policy-assignments", wsId] });
      setIsDrawerOpen(false);
      setTargetId("");
    }
  });

  const handleSave = () => {
    if (!targetId) {
      toast.error(t("attendance_policy_overrides.messages.validation_select_target"));
      return;
    }
    upsertMutation.mutate({
      target_type: targetType,
      target_id: targetId,
      work_unit_minutes: workUnitMinutes,
      rounding_policy: roundingPolicy,
      early_arrival_strategy: earlyArrivalStrategy,
      valid_from: validFrom,
    });
  };

  const targetOptions = targetType === "GROUP"
    ? departments.map(d => ({ value: d.id, label: d.name, icon: <Building2 className="w-4 h-4 text-indigo-500" /> }))
    : employees.map((e: Employee) => ({ value: e.id, label: e.full_name, description: e.employee_id_number, icon: <User className="w-4 h-4 text-teal-500" /> }));


  if (isLoadingAssignments) return <div className="h-40 animate-pulse bg-slate-100 dark:bg-slate-800 rounded-[2.5rem]"></div>;

  return (
    <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden mt-8">
      <div className="p-10 border-b border-slate-50 dark:border-slate-800">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase">{t("attendance_policy_overrides.title")}</h2>
            <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">{t("attendance_policy_overrides.description")}</p>
          </div>
          <div className="flex items-center gap-3">
            <div className="flex bg-slate-100 dark:bg-slate-800 p-1 rounded-xl">
              <button className={cn("px-4 py-2 rounded-lg text-xs font-bold tracking-widest uppercase transition-all", activeFilter === "ALL" ? "bg-white dark:bg-slate-900 shadow-sm text-slate-800 dark:text-slate-200" : "text-slate-500 hover:text-slate-700")} onClick={() => { setActiveFilter("ALL"); }}>{t("attendance_policy_overrides.filters.all")}</button>
              <button className={cn("px-4 py-2 rounded-lg text-xs font-bold tracking-widest uppercase transition-all", activeFilter === "GROUP" ? "bg-white dark:bg-slate-900 shadow-sm text-slate-800 dark:text-slate-200" : "text-slate-500 hover:text-slate-700")} onClick={() => { setActiveFilter("GROUP"); }}>{t("attendance_policy_overrides.filters.depts")}</button>
              <button className={cn("px-4 py-2 rounded-lg text-xs font-bold tracking-widest uppercase transition-all", activeFilter === "USER" ? "bg-white dark:bg-slate-900 shadow-sm text-slate-800 dark:text-slate-200" : "text-slate-500 hover:text-slate-700")} onClick={() => { setActiveFilter("USER"); }}>{t("attendance_policy_overrides.filters.users")}</button>
            </div>
            <Button icon={<Plus size={18} />} onClick={() => { setIsDrawerOpen(true); }}>{t("attendance_policy_overrides.add_button")}</Button>
          </div>
        </div>
      </div>

      <div className="p-10">
        {filteredAssignments.length === 0 ? (
          <div className="text-center py-12 border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-3xl bg-slate-50/50 dark:bg-slate-800/30">
            <Filter className="w-12 h-12 text-slate-300 dark:text-slate-600 mx-auto mb-4" />
            <h3 className="text-lg font-bold text-slate-600 dark:text-slate-300">{t("attendance_policy_overrides.empty_state.title")}</h3>
            <p className="text-sm text-slate-500 mt-1">{t("attendance_policy_overrides.empty_state.description")}</p>
          </div>
        ) : (
          <div className="space-y-4">
            {filteredAssignments.map(assignment => (
              <div className="group relative border border-slate-100 dark:border-slate-800 rounded-2xl p-6 bg-slate-50/50 dark:bg-slate-800/20 hover:bg-white dark:hover:bg-slate-800 transition-all flex items-center justify-between" key={assignment.id}>
                <div className="flex items-center gap-5">
                  <div className={cn("w-12 h-12 rounded-xl flex items-center justify-center shadow-sm border", assignment.target_type === "GROUP" ? "bg-indigo-50 dark:bg-indigo-950/30 border-indigo-100 dark:border-indigo-800 text-indigo-600 dark:text-indigo-400" : "bg-teal-50 dark:bg-teal-950/30 border-teal-100 dark:border-teal-800 text-teal-600 dark:text-teal-400")}>
                    {assignment.target_type === "GROUP" ? <Building2 size={24} /> : <User size={24} />}
                  </div>
                  <div>
                    <div className="flex items-center gap-2 mb-1">
                      <span className={cn("px-2 py-0.5 text-[9px] font-bold uppercase tracking-widest rounded-md", assignment.target_type === "GROUP" ? "bg-indigo-100 text-indigo-700 dark:bg-indigo-900/50 dark:text-indigo-300" : "bg-teal-100 text-teal-700 dark:bg-teal-900/50 dark:text-teal-300")}>
                        {assignment.target_type === "GROUP" ? t("attendance_policy_overrides.labels.department") : t("attendance_policy_overrides.labels.employee")}
                      </span>
                    </div>
                    <h4 className="text-lg font-bold text-slate-800 dark:text-slate-200">
                      {getTargetName(assignment.target_type, assignment.target_id)}
                    </h4>
                  </div>
                </div>

                <div className="flex items-center gap-8 px-6">
                  <div>
                    <p className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-1">{t("common.workspace_settings.work_unit")}</p>
                    <p className="text-sm font-bold text-slate-700 dark:text-slate-300">{workUnitLabel(assignment.rules.work_unit_minutes)}</p>
                  </div>
                  <div>
                    <p className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-1">{t("common.workspace_settings.rounding_policy")}</p>
                    <p className="text-sm font-bold text-slate-700 dark:text-slate-300">{roundingLabel(assignment.rules.rounding_policy)}</p>
                  </div>
                  <div>
                    <p className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-1">{t("common.workspace_settings.early_arrival_strategy")}</p>
                    <p className="text-sm font-bold text-slate-700 dark:text-slate-300">{t(`common.workspace_settings.early_arrival_options.${assignment.rules.early_arrival_strategy || "block"}`)}</p>
                  </div>
                </div>

                <div className="flex items-center gap-2">
                  <button className="w-10 h-10 rounded-xl flex items-center justify-center text-slate-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-950/30 transition-colors" onClick={() => { deleteMutation.mutate(assignment.id); }}>
                    <Trash2 size={18} />
                  </button>
                  <button className="w-10 h-10 rounded-xl flex items-center justify-center text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors">
                    <ChevronRight size={20} />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <Sheet
        isOpen={isDrawerOpen}
        onClose={() => { setIsDrawerOpen(false); }}
        subtitle={t("attendance_policy_overrides.drawer.add_subtitle")}
        title={t("attendance_policy_overrides.drawer.add_title")}
      >
        <div className="space-y-8">
          <div className="space-y-4">
            <h3 className="text-xs font-bold text-slate-500 uppercase tracking-widest">{t("attendance_policy_overrides.drawer.step_target")}</h3>
            
            <div className="grid grid-cols-2 gap-3 mb-4">
              <button
                className={cn("p-4 rounded-2xl border transition-all flex flex-col items-center gap-2", targetType === "GROUP" ? "bg-indigo-50 dark:bg-indigo-950/30 border-indigo-200 dark:border-indigo-800 text-indigo-700 dark:text-indigo-400 shadow-inner" : "bg-white dark:bg-slate-900 border-slate-100 dark:border-slate-800 text-slate-500 hover:border-slate-300 dark:hover:border-slate-700")}
                onClick={() => { setTargetType("GROUP"); setTargetId(""); }}
              >
                <Building2 size={24} />
                <span className="text-xs font-bold uppercase tracking-widest">{t("attendance_policy_overrides.labels.department")}</span>
              </button>
              <button
                className={cn("p-4 rounded-2xl border transition-all flex flex-col items-center gap-2", targetType === "USER" ? "bg-teal-50 dark:bg-teal-950/30 border-teal-200 dark:border-teal-800 text-teal-700 dark:text-teal-400 shadow-inner" : "bg-white dark:bg-slate-900 border-slate-100 dark:border-slate-800 text-slate-500 hover:border-slate-300 dark:hover:border-slate-700")}
                onClick={() => { setTargetType("USER"); setTargetId(""); }}
              >
                <User size={24} />
                <span className="text-xs font-bold uppercase tracking-widest">{t("attendance_policy_overrides.labels.employee")}</span>
              </button>
            </div>

            <div>
              <label className={LABEL_BASE}>{t("attendance_policy_overrides.drawer.select_target_type", { type: targetType === "GROUP" ? t("attendance_policy_overrides.labels.department") : t("attendance_policy_overrides.labels.employee") })}</label>
              <Combobox
                onChange={setTargetId}
                options={targetOptions}
                placeholder={t("attendance_policy_overrides.drawer.search_placeholder", { type: targetType === "GROUP" ? t("attendance_policy_overrides.labels.department") : t("attendance_policy_overrides.labels.employee") })}
                value={targetId}
              />
            </div>
          </div>

          <div className="space-y-4">
            <h3 className="text-xs font-bold text-slate-500 uppercase tracking-widest">{t("attendance_policy_overrides.drawer.step_rules")}</h3>
            
            <div className="space-y-5 bg-slate-50/50 dark:bg-slate-800/30 p-6 rounded-[2rem] border border-slate-100 dark:border-slate-800">
              <div className="space-y-3">
                <label className={LABEL_BASE}>{t("common.workspace_settings.valid_from") || "Effective From"}</label>
                <input className="w-full px-6 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl focus:ring-4 focus:ring-primary-500/10 outline-none transition-all font-bold text-sm tracking-tight shadow-sm" onChange={e => { setValidFrom(e.target.value); }} type="date" value={validFrom} />
              </div>

              <div className="space-y-3">
                <label className={LABEL_BASE}>{t("common.workspace_settings.work_unit")}</label>
                <Combobox className="bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 h-14" onChange={(v) => { setWorkUnitMinutes(parseInt(v)); }} options={[{ value: "1", label: t("common.workspace_settings.work_unit_options.1min"), description: t("common.workspace_settings.work_unit_options.1min_desc"), icon: <Clock size={14} /> }, { value: "15", label: t("common.workspace_settings.work_unit_options.15min"), description: t("common.workspace_settings.work_unit_options.15min_desc"), icon: <Clock size={14} /> }, { value: "30", label: t("common.workspace_settings.work_unit_options.30min"), description: t("common.workspace_settings.work_unit_options.30min_desc"), icon: <Clock size={14} /> }, { value: "60", label: t("common.workspace_settings.work_unit_options.60min"), description: t("common.workspace_settings.work_unit_options.60min_desc"), icon: <Clock size={14} /> }]} value={workUnitMinutes.toString()} />
              </div>

              <div className="space-y-3">
                <label className={LABEL_BASE}>{t("common.workspace_settings.rounding_policy")}</label>
                <Combobox className="bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 h-14" onChange={setRoundingPolicy} options={[{ value: "none", label: t("common.workspace_settings.rounding_options.none"), icon: <ShieldAlert size={14} /> }, { value: "floor", label: t("common.workspace_settings.rounding_options.floor"), icon: <Calculator size={14} /> }, { value: "ceil", label: t("common.workspace_settings.rounding_options.ceil"), icon: <Calculator size={14} /> }, { value: "round", label: t("common.workspace_settings.rounding_options.round"), icon: <Calculator size={14} /> }]} value={roundingPolicy} />
              </div>

              <div className="space-y-3">
                <label className={LABEL_BASE}>{t("common.workspace_settings.early_arrival_strategy")}</label>
                <Combobox
                  className="bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 h-14"
                  onChange={(v) => { setEarlyArrivalStrategy(v as "allow" | "block" | "clamp"); }}
                  options={[
                    { value: "block", label: t("common.workspace_settings.early_arrival_options.block"), description: t("common.workspace_settings.early_arrival_options.block_desc"), icon: <ShieldAlert size={14} /> },
                    { value: "allow", label: t("common.workspace_settings.early_arrival_options.allow"), description: t("common.workspace_settings.early_arrival_options.allow_desc"), icon: <Zap size={14} /> },
                    { value: "clamp", label: t("common.workspace_settings.early_arrival_options.clamp"), description: t("common.workspace_settings.early_arrival_options.clamp_desc"), icon: <Settings2 size={14} /> }
                  ]}
                  value={earlyArrivalStrategy}
                />
                { }
              </div>
            </div>
          </div>

          <div className="pt-4 border-t border-slate-100 dark:border-slate-800">
            <Button 
              className="w-full" 
              loading={upsertMutation.isPending} 
              onClick={handleSave}
            >
              {t("attendance_policy_overrides.drawer.save_button")}
            </Button>
          </div>
        </div>
      </Sheet>
    </div>
  );
}
