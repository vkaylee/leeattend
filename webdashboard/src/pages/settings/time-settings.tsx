import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Clock, Settings2, LayoutGrid, Zap, Plus } from "lucide-react";
import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import { TFunction } from "i18next";

import { PolicyAssignmentsSection } from "@/components/features/attendance/policy-assignments-section";
import { useToast } from "@/components/ui/toast";
import {
  fetchWorkspace,
  getCurrentWorkspaceId,
  Workspace,
  reprocessAllAttendance,
  fetchGenericJobStatus,
  fetchDefaultAttendanceRules,
  updateDefaultAttendanceRules,
  fetchAttendancePolicyVersions,
  AttendanceRules,
  AttendancePolicyVersion,
  UpdateAttendanceRulesRequest,
} from "@/lib/api";
import { cn } from "@/lib/utils";
import AssignmentRulesPage from "@/pages/assignment-rules-page";
import WorkingPatternsPage from "@/pages/working-patterns-page";

import { BulkReprocessSection } from "./time-settings/bulk-reprocess";
import { CreateVersionModal } from "./time-settings/create-version-modal";
import { ReprocessModal } from "./time-settings/reprocess-modal";
import { AttendanceVersionTimeline } from "./time-settings/timeline";

export default function TimeSettings() {
  const wsId = getCurrentWorkspaceId();

  const { data: workspace, isLoading: isLoadingWs } = useQuery({
    queryKey: ["workspace", wsId],
    queryFn: () => (wsId ? fetchWorkspace(wsId) : Promise.reject(new Error("No workspace ID"))),
    enabled: !!wsId,
  });

  const { data: rules, isLoading: isLoadingRules } = useQuery({
    queryKey: ["attendance-rules", wsId],
    queryFn: () => (wsId ? fetchDefaultAttendanceRules(wsId) : Promise.reject(new Error("No workspace ID"))),
    enabled: !!wsId,
  });

  const { data: versions = [], isLoading: isLoadingVersions } = useQuery({
    queryKey: ["attendance-policy-versions", wsId],
    queryFn: () => (wsId ? fetchAttendancePolicyVersions(wsId) : Promise.reject(new Error("No workspace ID"))),
    enabled: !!wsId,
  });

  if (isLoadingWs || isLoadingRules || isLoadingVersions) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
      </div>
    );
  }

  if (!workspace || !rules) {
    return null;
  }

  return (
    <TimeSettingsContent
      key={`${workspace.id}-${versions.length}`}
      rules={rules}
      versions={versions}
      workspace={workspace}
    />
  );
}

type TabType = "general" | "patterns" | "rules";

interface TimeSettingsContentProps {
  rules: AttendanceRules;
  versions: AttendancePolicyVersion[];
  workspace: Workspace;
}

function TimeSettingsContent({
  rules,
  versions,
  workspace,
}: TimeSettingsContentProps) {
  const { t } = useTranslation();
  const wsId = workspace.id;
  const [activeTab, setActiveTab] = useState<TabType>("general");

  const [workUnit, setWorkUnit] = useState(rules.work_unit_minutes);
  const [roundingPolicy, setRoundingPolicy] = useState(rules.rounding_policy);
  const [earlyArrivalStrategy, setEarlyArrivalStrategy] = useState<"allow" | "block" | "clamp">(
    rules.early_arrival_strategy
  );
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newValidFrom, setNewValidFrom] = useState(() => new Date().toISOString().split("T")[0]);

  const {
    jobStatus,
    pollingJobId,
    reprocessEnd,
    reprocessMutation,
    reprocessStart,
    setReprocessEnd,
    setReprocessStart,
    setShowReprocessConfirm,
    showReprocessConfirm,
  } = useBulkReprocess(wsId, t);

  const { handleSaveNewVersion, isPending } = useTimeSettingsRules(wsId, t, {
    earlyArrivalStrategy,
    roundingPolicy,
    valid_from: newValidFrom,
    work_unit_minutes: workUnit,
    onSuccess: () => {
      setShowCreateModal(false);
    },
  });

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <PageHeader
        activeTab={activeTab}
        onNewVersion={() => {
          setShowCreateModal(true);
          setNewValidFrom(new Date().toISOString().split("T")[0]);
        }}
        t={t}
      />
      <TabNavigation activeTab={activeTab} setActiveTab={setActiveTab} t={t} />
      <div className="animate-in slide-in-from-bottom-4 duration-500">
        <TabContent
          activeTab={activeTab}
          earlyArrivalStrategy={earlyArrivalStrategy}
          handleSaveNewVersion={handleSaveNewVersion}
          isPending={isPending}
          newValidFrom={newValidFrom}
          reprocessEnd={reprocessEnd}
          reprocessEndSetter={setReprocessEnd}
          reprocessStart={reprocessStart}
          reprocessStartSetter={setReprocessStart}
          roundingPolicy={roundingPolicy}
          setEarlyArrivalStrategy={setEarlyArrivalStrategy}
          setNewValidFrom={setNewValidFrom}
          setRoundingPolicy={setRoundingPolicy}
          setShowCreateModal={setShowCreateModal}
          setShowReprocessConfirm={setShowReprocessConfirm}
          setWorkUnit={setWorkUnit}
          showCreateModal={showCreateModal}
          t={t}
          versions={versions}
          workUnit={workUnit}
          wsId={wsId}
        />
      </div>
      <ReprocessModal
        isOpen={showReprocessConfirm}
        isPending={reprocessMutation.isPending}
        jobStatus={jobStatus}
        onClose={() => {
          if (!reprocessMutation.isPending && !pollingJobId) {
            setShowReprocessConfirm(false);
          }
        }}
        onSubmit={() => {
          reprocessMutation.mutate();
        }}
        pollingJobId={pollingJobId}
        reprocessEnd={reprocessEnd}
        reprocessStart={reprocessStart}
        t={t}
      />
    </div>
  );
}

function useTimeSettingsRules(
  wsId: string,
  t: TFunction,
  params: {
    earlyArrivalStrategy: "allow" | "block" | "clamp";
    roundingPolicy: string;
    valid_from: string;
    work_unit_minutes: number;
    onSuccess: () => void;
  }
) {
  const queryClient = useQueryClient();
  const toast = useToast();

  const updateRulesMutation = useMutation({
    mutationFn: (data: UpdateAttendanceRulesRequest) => updateDefaultAttendanceRules(wsId, data),
    onSuccess: () => {
      toast.success(t("common.success"));
      void queryClient.invalidateQueries({ queryKey: ["attendance-rules", wsId] });
      void queryClient.invalidateQueries({ queryKey: ["attendance-policy-versions", wsId] });
      params.onSuccess();
    },
  });

  const handleSaveNewVersion = () => {
    updateRulesMutation.mutate({
      early_arrival_strategy: params.earlyArrivalStrategy,
      rounding_policy: params.roundingPolicy,
      valid_from: params.valid_from,
      work_unit_minutes: params.work_unit_minutes,
    });
  };

  return { handleSaveNewVersion, isPending: updateRulesMutation.isPending };
}

function useBulkReprocess(wsId: string, t: TFunction) {
  const queryClient = useQueryClient();
  const toast = useToast();

  const [reprocessStart, setReprocessStart] = useState(() => {
    const d = new Date();
    d.setDate(d.getDate() - 30);
    return d.toISOString().split("T")[0];
  });
  const [reprocessEnd, setReprocessEnd] = useState(() => new Date().toISOString().split("T")[0]);
  const [showReprocessConfirm, setShowReprocessConfirm] = useState(false);
  const [pollingJobId, setPollingJobId] = useState<string | null>(null);

  const jobStatusQuery = useQuery({
    queryKey: ["generic-job-status", wsId, pollingJobId],
    queryFn: () => {
      if (!pollingJobId) return Promise.reject(new Error("No job ID"));
      return fetchGenericJobStatus(wsId, pollingJobId);
    },
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
    },
  });

  return {
    jobStatus: jobStatusQuery.data,
    pollingJobId,
    reprocessEnd,
    reprocessMutation,
    reprocessStart,
    setReprocessEnd,
    setReprocessStart,
    setShowReprocessConfirm,
    showReprocessConfirm,
  };
}

function PageHeader({ activeTab, onNewVersion, t }: { activeTab: TabType; onNewVersion: () => void; t: TFunction }) {
  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
      <div className="flex items-center gap-4">
        <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50">
          <Clock size={24} />
        </div>
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200 leading-tight">
            {t("common.settings.categories.time")}
          </h1>
          <p className="text-slate-500 dark:text-slate-400 mt-1 font-medium">{t("common.settings.categories.time_desc")}</p>
        </div>
      </div>
      {activeTab === "general" && (
        <Button
          icon={<Plus size={18} />}
          onClick={() => {
            onNewVersion();
          }}
        >
          {t("common.workspace_settings.new_version") || "NEW VERSION"}
        </Button>
      )}
    </div>
  );
}

function TabNavigation({ activeTab, setActiveTab, t }: { activeTab: TabType; setActiveTab: (v: TabType) => void; t: TFunction }) {
  return (
    <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
      <button
        className={cn(
          "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
          activeTab === "general"
            ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700"
        )}
        onClick={() => {
          setActiveTab("general");
        }}
        type="button"
      >
        <Settings2 size={14} /> {t("common.settings.tabs.general")}
      </button>
      <button
        className={cn(
          "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
          activeTab === "patterns"
            ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700"
        )}
        onClick={() => {
          setActiveTab("patterns");
        }}
        type="button"
      >
        <LayoutGrid size={14} /> {t("common.settings.tabs.patterns")}
      </button>
      <button
        className={cn(
          "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
          activeTab === "rules"
            ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700"
        )}
        onClick={() => {
          setActiveTab("rules");
        }}
        type="button"
      >
        <Zap size={14} /> {t("common.settings.tabs.rules")}
      </button>
    </div>
  );
}

interface TabContentProps {
  activeTab: TabType;
  earlyArrivalStrategy: "allow" | "block" | "clamp";
  handleSaveNewVersion: () => void;
  isPending: boolean;
  newValidFrom: string;
  reprocessEnd: string;
  reprocessEndSetter: (v: string) => void;
  reprocessStart: string;
  reprocessStartSetter: (v: string) => void;
  roundingPolicy: string;
  setEarlyArrivalStrategy: (v: "allow" | "block" | "clamp") => void;
  setNewValidFrom: (v: string) => void;
  setRoundingPolicy: (v: string) => void;
  setShowCreateModal: (v: boolean) => void;
  setShowReprocessConfirm: (v: boolean) => void;
  setWorkUnit: (v: number) => void;
  showCreateModal: boolean;
  t: TFunction;
  versions: AttendancePolicyVersion[];
  workUnit: number;
  wsId: string;
}

function TabContent({
  activeTab,
  earlyArrivalStrategy,
  handleSaveNewVersion,
  isPending,
  newValidFrom,
  reprocessEnd,
  reprocessEndSetter,
  reprocessStart,
  reprocessStartSetter,
  setEarlyArrivalStrategy,
  setNewValidFrom,
  setRoundingPolicy,
  setShowCreateModal,
  setShowReprocessConfirm,
  setWorkUnit,
  showCreateModal,
  roundingPolicy,
  t,
  versions,
  workUnit,
  wsId,
}: TabContentProps) {
  if (activeTab === "patterns") {
    return (
      <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm p-4">
        <WorkingPatternsPage />
      </div>
    );
  }

  if (activeTab === "rules") {
    return (
      <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm p-4">
        <AssignmentRulesPage />
      </div>
    );
  }

  return (
    <>
      <AttendanceVersionTimeline t={t} versions={versions} />
      <PolicyAssignmentsSection wsId={wsId} />
      <BulkReprocessSection
        reprocessEnd={reprocessEnd}
        reprocessStart={reprocessStart}
        setReprocessEnd={reprocessEndSetter}
        setReprocessStart={reprocessStartSetter}
        setShowReprocessConfirm={setShowReprocessConfirm}
        t={t}
      />
      <CreateVersionModal
        earlyArrivalStrategy={earlyArrivalStrategy}
        isOpen={showCreateModal}
        isPending={isPending}
        newValidFrom={newValidFrom}
        onClose={() => {
          setShowCreateModal(false);
        }}
        onSave={handleSaveNewVersion}
        roundingPolicy={roundingPolicy}
        setEarlyArrivalStrategy={setEarlyArrivalStrategy}
        setNewValidFrom={setNewValidFrom}
        setRoundingPolicy={setRoundingPolicy}
        setWorkUnit={setWorkUnit}
        t={t}
        workUnit={workUnit}
      />
    </>
  );
}

interface ButtonProps {
  children: React.ReactNode;
  className?: string;
  icon?: React.ReactNode;
  loading?: boolean;
  onClick: () => void;
  size?: "lg" | "sm";
  variant?: "outline" | "secondary";
}

function Button({ children, className, icon, loading, onClick, size, variant }: ButtonProps) {
  return (
    <button
      className={cn(
        "flex items-center gap-2 px-6 py-3.5 rounded-2xl font-bold transition-all active:scale-95 disabled:opacity-50",
        variant === "outline"
          ? "border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-300"
          : "bg-primary-600 hover:bg-primary-700 text-white shadow-xl shadow-primary-500/20",
        size === "lg" ? "px-8 py-4 text-sm" : "text-xs",
        className
      )}
      disabled={loading}
      onClick={() => {
        onClick();
      }}
      type="button"
    >
      {loading ? <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" /> : icon}
      {children}
    </button>
  );
}
