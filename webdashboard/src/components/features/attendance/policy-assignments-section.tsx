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
  Filter,
  Info
} from "lucide-react";
import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { ErrorBoundary } from "@/components/common/error-boundary";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { LABEL_BASE } from "@/components/ui/form-field";
import Sheet from "@/components/ui/sheet";
import { Typography } from "@/components/ui/typography";
import { Employee, PolicyAssignmentResponse } from "@/lib/api";
import { cn } from "@/lib/utils";

import { usePolicyAssignments, StrategyType } from "./hooks/use-policy-assignments";

interface AssignmentCardProps {
  assignment: PolicyAssignmentResponse;
  getTargetName: (t: string, id: string) => string;
  onDelete: (id: string) => void;
  roundingLabel: (v: string) => string;
  workUnitLabel: (v: number) => string;
}

function AssignmentCard({ assignment, getTargetName, workUnitLabel, roundingLabel, onDelete }: AssignmentCardProps) {
  const { t } = useTranslation();
  return (
    <div className="group relative border border-slate-100 dark:border-slate-800 rounded-2xl p-6 bg-slate-50/50 dark:bg-slate-800/20 hover:bg-white dark:hover:bg-slate-800 transition-all flex items-center justify-between">
      <div className="flex items-center gap-5">
        <div className={cn("w-12 h-12 rounded-xl flex items-center justify-center shadow-sm border", assignment.target_type === "GROUP" ? "bg-primary-50 dark:bg-primary-950/30 border-primary-100 dark:border-primary-800 text-primary-600 dark:text-primary-400" : "bg-teal-50 dark:bg-teal-950/30 border-teal-100 dark:border-teal-800 text-teal-600 dark:text-teal-400")}>
          {assignment.target_type === "GROUP" ? <Building2 size={24} /> : <User size={24} />}
        </div>
        <div>
          <div className="flex items-center gap-2 mb-1">
            <span className={cn("px-2 py-0.5 text-[9px] font-bold uppercase tracking-widest rounded-md", assignment.target_type === "GROUP" ? "bg-primary-100 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300" : "bg-teal-100 text-teal-700 dark:bg-teal-900/50 dark:text-teal-300")}>
              {assignment.target_type === "GROUP" ? t("attendance_policy_overrides.labels.department") : t("attendance_policy_overrides.labels.employee")}
            </span>
          </div>
          <Typography className="text-lg font-bold text-slate-800 dark:text-slate-200" variant="h4">{getTargetName(assignment.target_type, assignment.target_id)}</Typography>
        </div>
      </div>
      <div className="flex items-center gap-8 px-6">
        <div><Typography className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-1" variant="p">{t("common.organization_settings.work_unit")}</Typography><Typography className="text-sm font-bold text-slate-700 dark:text-slate-300" variant="p">{workUnitLabel(assignment.rules.work_unit_minutes)}</Typography></div>
        <div><Typography className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-1" variant="p">{t("common.organization_settings.rounding_policy")}</Typography><Typography className="text-sm font-bold text-slate-700 dark:text-slate-300" variant="p">{roundingLabel(assignment.rules.rounding_policy)}</Typography></div>
        <div><Typography className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-1" variant="p">{t("common.organization_settings.early_arrival_strategy")}</Typography><Typography className="text-sm font-bold text-slate-700 dark:text-slate-300" variant="p">{t(`common.organization_settings.early_arrival_options.${assignment.rules.early_arrival_strategy}`)}</Typography></div>
        <div>
          <Typography className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-1" variant="p">
            {t("common.organization_settings.grace_period_minutes") || "Grace Period"}
          </Typography>
          <Typography className="text-sm font-bold text-slate-700 dark:text-slate-300" variant="p">
            {assignment.rules.grace_period_minutes != null
              ? `${assignment.rules.grace_period_minutes}m`
              : t("attendance_policy_overrides.labels.not_set", "Not Set")}
          </Typography>
        </div>
      </div>
      <div className="flex items-center gap-2">
        <Button 
          aria-label={t("common.delete", "Delete")}
          className="text-slate-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-950/30 transition-colors border-0 shadow-none" 
          icon={<Trash2 size={18} />}
          onClick={() => { onDelete(assignment.id); }}
          size="icon"
          variant="outline"
        />
        <Button 
          aria-label={t("common.show_details", "Show Details")}
          className="text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors border-0 shadow-none" 
          icon={<ChevronRight size={20} />} 
          size="icon" 
          variant="outline"
        />
      </div>
    </div>
  );
}

  // eslint-disable-next-line max-lines-per-function
function PolicyAssignmentsSectionContent() {
  const { t } = useTranslation();
  
  const {
    departments,
    employees,
    isLoadingAssignments,
    deleteMutation,
    upsertMutation,
    activeFilter,
    setActiveFilter,
    isDrawerOpen,
    setIsDrawerOpen,
    targetType,
    setTargetType,
    targetId,
    setTargetId,
    workUnitMinutes,
    setWorkUnitMinutes,
    roundingPolicy,
    setRoundingPolicy,
    earlyArrivalStrategy,
    setEarlyArrivalStrategy,
    validFrom,
    setValidFrom,
    gracePeriodMinutes,
    setGracePeriodMinutes,
    filteredAssignments,
    getTargetName,
    workUnitLabel,
    roundingLabel,
    handleSave
  } = usePolicyAssignments();

  const targetOptions = useMemo(() => 
    targetType === "GROUP" 
      ? departments.map(d => ({ value: d.id, label: d.name, icon: <Building2 className="w-4 h-4 text-primary-500" /> })) 
      : employees.map((e: Employee) => ({ value: e.id, label: e.full_name, description: e.employee_id_number, icon: <User className="w-4 h-4 text-teal-500" /> })),
    [targetType, departments, employees]
  );

  if (isLoadingAssignments) return <div className="h-40 animate-pulse bg-slate-100 dark:bg-slate-800 rounded-[2.5rem] mt-8" data-testid="policy-assignments-skeleton"></div>;

  return (
    <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden mt-8">
      <div className="p-10 border-b border-slate-50 dark:border-slate-800">
        <div className="flex items-center justify-between">
          <div>
            <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase" variant="h2">
              {t("attendance_policy_overrides.title")}
            </Typography>
            <Typography className="text-sm text-slate-500 dark:text-slate-400 mt-1" variant="p">
              {t("attendance_policy_overrides.description")}
            </Typography>
          </div>
          <div className="flex items-center gap-3">
            <div className="flex bg-slate-100 dark:bg-slate-800 p-1 rounded-xl">
              {(["ALL", "GROUP", "USER"] as const).map(f => (
                <Button 
                  className={cn("px-4 rounded-lg", activeFilter === f ? "bg-white dark:bg-slate-900 shadow-sm text-slate-800 dark:text-slate-200" : "text-slate-500 hover:text-slate-700")} 
                  key={f} 
                  onClick={() => { setActiveFilter(f); }} 
                  size="sm" 
                  variant="ghost"
                >
                  {t(`attendance_policy_overrides.filters.${f === "ALL" ? "all" : f === "GROUP" ? "depts" : "users"}`)}
                </Button>
              ))}
            </div>
            <Button icon={<Plus size={18} />} onClick={() => { setIsDrawerOpen(true); }} variant="primary">
              {t("attendance_policy_overrides.add_button")}
            </Button>
          </div>
        </div>
      </div>
      <div className="p-10">
        {filteredAssignments.length === 0 ? (
          <div className="text-center py-12 border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-3xl bg-slate-50/50 dark:bg-slate-800/30">
            <Filter className="w-12 h-12 text-slate-300 dark:text-slate-600 mx-auto mb-4" />
            <Typography className="text-lg font-bold text-slate-600 dark:text-slate-300" variant="h3">
              {t("attendance_policy_overrides.empty_state.title")}
            </Typography>
            <Typography className="text-sm text-slate-500 mt-1" variant="p">
              {t("attendance_policy_overrides.empty_state.description")}
            </Typography>
          </div>
        ) : (
          <div className="space-y-4">
            {filteredAssignments.map(a => (
              <AssignmentCard 
                assignment={a} 
                getTargetName={getTargetName} 
                key={a.id} 
                onDelete={(id) => { deleteMutation.mutate(id); }} 
                roundingLabel={roundingLabel} 
                workUnitLabel={workUnitLabel} 
              />
            ))}
          </div>
        )}
      </div>
      <Sheet isOpen={isDrawerOpen} onClose={() => { setIsDrawerOpen(false); }} subtitle={t("attendance_policy_overrides.drawer.add_subtitle")} title={t("attendance_policy_overrides.drawer.add_title")}>
        <div className="space-y-8">
          <div className="space-y-4">
            <Typography className="text-xs font-bold text-slate-500 uppercase tracking-widest" variant="h3">
              {t("attendance_policy_overrides.drawer.step_target")}
            </Typography>
            <div className="grid grid-cols-2 gap-3 mb-4">
              {(([{ type: "GROUP", icon: <Building2 size={24} />, label: "department" }, { type: "USER", icon: <User size={24} />, label: "employee" }] as const).map(t_ => (
                <Button 
                  className={cn("h-auto py-4 rounded-2xl flex-col gap-2", targetType === t_.type ? "bg-primary-50 dark:bg-primary-950/30 border-primary-200 dark:border-primary-800 text-primary-700 dark:text-primary-400 shadow-inner hover:bg-primary-50" : "text-slate-500 hover:border-slate-300 dark:hover:border-slate-700")} 
                  key={t_.type} 
                  onClick={() => { setTargetType(t_.type); setTargetId(""); }} 
                  variant="outline"
                >
                  {t_.icon}
                  <span className="text-xs font-bold uppercase tracking-widest">{t(`attendance_policy_overrides.labels.${t_.label}`)}</span>
                </Button>
              )))}
            </div>
            <div>
              <label className={LABEL_BASE}>
                {t("attendance_policy_overrides.drawer.select_target_type", { type: t(`attendance_policy_overrides.labels.${targetType === "GROUP" ? "department" : "employee"}`) })}
              </label>
              <Combobox 
                onChange={setTargetId} 
                options={targetOptions} 
                placeholder={t("attendance_policy_overrides.drawer.search_placeholder", { type: t(`attendance_policy_overrides.labels.${targetType === "GROUP" ? "department" : "employee"}`) })} 
                value={targetId} 
              />
            </div>
          </div>
          <div className="space-y-4">
            <Typography className="text-xs font-bold text-slate-500 uppercase tracking-widest" variant="h3">
              {t("attendance_policy_overrides.drawer.step_rules")}
            </Typography>
            <div className="space-y-5 bg-slate-50/50 dark:bg-slate-800/30 p-6 rounded-[2rem] border border-slate-100 dark:border-slate-800">
              <div className="text-xs bg-primary-500/10 border border-primary-500/20 rounded-xl p-3 flex items-start gap-2.5 text-primary-700 dark:text-primary-400 mb-2">
                <Info className="mt-0.5 shrink-0" size={14} />
                <span>{t("attendance_policy_overrides.drawer.override_notice")}</span>
              </div>
              <div className="space-y-3">
                <label className={LABEL_BASE}>{t("common.organization_settings.valid_from") || "Effective From"}</label>
                <input 
                  className="w-full px-6 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl focus:ring-4 focus:ring-primary-500/10 outline-none transition-all font-bold text-sm tracking-tight shadow-sm" 
                  onChange={e => { setValidFrom(e.target.value); }} 
                  type="date" 
                  value={validFrom} 
                />
              </div>
              <div className="space-y-3">
                <label className={LABEL_BASE}>{t("common.organization_settings.work_unit")}</label>
                <Combobox 
                  className="bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 h-14" 
                  onChange={(v) => { setWorkUnitMinutes(parseInt(v)); }} 
                  options={["1", "15", "30", "60"].map(v => ({ value: v, label: t(`common.organization_settings.work_unit_options.${v}min`), description: t(`common.organization_settings.work_unit_options.${v}min_desc`), icon: <Clock size={14} />}))}
                  value={workUnitMinutes.toString()} 
                />
              </div>
              <div className="space-y-3">
                <label className={LABEL_BASE}>{t("common.organization_settings.rounding_policy")}</label>
                <Combobox 
                  className="bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 h-14" 
                  onChange={(v) => { setRoundingPolicy(v as "none" | "floor" | "round" | "ceil"); }}
                  options={["none", "floor", "ceil", "round"].map(v => ({ value: v, label: t(`common.organization_settings.rounding_options.${v}`), icon: v === "none" ? <ShieldAlert size={14} /> : <Calculator size={14} />}))}
                  value={roundingPolicy} 
                />
              </div>
              <div className="space-y-3">
                <label className={LABEL_BASE}>{t("common.organization_settings.early_arrival_strategy")}</label>
                <Combobox 
                  className="bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 h-14" 
                  onChange={(v) => { setEarlyArrivalStrategy(v as StrategyType); }} 
                  options={["block", "allow", "clamp"].map(v => ({ value: v, label: t(`common.organization_settings.early_arrival_options.${v}`), description: t(`common.organization_settings.early_arrival_options.${v}_desc`), icon: v === "block" ? <ShieldAlert size={14} /> : v === "allow" ? <Zap size={14} /> : <Settings2 size={14} />}))}
                  value={earlyArrivalStrategy} 
                />
              </div>
              <div className="space-y-3">
                <label className={LABEL_BASE}>
                  {t("common.organization_settings.grace_period_minutes") || "Late Grace Period (minutes)"}
                </label>
                <input 
                  className="w-full px-6 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl focus:ring-4 focus:ring-primary-500/10 outline-none transition-all font-bold text-sm tracking-tight shadow-sm" 
                  max={1440}
                  min={0}
                  onChange={e => {
                    const val = parseInt(e.target.value);
                    setGracePeriodMinutes(isNaN(val) ? undefined : val);
                  }} 
                  placeholder={t("attendance_policy_overrides.drawer.grace_period_placeholder", "Using default organization value")}
                  type="number" 
                  value={gracePeriodMinutes ?? ""} 
                />
              </div>
            </div>
          </div>
          <div className="pt-4 border-t border-slate-100 dark:border-slate-800">
            <Button className="w-full" loading={upsertMutation.isPending} onClick={handleSave} variant="primary">
              {t("attendance_policy_overrides.drawer.save_button")}
            </Button>
          </div>
        </div>
      </Sheet>
    </div>
  );
}

export function PolicyAssignmentsSection() {
  return (
    <ErrorBoundary variant="compact">
      <PolicyAssignmentsSectionContent   />
    </ErrorBoundary>
  );
}
