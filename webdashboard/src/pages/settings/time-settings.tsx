import { useQuery } from "@tanstack/react-query";
import { Clock } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { Typography } from "@/components/ui/typography";
import {
  fetchWorkspace,
  getCurrentWorkspaceId,
  fetchDefaultAttendanceRules,
  fetchAttendancePolicyVersions,
  AttendanceRules,
  AttendancePolicyVersion,
  Workspace,
} from "@/lib/api";

import { PageHeader, TabContent } from "./time-settings/components";
import { useTimeSettingsRules, useBulkReprocess } from "./time-settings/hooks";
import { ReprocessModal } from "./time-settings/reprocess-modal";

  // eslint-disable-next-line complexity
export default function TimeSettings() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();

  const { data: workspace, isLoading: isLoadingWs, isError: isErrorWs, error: errorWs } = useQuery({
    queryKey: ["workspace", wsId],
    queryFn: () => (wsId ? fetchWorkspace(wsId) : Promise.reject(new Error("No workspace ID"))),
    enabled: !!wsId,
  });

  const { data: rules, isLoading: isLoadingRules, isError: isErrorRules, error: errorRules } = useQuery({
    queryKey: ["attendance-rules", wsId],
    queryFn: () => (wsId ? fetchDefaultAttendanceRules(wsId) : Promise.reject(new Error("No workspace ID"))),
    enabled: !!wsId,
  });

  const { data: versions = [], isLoading: isLoadingVersions, isError: isErrorVersions, error: errorVersions } = useQuery({
    queryKey: ["attendance-policy-versions", wsId],
    queryFn: () => (wsId ? fetchAttendancePolicyVersions(wsId) : Promise.reject(new Error("No workspace ID"))),
    enabled: !!wsId,
  });

  const isError = isErrorWs || isErrorRules || isErrorVersions;
  const error = errorWs ?? errorRules ?? errorVersions;

  if (isLoadingWs || isLoadingRules || isLoadingVersions) {
    return (
      <div className="space-y-6">
        <Skeleton className="h-[200px] w-full rounded-[2.5rem]" />
        <Skeleton className="h-[400px] w-full rounded-[2.5rem]" />
      </div>
    );
  }

  if (isError) {
    return (
      <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm flex flex-col items-center justify-center min-h-[300px] text-center space-y-4">
        <div className="w-16 h-16 rounded-full bg-rose-50 dark:bg-rose-950/30 flex items-center justify-center text-rose-50 border border-rose-100 dark:border-rose-900/50">
          <Clock className="w-8 h-8 text-rose-500" />
        </div>
        <div>
          <Typography className="text-lg font-bold text-slate-800 dark:text-slate-200" variant="h3">
            {t("common.settings.time.error_title") || "Failed to Load Time Settings"}
          </Typography>
          <Typography className="text-sm text-slate-500 dark:text-slate-400 mt-1 max-w-md" variant="p">
            {error instanceof Error ? error.message : t("common.settings.time.error_desc") || "An unexpected error occurred while fetching the settings rules. Please try again."}
          </Typography>
        </div>
        <Button
          className="px-6 py-2.5 bg-primary-600 hover:bg-primary-700 text-white font-semibold rounded-2xl shadow-sm transition-all"
          onClick={() => {
            window.location.reload();
          }}
        >
          {t("common.retry") || "Retry"}
        </Button>
      </div>
    );
  }

  if (!workspace || !rules) {
    return (
      <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm flex flex-col items-center justify-center min-h-[300px] text-center space-y-4">
        <div className="w-16 h-16 rounded-full bg-amber-50 dark:bg-amber-950/30 flex items-center justify-center text-amber-500 border border-amber-100 dark:border-amber-900/50">
          <Clock className="w-8 h-8 text-amber-500" />
        </div>
        <div>
          <Typography className="text-lg font-bold text-slate-800 dark:text-slate-200" variant="h3">
            {t("common.settings.time.missing_title") || "No Data Found"}
          </Typography>
          <Typography className="text-sm text-slate-500 dark:text-slate-400 mt-1 max-w-md" variant="p">
            {t("common.settings.time.missing_desc") || "Workspace details or default rules are not configured for this workspace."}
          </Typography>
        </div>
      </div>
    );
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

  const [workUnit, setWorkUnit] = useState(rules.work_unit_minutes);
  const [roundingPolicy, setRoundingPolicy] = useState(rules.rounding_policy);
  const [earlyArrivalStrategy, setEarlyArrivalStrategy] = useState<"allow" | "block" | "clamp">(
    rules.early_arrival_strategy ?? "block"
  );
  const [gracePeriodMinutes, setGracePeriodMinutes] = useState(rules.grace_period_minutes ?? 15);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newValidFrom, setNewValidFrom] = useState(() => new Date().toISOString().split("T")[0]);

  const {
    isPending: isBulkReprocessPending,
    jobStatus,
    pollingJobId,
    reprocessEnd,
    reprocessStart,
    submitBulkReprocess,
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
    grace_period_minutes: gracePeriodMinutes,
    onSuccess: () => {
      setShowCreateModal(false);
    },
  });

  return (
    <div className="space-y-6 animate-in fade-in duration-500">
      <PageHeader
        onNewVersion={() => {
          setShowCreateModal(true);
          setNewValidFrom(new Date().toISOString().split("T")[0]);
        }}
        t={t}
      />
      <div className="animate-in slide-in-from-bottom-4 duration-500">
        <TabContent
          earlyArrivalStrategy={earlyArrivalStrategy}
          gracePeriodMinutes={gracePeriodMinutes}
          handleSaveNewVersion={handleSaveNewVersion}
          isPending={isPending}
          newValidFrom={newValidFrom}
          reprocessEnd={reprocessEnd}
          reprocessEndSetter={setReprocessEnd}
          reprocessStart={reprocessStart}
          reprocessStartSetter={setReprocessStart}
          roundingPolicy={roundingPolicy}
          setEarlyArrivalStrategy={setEarlyArrivalStrategy}
          setGracePeriodMinutes={setGracePeriodMinutes}
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
        isPending={isBulkReprocessPending}
        jobStatus={jobStatus}
        onClose={() => {
          if (!isBulkReprocessPending && !pollingJobId) {
            setShowReprocessConfirm(false);
          }
        }}
        onSubmit={submitBulkReprocess}
        pollingJobId={pollingJobId}
        reprocessEnd={reprocessEnd}
        reprocessStart={reprocessStart}
        t={t}
      />
    </div>
  );
}
