import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Clock, Calculator, ShieldAlert, Settings2, LayoutGrid, Zap, RefreshCw, AlertTriangle, Plus, History, CheckCircle2 } from "lucide-react";
import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import { PolicyAssignmentsSection } from "@/components/features/attendance/policy-assignments-section";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { LABEL_BASE } from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { fetchWorkspace, updateWorkspace, getCurrentWorkspaceId, Workspace, reprocessAllAttendance, fetchGenericJobStatus, fetchDefaultAttendanceRules, updateDefaultAttendanceRules, fetchAttendancePolicyVersions, AttendanceRules, AttendancePolicyVersion, UpdateAttendanceRulesRequest } from "@/lib/api";
import { cn } from "@/lib/utils";
import AssignmentRulesPage from "@/pages/assignment-rules-page";
import WorkingPatternsPage from "@/pages/working-patterns-page";

export default function TimeSettings() {
  const wsId = getCurrentWorkspaceId();

  const { data: workspace, isLoading: isLoadingWs } = useQuery({
    queryKey: ["workspace", wsId],
    queryFn: () => fetchWorkspace(wsId!),
    enabled: !!wsId,
  });

  const { data: rules, isLoading: isLoadingRules } = useQuery({
    queryKey: ["attendance-rules", wsId],
    queryFn: () => fetchDefaultAttendanceRules(wsId!),
    enabled: !!wsId,
  });

  const { data: versions = [], isLoading: isLoadingVersions } = useQuery({
    queryKey: ["attendance-policy-versions", wsId],
    queryFn: () => fetchAttendancePolicyVersions(wsId!),
    enabled: !!wsId,
  });

  if (isLoadingWs || isLoadingRules || isLoadingVersions) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
      </div>
    );
  }

  if (!workspace || !rules) return null;

  return <TimeSettingsContent key={`${workspace.id}-${versions.length}`} rules={rules} versions={versions} workspace={workspace} />;
}

type TabType = "general" | "patterns" | "rules";

function formatDate(iso: string) {
  try { return new Date(iso).toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" }); } catch { return iso; }
}

function workUnitLabel(v: number, t: (k: string) => string) {
  if (v === 1) return t("common.workspace_settings.work_unit_options.1min");
  if (v === 15) return t("common.workspace_settings.work_unit_options.15min");
  if (v === 30) return t("common.workspace_settings.work_unit_options.30min");
  if (v === 60) return t("common.workspace_settings.work_unit_options.60min");
  return `${v} min`;
}

function roundingLabel(v: string, t: (k: string) => string) {
  const map: Record<string, string> = { 
    none: t("common.workspace_settings.rounding_options.none"), 
    floor: t("common.workspace_settings.rounding_options.floor"), 
    ceil: t("common.workspace_settings.rounding_options.ceil"), 
    round: t("common.workspace_settings.rounding_options.round") 
  };
  return map[v] || v;
}

// --- Sub-components to reduce max-lines-per-function ---

function AttendanceVersionTimeline({ versions, t }: { versions: AttendancePolicyVersion[], t: (k: string) => string }) {
  return (
    <div className="bg-white dark:bg-slate-900 p-10 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm space-y-8">
      <div className="flex items-center justify-between border-b border-slate-50 dark:border-slate-800 pb-6">
        <div className="flex items-center gap-3"><History className="w-6 h-6 text-indigo-500" /><h2 className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase">{t("common.workspace_settings.attendance_config")}</h2></div>
        <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">{versions.length} {versions.length === 1 ? "VERSION" : "VERSIONS"}</span>
      </div>

      {versions.length === 0 ? (
        <div className="py-16 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[2rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center">
          <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-5 border border-slate-100 dark:border-slate-800"><Calculator className="text-slate-200 dark:text-slate-700" size={36} /></div>
          <h3 className="text-lg font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight">{t("common.workspace_settings.no_versions") || "No configuration yet"}</h3>
          <p className="text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-widest mt-1 opacity-50 italic">{t("common.workspace_settings.no_versions_desc") || "Create the first version to get started"}</p>
        </div>
      ) : (
        <div className="space-y-4">
          {versions.map((v, index) => {
            const isCurrent = index === 0 && !v.valid_to;
            return (
              <div className={cn(
                "relative p-6 rounded-2xl border transition-all group",
                isCurrent
                  ? "bg-primary-50/50 dark:bg-primary-950/30 border-primary-200 dark:border-primary-800 shadow-md"
                  : "bg-slate-50/50 dark:bg-slate-800/30 border-slate-100 dark:border-slate-800 opacity-70"
              )} key={v.id}>
                <div className="flex items-start justify-between gap-4">
                  <div className="flex items-center gap-4">
                    <div className={cn(
                      "w-12 h-12 rounded-xl flex items-center justify-center flex-shrink-0",
                      isCurrent ? "bg-primary-600 text-white shadow-lg" : "bg-white dark:bg-slate-900 text-slate-400 dark:text-slate-600 border border-slate-100 dark:border-slate-800"
                    )}>
                      {isCurrent ? <CheckCircle2 size={24} /> : <Clock size={24} />}
                    </div>
                    <div>
                      <div className="flex items-center gap-3 mb-1">
                        {!!isCurrent && <span className="px-2.5 py-0.5 bg-primary-600 text-white text-[9px] font-bold uppercase tracking-widest rounded-lg shadow-sm">{t("status_active_caps")}</span>}
                        {!!v.valid_to && <span className="px-2.5 py-0.5 bg-slate-200 dark:bg-slate-700 text-slate-600 dark:text-slate-300 text-[9px] font-bold uppercase tracking-widest rounded-lg">{t("status_expired")}</span>}
                      </div>
                      <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                        {formatDate(v.valid_from)}{v.valid_to ? ` → ${formatDate(v.valid_to)}` : " → Present"}
                      </p>
                    </div>
                  </div>
                </div>

                <div className="mt-4 grid grid-cols-2 gap-4">
                  <div className="p-4 bg-white dark:bg-slate-900 rounded-xl border border-slate-100 dark:border-slate-800">
                    <p className="text-[9px] font-bold text-slate-400 uppercase tracking-widest mb-1">{t("common.workspace_settings.work_unit")}</p>
                    <p className={cn("text-lg font-bold tracking-tight", isCurrent ? "text-primary-600 dark:text-primary-400" : "text-slate-600 dark:text-slate-300")}>{workUnitLabel(v.work_unit_minutes, t)}</p>
                  </div>
                  <div className="p-4 bg-white dark:bg-slate-900 rounded-xl border border-slate-100 dark:border-slate-800">
                    <p className="text-[9px] font-bold text-slate-400 uppercase tracking-widest mb-1">{t("common.workspace_settings.rounding_policy")}</p>
                    <p className={cn("text-lg font-bold tracking-tight", isCurrent ? "text-primary-600 dark:text-primary-400" : "text-slate-600 dark:text-slate-300")}>{roundingLabel(v.rounding_policy, t)}</p>
                  </div>
                  <div className="col-span-2 p-4 bg-white dark:bg-slate-900 rounded-xl border border-slate-100 dark:border-slate-800 flex items-center justify-between">
                    <div>
                        <p className="text-[9px] font-bold text-slate-400 uppercase tracking-widest mb-1">{t("common.workspace_settings.early_arrival_strategy")}</p>
                        <p className={cn("text-sm font-bold tracking-tight", isCurrent ? "text-amber-600 dark:text-amber-400" : "text-slate-600 dark:text-slate-300")}>
                          {t(`common.workspace_settings.early_arrival_options.${v.early_arrival_strategy || "block"}`)}
                        </p>
                    </div>
                    <ShieldAlert className={cn("w-5 h-5", isCurrent ? "text-amber-500/50" : "text-slate-400/50")} />
                  </div>
                </div>
                <p className="mt-3 text-[9px] font-medium text-slate-400 dark:text-slate-500 italic">Created {formatDate(v.created_at)}</p>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

function BulkReprocessSection({ t, reprocessStart, setReprocessStart, reprocessEnd, setReprocessEnd, setShowReprocessConfirm }: any) {
  return (
    <div className="bg-white dark:bg-slate-900 p-10 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm space-y-10 mt-8">
      <div className="flex items-center justify-between border-b border-slate-50 dark:border-slate-800 pb-6">
        <div className="flex items-center gap-3">
          <RefreshCw className="w-6 h-6 text-orange-500" />
          <h2 className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase">
            {t("reports.reprocess.bulk_tooltip") || "Reprocess Attendance"}
          </h2>
        </div>
      </div>
      <p className="text-sm text-slate-500 dark:text-slate-400">
        {t("reports.reprocess.bulk_confirm_message") || "Recalculate attendance for all employees in the selected date range."}
      </p>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-10">
        <div className="space-y-3">
          <label className={LABEL_BASE}>{t("common.startDate") || "Start Date"}</label>
          <input className="w-full px-4 py-3 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm font-bold h-14" onChange={(e) => { setReprocessStart(e.target.value); }} type="date" value={reprocessStart} />
        </div>
        <div className="space-y-3">
          <label className={LABEL_BASE}>{t("common.endDate") || "End Date"}</label>
          <input className="w-full px-4 py-3 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm font-bold h-14" onChange={(e) => { setReprocessEnd(e.target.value); }} type="date" value={reprocessEnd} />
        </div>
        <div className="space-y-3 md:col-span-2 flex justify-end">
          <Button className="text-orange-600 border-orange-200 hover:bg-orange-50 dark:border-orange-900/50 dark:hover:bg-orange-900/20" icon={<RefreshCw size={18} />} onClick={() => { setShowReprocessConfirm(true); }} variant="outline">
            {(t("reports.reprocess.bulk_tooltip") || "REPROCESS DATA").toUpperCase()}
          </Button>
        </div>
      </div>
    </div>
  );
}

function TimeSettingsContent({ workspace, rules, versions }: { workspace: Workspace; rules: AttendanceRules; versions: AttendancePolicyVersion[] }) {
  const { t } = useTranslation();
  const wsId = workspace.id;
  const queryClient = useQueryClient();
  const toast = useToast();

  const [activeTab, setActiveTab] = useState<TabType>("general");

  const [workUnit, setWorkUnit] = useState(rules.work_unit_minutes ?? 15);
  const [roundingPolicy, setRoundingPolicy] = useState(rules.rounding_policy || "none");
  const [earlyArrivalStrategy, setEarlyArrivalStrategy] = useState(rules.early_arrival_strategy || "block");
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newValidFrom, setNewValidFrom] = useState(() => new Date().toISOString().split("T")[0]);

  // --- Bulk Reprocess State ---
  const [reprocessStart, setReprocessStart] = useState(() => {
    const d = new Date();
    d.setDate(d.getDate() - 30);
    return d.toISOString().split("T")[0];
  });
  const [reprocessEnd, setReprocessEnd] = useState(() => {
    return new Date().toISOString().split("T")[0];
  });
  const [showReprocessConfirm, setShowReprocessConfirm] = useState(false);
  const [pollingJobId, setPollingJobId] = useState<string | null>(null);

  const jobStatusQuery = useQuery({
    queryKey: ["generic-job-status", wsId, pollingJobId],
    queryFn: () => fetchGenericJobStatus(wsId, pollingJobId!),
    enabled: !!pollingJobId,
    refetchInterval: (query: { state: { data?: { status: string } } }) => {
      const data = query.state.data;
      return data && (data.status === "completed" || data.status === "failed") ? false : 3000;
    },
  });

  useEffect(() => {
    if (jobStatusQuery.data) {
      if (jobStatusQuery.data.status === "completed") {
        setTimeout(() => {
          toast.success(t("common.success") + " - Reprocessing finished");
          void queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
          void queryClient.invalidateQueries({ queryKey: ["attendance-logs"] });
          setShowReprocessConfirm(false);
          setPollingJobId(null);
        }, 0);
      } else if (jobStatusQuery.data.status === "failed") {
        setTimeout(() => {
          toast.error("Process failed: " + (jobStatusQuery.data.error ?? "Unknown error"));
          setShowReprocessConfirm(false);
          setPollingJobId(null);
        }, 0);
      }
    }
  }, [jobStatusQuery.data, queryClient, t, toast]);

  const reprocessMutation = useMutation({
    mutationFn: () => reprocessAllAttendance(wsId, reprocessStart, reprocessEnd),
    onSuccess: (data) => {
      setPollingJobId(data.job_id);
    },
    onError: (e: Error) => {
      toast.error(e.message || t("common.error"));
    }
  });

  const handleReprocessSubmit = () => {
    reprocessMutation.mutate();
  };

  const updateRulesMutation = useMutation({
    mutationFn: (data: UpdateAttendanceRulesRequest) => updateDefaultAttendanceRules(wsId, data),
    onSuccess: () => {
      toast.success(t("common.success"));
      void queryClient.invalidateQueries({ queryKey: ["attendance-rules", wsId] });
      void queryClient.invalidateQueries({ queryKey: ["attendance-policy-versions", wsId] });
      setShowCreateModal(false);
    },
  });

  function handleSaveNewVersion() {
    updateRulesMutation.mutate({
      work_unit_minutes: workUnit,
      rounding_policy: roundingPolicy,
      early_arrival_strategy: earlyArrivalStrategy,
      valid_from: newValidFrom,
    });
  }

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50"><Clock size={24} /></div>
          <div>
            <h1 className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200 leading-tight">{t("common.settings.categories.time")}</h1>
            <p className="text-slate-500 dark:text-slate-400 mt-1 font-medium">{t("common.settings.categories.time_desc")}</p>
          </div>
        </div>
        {activeTab === "general" && (
          <Button icon={<Plus size={18} />} onClick={() => { setShowCreateModal(true); setNewValidFrom(new Date().toISOString().split("T")[0]); }}>{t("common.workspace_settings.new_version") || "NEW VERSION"}</Button>
        )}
      </div>

      <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
        <button className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'general' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")} onClick={() => { setActiveTab("general"); }}><Settings2 size={14} /> {t("common.settings.tabs.general")}</button>
        <button className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'patterns' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")} onClick={() => { setActiveTab("patterns"); }}><LayoutGrid size={14} /> {t("common.settings.tabs.patterns")}</button>
        <button className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'rules' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")} onClick={() => { setActiveTab("rules"); }}><Zap size={14} /> {t("common.settings.tabs.rules")}</button>
      </div>

      <div className="animate-in slide-in-from-bottom-4 duration-500">
        {activeTab === "general" && (
          <>
            <AttendanceVersionTimeline t={t} versions={versions} />
            <PolicyAssignmentsSection wsId={wsId} />
            <BulkReprocessSection reprocessEnd={reprocessEnd} reprocessStart={reprocessStart} setReprocessEnd={setReprocessEnd} setReprocessStart={setReprocessStart} setShowReprocessConfirm={setShowReprocessConfirm} t={t} />

            <Modal icon={<Plus size={24} />} isOpen={showCreateModal} maxWidth="xl" onClose={() => { setShowCreateModal(false); }} subtitle={t("common.workspace_settings.new_version_desc") || "Create a new configuration version with an effective date"} title={t("common.workspace_settings.new_version") || "New Attendance Config"}>
              <div className="space-y-8 pt-4 animate-in fade-in duration-500">
                <div className="space-y-3 px-1">
                  <label className={LABEL_BASE}>{t("common.workspace_settings.valid_from") || "Effective From"}</label>
                  <input className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight shadow-sm" onChange={e => { setNewValidFrom(e.target.value); }} type="date" value={newValidFrom} />
                  <p className="text-[9px] text-slate-500 dark:text-slate-400 font-bold uppercase italic leading-relaxed px-1 opacity-60">{t("common.workspace_settings.valid_from_note") || "Payroll calculations will use this configuration from this date forward."}</p>
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                  <div className="space-y-3 px-1">
                    <label className={LABEL_BASE}>{t("common.workspace_settings.work_unit")}</label>
                    <Combobox className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" onChange={(v) => { setWorkUnit(parseInt(v)); }} options={[{ value: "1", label: t("common.workspace_settings.work_unit_options.1min"), description: t("common.workspace_settings.work_unit_options.1min_desc"), icon: <Clock size={14} /> }, { value: "15", label: t("common.workspace_settings.work_unit_options.15min"), description: t("common.workspace_settings.work_unit_options.15min_desc"), icon: <Clock size={14} /> }, { value: "30", label: t("common.workspace_settings.work_unit_options.30min"), description: t("common.workspace_settings.work_unit_options.30min_desc"), icon: <Clock size={14} /> }, { value: "60", label: t("common.workspace_settings.work_unit_options.60min"), description: t("common.workspace_settings.work_unit_options.60min_desc"), icon: <Clock size={14} /> }]} value={workUnit.toString() || "15"} />
                  </div>
                  <div className="space-y-3 px-1">
                    <label className={LABEL_BASE}>{t("common.workspace_settings.rounding_policy")}</label>
                    <Combobox className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" onChange={setRoundingPolicy} options={[{ value: "none", label: t("common.workspace_settings.rounding_options.none"), icon: <ShieldAlert size={14} /> }, { value: "floor", label: t("common.workspace_settings.rounding_options.floor"), icon: <Calculator size={14} /> }, { value: "ceil", label: t("common.workspace_settings.rounding_options.ceil"), icon: <Calculator size={14} /> }, { value: "round", label: t("common.workspace_settings.rounding_options.round"), icon: <Calculator size={14} /> }]} value={roundingPolicy || "none"} />
                  </div>
                </div>
                <div className="space-y-3 px-1">
                  <label className={LABEL_BASE}>{t("common.workspace_settings.early_arrival_strategy")}</label>
                  <Combobox
                    className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14"
                    onChange={(v) => { setEarlyArrivalStrategy(v as "block" | "allow" | "clamp"); }}
 
                    options={[
                      { value: "block", label: t("common.workspace_settings.early_arrival_options.block"), description: t("common.workspace_settings.early_arrival_options.block_desc"), icon: <ShieldAlert size={14} /> },
                      { value: "allow", label: t("common.workspace_settings.early_arrival_options.allow"), description: t("common.workspace_settings.early_arrival_options.allow_desc"), icon: <Zap size={14} /> },
                      { value: "clamp", label: t("common.workspace_settings.early_arrival_options.clamp"), description: t("common.workspace_settings.early_arrival_options.clamp_desc"), icon: <Settings2 size={14} /> }
                    ]}
                    value={earlyArrivalStrategy}
                  />
                </div>
                <div className="pt-8 border-t border-slate-50 dark:border-slate-800 grid grid-cols-2 gap-6">
                  <Button onClick={() => { setShowCreateModal(false); }} size="lg" variant="outline">{t("common.cancel").toUpperCase()}</Button>
                  <Button loading={updateRulesMutation.isPending} onClick={handleSaveNewVersion} size="lg" variant="secondary">{(t("common.workspace_settings.save_config") || "SAVE").toUpperCase()}</Button>
                </div>
              </div>
            </Modal>
          </>
        )}

        {activeTab === "patterns" && (
          <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm p-4"><WorkingPatternsPage /></div>
        )}

        {activeTab === "rules" && (
          <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm p-4"><AssignmentRulesPage /></div>
        )}

      </div>

      <Modal
        icon={<AlertTriangle size={24} />}
        isOpen={showReprocessConfirm}
        maxWidth="md"
        onClose={() => !reprocessMutation.isPending && !pollingJobId && setShowReprocessConfirm(false)}
        title={t("reports.reprocess.bulk_confirm_title") || "Confirm Reprocessing"}
      >
        <div className="space-y-6 pt-2 animate-in fade-in duration-500">
          <div className="p-4 bg-orange-50 dark:bg-orange-950 rounded-2xl border border-orange-200 dark:border-orange-800">
            <p className="text-sm font-bold text-orange-800 dark:text-orange-300">
              Are you sure you want to reprocess attendance from {reprocessStart} to {reprocessEnd}?
            </p>
            <p className="text-xs text-orange-700 dark:text-orange-400 mt-2">
              {t("reports.reprocess.bulk_confirm_message") || "This action will recalculate work hours, overtime, and late/early statuses for ALL employees in the selected date range. This process cannot be undone."}
            </p>
          </div>
          
          {pollingJobId && jobStatusQuery.data ? (
            <div className="space-y-3 pt-4 border-t border-slate-100 dark:border-slate-800">
              <div className="flex justify-between text-xs font-bold text-slate-500 uppercase tracking-widest">
                <span>{t("common.working")}</span>
                <span>{Math.round(jobStatusQuery.data.progress.percentage).toString()}%</span>
              </div>
              <div className="h-3 w-full bg-slate-100 dark:bg-slate-800 rounded-full overflow-hidden">
                <div 
                  className="h-full bg-orange-500 transition-all duration-500 ease-out"
                  style={{ width: `${Math.max(5, jobStatusQuery.data.progress.percentage).toString()}%` }}
                />
              </div>
              <p className="text-center text-xs text-slate-400 mt-2 font-medium">
                {t("common.workspace_settings.please_wait_minutes")}
              </p>
            </div>
          ) : (
            <div className="flex gap-4 border-t border-slate-100 dark:border-slate-800 pt-4">
              <button
                className="flex-1 px-6 py-3.5 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95"
                disabled={reprocessMutation.isPending}
                onClick={() => { setShowReprocessConfirm(false); }}
              >
                {t("common.cancel")}
              </button>
              <button
                className="flex-[1.5] bg-orange-600 hover:bg-orange-700 text-white font-bold px-6 py-3.5 rounded-2xl flex items-center justify-center gap-3 transition-all active:scale-95 disabled:opacity-50 shadow-xl text-xs uppercase tracking-widest"
                disabled={reprocessMutation.isPending || !reprocessStart || !reprocessEnd}
                onClick={handleReprocessSubmit}
              >
                {reprocessMutation.isPending ? (
                    <RefreshCw className="w-4 h-4 animate-spin" />
                ) : (
                    <RefreshCw size={18} />
                )}
                {(t("reports.reprocess.confirm") || "CONFIRM").toUpperCase()}
              </button>
            </div>
          )}
        </div>
      </Modal>

    </div>
  );
}
