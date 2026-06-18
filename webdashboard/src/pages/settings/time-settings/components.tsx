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
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
      <div className="flex items-center gap-4">
        <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50">
          <Clock size={24} />
        </div>
        <div>
          <Typography className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200 leading-tight" variant="h1">
            {t("common.settings.categories.time")}
          </Typography>
          <Typography className="text-slate-500 dark:text-slate-400 mt-1 font-medium" variant="p">{t("common.settings.categories.time_desc")}</Typography>
        </div>
      </div>
      <Button
        icon={<Plus size={18} />}
        onClick={() => {
          onNewVersion();
        }}
      >
        {t("common.workspace_settings.new_version") || "NEW VERSION"}
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
  roundingPolicy: string;
  setEarlyArrivalStrategy: (v: "allow" | "block" | "clamp") => void;
  setGracePeriodMinutes: (v: number) => void;
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
