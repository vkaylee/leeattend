import { Clock, Plus } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import { PolicyAssignmentsSection } from "@/components/features/attendance/policy-assignments-section";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { AttendancePolicyVersion } from "@/lib/api";

import { BulkReprocessSection } from "./bulk-reprocess";
import { CreateVersionModal } from "./create-version-modal";
import { AttendanceVersionTimeline } from "./timeline";

export function PageHeader({ onNewVersion, t }: { onNewVersion: () => void; t: TFunction }) {
  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-2 border-b border-slate-200/60 dark:border-slate-800/60">
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950/70 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-2xs border border-primary-100/50 dark:border-primary-900/50">
          <Clock size={20} />
        </div>
        <div>
          <Typography className="text-xl font-bold tracking-tight text-slate-800 dark:text-slate-100 normal-case" variant="h1">
            {t("common.settings.categories.time")}
          </Typography>
          <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium normal-case tracking-normal" variant="small">{t("common.settings.categories.time_desc")}</Typography>
        </div>
      </div>
      <Button
        className="rounded-xl px-4 py-2 text-xs font-semibold shadow-xs"
        icon={<Plus size={16} />}
        onClick={() => {
          onNewVersion();
        }}
      >
        {t("common.workspace_settings.new_version", { defaultValue: "Phiên bản mới" })}
      </Button>
    </div>
  );
}

interface TabContentProps {
  earlyArrivalStrategy: "allow" | "block" | "clamp";
  gracePeriodMinutes: number;
  handleSaveNewVersion: () => void;
  isPending: boolean;
  newValidFrom: string;
  reprocessEnd: string;
  reprocessEndSetter: (v: string) => void;
  reprocessStart: string;
  reprocessStartSetter: (v: string) => void;
  roundingPolicy: AttendancePolicyVersion["rounding_policy"];
  setEarlyArrivalStrategy: (v: "allow" | "block" | "clamp") => void;
  setGracePeriodMinutes: (v: number) => void;
  setNewValidFrom: (v: string) => void;
  setRoundingPolicy: (v: AttendancePolicyVersion["rounding_policy"]) => void;
  setShowCreateModal: (v: boolean) => void;
  setShowReprocessConfirm: (v: boolean) => void;
  setWorkUnit: (v: number) => void;
  showCreateModal: boolean;
  t: TFunction;
  versions: AttendancePolicyVersion[];
  workUnit: number;
  wsId: string;
}

export function TabContent({
  earlyArrivalStrategy,
  gracePeriodMinutes,
  handleSaveNewVersion,
  isPending,
  newValidFrom,
  reprocessEnd,
  reprocessEndSetter,
  reprocessStart,
  reprocessStartSetter,
  setEarlyArrivalStrategy,
  setGracePeriodMinutes,
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
        gracePeriodMinutes={gracePeriodMinutes}
        isOpen={showCreateModal}
        isPending={isPending}
        newValidFrom={newValidFrom}
        onClose={() => {
          setShowCreateModal(false);
        }}
        onSave={handleSaveNewVersion}
        roundingPolicy={roundingPolicy}
        setEarlyArrivalStrategy={setEarlyArrivalStrategy}
        setGracePeriodMinutes={setGracePeriodMinutes}
        setNewValidFrom={setNewValidFrom}
        setRoundingPolicy={setRoundingPolicy}
        setWorkUnit={setWorkUnit}
        t={t}
        workUnit={workUnit}
      />
    </>
  );
}
