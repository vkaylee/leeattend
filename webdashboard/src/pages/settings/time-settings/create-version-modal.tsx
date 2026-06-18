 
import { Calculator, Clock, Plus, ShieldAlert, Settings2, Zap } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { LABEL_BASE } from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";

interface CreateVersionModalProps {
  earlyArrivalStrategy: "allow" | "block" | "clamp";
  gracePeriodMinutes: number;
  isOpen: boolean;
  isPending: boolean;
  newValidFrom: string;
  onClose: () => void;
  onSave: () => void;
  roundingPolicy: string;
  setEarlyArrivalStrategy: (v: "allow" | "block" | "clamp") => void;
  setGracePeriodMinutes: (v: number) => void;
  setNewValidFrom: (v: string) => void;
  setRoundingPolicy: (v: string) => void;
  setWorkUnit: (v: number) => void;
  t: TFunction;
  workUnit: number;
}

export function CreateVersionModal({
  earlyArrivalStrategy,
  gracePeriodMinutes,
  isOpen,
  isPending,
  newValidFrom,
  onClose,
  onSave,
  roundingPolicy,
  setEarlyArrivalStrategy,
  setGracePeriodMinutes,
  setNewValidFrom,
  setRoundingPolicy,
  setWorkUnit,
  t,
  workUnit,
}: CreateVersionModalProps) {
  return (
    <Modal
      icon={<Plus size={24} />}
      isOpen={isOpen}
      maxWidth="xl"
      onClose={onClose}
      subtitle={
        t("common.workspace_settings.new_version_desc") ||
        "Create a new configuration version with an effective date"
      }
      title={t("common.workspace_settings.new_version") || "New Attendance Config"}
    >
      <div className="space-y-8 pt-4 animate-in fade-in duration-500">
        <DateInput
          newValidFrom={newValidFrom}
          setNewValidFrom={setNewValidFrom}
          t={t}
        />

        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          <WorkUnitInput
            setWorkUnit={setWorkUnit}
            t={t}
            workUnit={workUnit}
          />
          <RoundingPolicyInput
            roundingPolicy={roundingPolicy}
            setRoundingPolicy={setRoundingPolicy}
            t={t}
          />
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          <EarlyArrivalInput
            earlyArrivalStrategy={earlyArrivalStrategy}
            setEarlyArrivalStrategy={setEarlyArrivalStrategy}
            t={t}
          />
          <GracePeriodInput
            gracePeriodMinutes={gracePeriodMinutes}
            setGracePeriodMinutes={setGracePeriodMinutes}
            t={t}
          />
        </div>

        <ModalFooter
          isPending={isPending}
          onClose={onClose}
          onSave={onSave}
          t={t}
        />
      </div>
    </Modal>
  );
}

function DateInput({
  newValidFrom,
  setNewValidFrom,
  t,
}: {
  newValidFrom: string;
  setNewValidFrom: (v: string) => void;
  t: TFunction;
}) {
  return (
    <div className="space-y-3 px-1">
      <label className={LABEL_BASE}>{t("common.workspace_settings.valid_from") || "Effective From"}</label>
      <input
        className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight shadow-sm"
        onChange={(e) => {
          setNewValidFrom(e.target.value);
        }}
        type="date"
        value={newValidFrom}
      />
      <Typography className="text-[9px] text-slate-500 dark:text-slate-400 font-bold uppercase italic leading-relaxed px-1 opacity-60" variant="p">
        {t("common.workspace_settings.valid_from_note") ||
          "Payroll calculations will use this configuration from this date forward."}
      </Typography>
    </div>
  );
}

function WorkUnitInput({
  setWorkUnit,
  t,
  workUnit,
}: {
  setWorkUnit: (v: number) => void;
  t: TFunction;
  workUnit: number;
}) {
  return (
    <div className="space-y-3 px-1">
      <label className={LABEL_BASE}>{t("common.workspace_settings.work_unit")}</label>
      <Combobox
        className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14"
        onChange={(v) => {
          setWorkUnit(parseInt(v));
        }}
        options={[
          {
            description: t("common.workspace_settings.work_unit_options.1min_desc"),
            icon: <Clock size={14} />,
            label: t("common.workspace_settings.work_unit_options.1min"),
            value: "1",
          },
          {
            description: t("common.workspace_settings.work_unit_options.15min_desc"),
            icon: <Clock size={14} />,
            label: t("common.workspace_settings.work_unit_options.15min"),
            value: "15",
          },
          {
            description: t("common.workspace_settings.work_unit_options.30min_desc"),
            icon: <Clock size={14} />,
            label: t("common.workspace_settings.work_unit_options.30min"),
            value: "30",
          },
          {
            description: t("common.workspace_settings.work_unit_options.60min_desc"),
            icon: <Clock size={14} />,
            label: t("common.workspace_settings.work_unit_options.60min"),
            value: "60",
          },
        ]}
        value={workUnit.toString()}
      />
    </div>
  );
}

function RoundingPolicyInput({
  roundingPolicy,
  setRoundingPolicy,
  t,
}: {
  roundingPolicy: string;
  setRoundingPolicy: (v: string) => void;
  t: TFunction;
}) {
  return (
    <div className="space-y-3 px-1">
      <label className={LABEL_BASE}>{t("common.workspace_settings.rounding_policy")}</label>
      <Combobox
        className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14"
        onChange={setRoundingPolicy}
        options={[
          {
            icon: <ShieldAlert size={14} />,
            label: t("common.workspace_settings.rounding_options.none"),
            value: "none",
          },
          {
            icon: <Calculator size={14} />,
            label: t("common.workspace_settings.rounding_options.floor"),
            value: "floor",
          },
          {
            icon: <Calculator size={14} />,
            label: t("common.workspace_settings.rounding_options.ceil"),
            value: "ceil",
          },
          {
            icon: <Calculator size={14} />,
            label: t("common.workspace_settings.rounding_options.round"),
            value: "round",
          },
        ]}
        value={roundingPolicy || "none"}
      />
    </div>
  );
}

function EarlyArrivalInput({
  earlyArrivalStrategy,
  setEarlyArrivalStrategy,
  t,
}: {
  earlyArrivalStrategy: "allow" | "block" | "clamp";
  setEarlyArrivalStrategy: (v: "allow" | "block" | "clamp") => void;
  t: TFunction;
}) {
  return (
    <div className="space-y-3 px-1">
      <label className={LABEL_BASE}>{t("common.workspace_settings.early_arrival_strategy")}</label>
      <Combobox
        className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14"
        onChange={(v) => {
          setEarlyArrivalStrategy(v as "block" | "allow" | "clamp");
        }}
        options={[
          {
            description: t("common.workspace_settings.early_arrival_options.block_desc"),
            icon: <ShieldAlert size={14} />,
            label: t("common.workspace_settings.early_arrival_options.block"),
            value: "block",
          },
          {
            description: t("common.workspace_settings.early_arrival_options.allow_desc"),
            icon: <Zap size={14} />,
            label: t("common.workspace_settings.early_arrival_options.allow"),
            value: "allow",
          },
          {
            description: t("common.workspace_settings.early_arrival_options.clamp_desc"),
            icon: <Settings2 size={14} />,
            label: t("common.workspace_settings.early_arrival_options.clamp"),
            value: "clamp",
          },
        ]}
        value={earlyArrivalStrategy}
      />
    </div>
  );
}

function GracePeriodInput({
  gracePeriodMinutes,
  setGracePeriodMinutes,
  t,
}: {
  gracePeriodMinutes: number;
  setGracePeriodMinutes: (v: number) => void;
  t: TFunction;
}) {
  return (
    <div className="space-y-3 px-1">
      <label className={LABEL_BASE}>{t("common.workspace_settings.grace_period_minutes") || "Late Grace Period (minutes)"}</label>
      <input
        className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight shadow-sm"
        min={0}
        max={1440}
        onChange={(e) => {
          const val = parseInt(e.target.value);
          setGracePeriodMinutes(isNaN(val) ? 0 : val);
        }}
        type="number"
        value={gracePeriodMinutes}
      />
      <Typography className="text-[9px] text-slate-500 dark:text-slate-400 font-bold uppercase italic leading-relaxed px-1 opacity-60" variant="p">
        {t("common.workspace_settings.grace_period_minutes_desc") ||
          "Allowed buffer time after shift start before attendance is marked as late."}
      </Typography>
    </div>
  );
}

function ModalFooter({
  isPending,
  onClose,
  onSave,
  t,
}: {
  isPending: boolean;
  onClose: () => void;
  onSave: () => void;
  t: TFunction;
}) {
  return (
    <div className="pt-8 border-t border-slate-50 dark:border-slate-800 grid grid-cols-2 gap-6">
      <Button
        onClick={onClose}
        size="lg"
        variant="outline"
      >
        {t("common.cancel").toUpperCase()}
      </Button>
      <Button
        loading={isPending}
        onClick={onSave}
        size="lg"
        variant="secondary"
      >
        {(t("common.workspace_settings.save_config") || "SAVE").toUpperCase()}
      </Button>
    </div>
  );
}
