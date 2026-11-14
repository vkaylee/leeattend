import { Clock, Plus } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import { PolicyAssignmentsSection } from "@/components/features/attendance/policy-assignments-section";
import Button from "@/components/ui/button";
import { SettingsHeader } from "@/components/ui/settings/settings-header";
import { AttendancePolicyVersion } from "@/lib/api";

import { BulkReprocessSection } from "./bulk-reprocess";
import { CreateVersionModal } from "./create-version-modal";
import { AttendanceVersionTimeline } from "./timeline";

export function PageHeader({ onNewVersion, t }: { onNewVersion: () => void; t: TFunction }) {
  return (
    <SettingsHeader
      actions={
        <Button
          className="rounded-xl px-4 py-2 text-xs font-semibold shadow-xs normal-case tracking-normal"
          icon={<Plus size={16} />}
          onClick={onNewVersion}
          variant="primary"
        >
          {t("common.organization_settings.new_version", { defaultValue: "Phiên bản mới" })}
        </Button>
      }
      description={t("common.settings.categories.time_desc")}
      icon={<Clock size={20} />}
      title={t("common.settings.categories.time")}
    />
  );
}

interface TabContentProps { earlyArrivalStrategy: "allow" | "block" | "clamp";
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
workUnit: number;  }

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
}: TabContentProps) {
  return (
    <>
      <AttendanceVersionTimeline onNewVersion={() => { setShowCreateModal(true); }} t={t} versions={versions} />
      <PolicyAssignmentsSection   />
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
