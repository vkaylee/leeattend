import { AlertCircle, Coffee, Info } from "lucide-react";
import { memo } from "react";

import type { ShiftDetailsListProps } from "./types";

interface BreakPolicyProps {
  isAutoDeduct: boolean;
  isBreakOvertime: boolean;
  isFixedBreak: boolean;
  t: ShiftDetailsListProps["t"];
}

export const BreakPolicy = memo(function BreakPolicy({
  isAutoDeduct,
  isBreakOvertime,
  isFixedBreak,
  t,
}: BreakPolicyProps) {
  let policyText = "";
  if (isFixedBreak && !isAutoDeduct && !isBreakOvertime) {
    policyText = t("shifts.modal.break_policy_no_deduct");
  } else if (isFixedBreak && isBreakOvertime) {
    policyText = t("shifts.modal.break_policy_overtime");
  } else if (isAutoDeduct) {
    policyText = t("shifts.modal.break_policy_auto_deduct");
  } else {
    policyText = t("shifts.modal.break_policy_no_deduct");
  }

  return (
    <div className="flex justify-between items-center pb-2.5 border-b border-slate-50 dark:border-slate-800/80">
      <span className="font-bold text-slate-500 dark:text-slate-400">{t("shifts.modal.break_policy_label")}</span>
      <span className="font-bold text-slate-700 dark:text-slate-200">
        {policyText}
      </span>
    </div>
  );
});

interface GracePeriodProps {
  shiftGrace: number;
  t: ShiftDetailsListProps["t"];
}

export const GracePeriod = memo(function GracePeriod({ shiftGrace, t }: GracePeriodProps) {
  if (shiftGrace <= 0) return null;
  return (
    <div className="flex justify-between items-center pb-2.5 border-b border-slate-50 dark:border-slate-800/80">
      <span className="font-bold text-slate-500 dark:text-slate-400 flex items-center gap-1.5">
        <Info className="text-slate-400" size={12} />
        {t("shifts.grace_period")}
      </span>
      <span className="font-bold text-slate-700 dark:text-slate-200 font-mono">
        -{shiftGrace} {t("common.minutes")}
      </span>
    </div>
  );
});

interface EarlyOtProps {
  allowEarlyOt: boolean;
  maxEarlyOtMinutes: number;
  t: ShiftDetailsListProps["t"];
}

export const EarlyOt = memo(function EarlyOt({
  allowEarlyOt,
  maxEarlyOtMinutes,
  t,
}: EarlyOtProps) {
  if (!allowEarlyOt) return null;
  return (
    <div className="flex justify-between items-center">
      <span className="font-bold text-slate-500 dark:text-slate-400 flex items-center gap-1.5">
        <AlertCircle className="text-blue-500" size={12} />
        {t("shifts.modal.allow_early_ot")}
      </span>
      <span className="font-bold text-slate-700 dark:text-slate-200 font-mono">
        +{maxEarlyOtMinutes} {t("common.minutes")} max
      </span>
    </div>
  );
});

export const ShiftDetailsList = memo(function ShiftDetailsList({
  shiftName,
  shiftStart,
  shiftEnd,
  isFixedBreak,
  breakStart,
  breakEnd,
  shiftBreak,
  isAutoDeduct,
  isBreakOvertime,
  shiftGrace,
  allowEarlyOt,
  maxEarlyOtMinutes,
  t,
}: ShiftDetailsListProps) {
  return (
    <div className="bg-white dark:bg-slate-900 p-5 rounded-3xl border border-slate-100 dark:border-slate-800 shadow-sm space-y-3 text-xs">
      <div className="flex justify-between items-center pb-2.5 border-b border-slate-50 dark:border-slate-800/80">
        <span className="font-bold text-slate-500 dark:text-slate-400">{t("shifts.modal.shift_name_label", "Tên ca")}</span>
        <span className="font-black text-slate-700 dark:text-slate-200 uppercase">{shiftName || "CHƯA ĐẶT TÊN"}</span>
      </div>
      <div className="flex justify-between items-center pb-2.5 border-b border-slate-50 dark:border-slate-800/80">
        <span className="font-bold text-slate-500 dark:text-slate-400">{t("shifts.modal.start_time")}</span>
        <span className="font-bold font-mono text-slate-700 dark:text-slate-200">{shiftStart || "--:--"}</span>
      </div>
      <div className="flex justify-between items-center pb-2.5 border-b border-slate-50 dark:border-slate-800/80">
        <span className="font-bold text-slate-500 dark:text-slate-400">{t("shifts.modal.end_time")}</span>
        <span className="font-bold font-mono text-slate-700 dark:text-slate-200">{shiftEnd || "--:--"}</span>
      </div>

      {/* Break Details */}
      <div className="flex justify-between items-center pb-2.5 border-b border-slate-50 dark:border-slate-800/80">
        <span className="font-bold text-slate-500 dark:text-slate-400 flex items-center gap-1.5">
          <Coffee className="text-amber-500" size={12} />
          {t("shifts.modal.break_config")}
        </span>
        <span className="font-bold text-slate-700 dark:text-slate-200">
          {isFixedBreak ? (
            <span className="font-mono">{breakStart || "--:--"} - {breakEnd || "--:--"}</span>
          ) : (
            <span>{shiftBreak} {t("common.minutes")}</span>
          )}
        </span>
      </div>

      <BreakPolicy
        isAutoDeduct={isAutoDeduct}
        isBreakOvertime={isBreakOvertime}
        isFixedBreak={isFixedBreak}
        t={t}
      />

      <GracePeriod shiftGrace={shiftGrace} t={t} />

      <EarlyOt
        allowEarlyOt={allowEarlyOt}
        maxEarlyOtMinutes={maxEarlyOtMinutes}
        t={t}
      />
    </div>
  );
});
