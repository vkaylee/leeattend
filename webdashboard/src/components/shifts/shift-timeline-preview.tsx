import { Clock, Coffee, AlertCircle, Info } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { TFunction } from "i18next";

interface ShiftTimelinePreviewProps {
  allowEarlyOt: boolean;
  breakEnd: string;
  breakStart: string;
  isAutoDeduct: boolean;
  isBreakOvertime: boolean;
  isFixedBreak: boolean;
  maxEarlyOtMinutes: number;
  shiftBreak: number;
  shiftEnd: string;
  shiftGrace: number;
  shiftName: string;
  shiftStart: string;
}

const timeToMinutes = (t: string): number => {
  if (!t) return 0;
  const [h, m] = t.split(":").map(Number);
  return (h || 0) * 60 + (m || 0);
};

const getSegmentStyles = (start: number, end: number) => {
  const left = (start / 1440) * 100;
  const width = end >= start ? ((end - start) / 1440) * 100 : 0;
  return { left: `${left}%`, width: `${width}%` };
};

const calculateShiftDuration = (shiftStart: string, shiftEnd: string, startMin: number, endMin: number): number => {
  if (!shiftStart || !shiftEnd) return 0;
  if (endMin >= startMin) {
    return endMin - startMin;
  }
  return 1440 - startMin + endMin;
};

const calculateBreakDuration = (
  isFixedBreak: boolean,
  breakStart: string,
  breakEnd: string,
  shiftBreak: number
): number => {
  if (isFixedBreak && breakStart && breakEnd) {
    const bStart = timeToMinutes(breakStart);
    const bEnd = timeToMinutes(breakEnd);
    if (bEnd >= bStart) {
      return bEnd - bStart;
    }
    return 1440 - bStart + bEnd;
  }
  if (!isFixedBreak) {
    return shiftBreak || 0;
  }
  return 0;
};

interface TimelineBarProps {
  allowEarlyOt: boolean;
  breakEnd: string;
  breakStart: string;
  endMin: number;
  isCrossMidnight: boolean;
  isFixedBreak: boolean;
  maxEarlyOtMinutes: number;
  shiftEnd: string;
  shiftStart: string;
  startMin: number;
}

function TimelineBar({
  allowEarlyOt,
  breakEnd,
  breakStart,
  endMin,
  isCrossMidnight,
  isFixedBreak,
  maxEarlyOtMinutes,
  shiftEnd,
  shiftStart,
  startMin,
}: TimelineBarProps) {
  const bStart = timeToMinutes(breakStart);
  const bEnd = timeToMinutes(breakEnd);
  const bCross = bEnd < bStart;
  const earlyMin = Math.max(0, startMin - maxEarlyOtMinutes);

  return (
    <div className="relative h-14 bg-slate-200 dark:bg-slate-900 rounded-3xl overflow-hidden shadow-inner border border-slate-150 dark:border-slate-800">
      {/* Hour markers background */}
      <div className="absolute inset-0 flex justify-between px-2 pointer-events-none opacity-20">
        {Array.from({ length: 7 }).map((_, i) => (
          <div className="h-full w-px bg-slate-400 dark:bg-slate-500" key={i} />
        ))}
      </div>

      {/* Render Shift Duration */}
      {shiftStart && shiftEnd ? (
        !isCrossMidnight ? (
          <div
            className="absolute h-full bg-primary-500/20 dark:bg-primary-500/10 border-x border-primary-500/50 flex items-center justify-center"
            style={getSegmentStyles(startMin, endMin)}
          >
            <div className="h-[70%] w-full bg-primary-600 dark:bg-primary-500 rounded-2xl mx-1 shadow-sm opacity-90" />
          </div>
        ) : (
          <>
            {/* Segment 1: start to midnight */}
            <div
              className="absolute h-full bg-primary-500/20 dark:bg-primary-500/10 border-l border-primary-500/50 flex items-center justify-center"
              style={getSegmentStyles(startMin, 1440)}
            >
              <div className="h-[70%] w-full bg-primary-600 dark:bg-primary-500 rounded-l-2xl ml-1 shadow-sm opacity-90" />
            </div>
            {/* Segment 2: midnight to end */}
            <div
              className="absolute h-full bg-primary-500/20 dark:bg-primary-500/10 border-r border-primary-500/50 flex items-center justify-center"
              style={getSegmentStyles(0, endMin)}
            >
              <div className="h-[70%] w-full bg-primary-600 dark:bg-primary-500 rounded-r-2xl mr-1 shadow-sm opacity-90" />
            </div>
          </>
        )
      ) : null}

      {/* Render Fixed Break Segment */}
      {isFixedBreak && breakStart && breakEnd ? (
        !bCross ? (
          <div
            className="absolute h-full flex items-center justify-center z-10"
            style={getSegmentStyles(bStart, bEnd)}
          >
            <div className="h-[70%] w-full bg-amber-500 dark:bg-amber-600 rounded-2xl mx-0.5 shadow-sm border border-amber-400/30 flex items-center justify-center text-[9px] text-white font-bold">
              ☕
            </div>
          </div>
        ) : (
          <>
            <div
              className="absolute h-full flex items-center justify-center z-10"
              style={getSegmentStyles(bStart, 1440)}
            >
              <div className="h-[70%] w-full bg-amber-500 dark:bg-amber-600 rounded-l-2xl ml-0.5 shadow-sm flex items-center justify-center text-[9px] text-white font-bold">
                ☕
              </div>
            </div>
            <div
              className="absolute h-full flex items-center justify-center z-10"
              style={getSegmentStyles(0, bEnd)}
            >
              <div className="h-[70%] w-full bg-amber-500 dark:bg-amber-600 rounded-r-2xl mr-0.5 shadow-sm flex items-center justify-center text-[9px] text-white font-bold">
                ☕
              </div>
            </div>
          </>
        )
      ) : null}

      {/* Visual indicators for grace/early OT */}
      {allowEarlyOt && shiftStart ? (
        <div
          className="absolute h-full border-l-2 border-dashed border-blue-400 dark:border-blue-500/80 bg-blue-400/10 z-0 pointer-events-none"
          style={getSegmentStyles(earlyMin, startMin)}
        />
      ) : null}
    </div>
  );
}

interface ShiftDetailsListProps {
  allowEarlyOt: boolean;
  breakEnd: string;
  breakStart: string;
  isAutoDeduct: boolean;
  isBreakOvertime: boolean;
  isFixedBreak: boolean;
  maxEarlyOtMinutes: number;
  shiftBreak: number;
  shiftEnd: string;
  shiftGrace: number;
  shiftName: string;
  shiftStart: string;
  t: TFunction;
}

function BreakPolicy({
  isAutoDeduct,
  isBreakOvertime,
  isFixedBreak,
  t,
}: {
  isAutoDeduct: boolean;
  isBreakOvertime: boolean;
  isFixedBreak: boolean;
  t: TFunction;
}) {
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
}

function GracePeriod({ shiftGrace, t }: { shiftGrace: number; t: TFunction }) {
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
}

function EarlyOt({
  allowEarlyOt,
  maxEarlyOtMinutes,
  t,
}: {
  allowEarlyOt: boolean;
  maxEarlyOtMinutes: number;
  t: TFunction;
}) {
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
}

function ShiftDetailsList({
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
}

function MetricsGrid({
  shiftDurationMin,
  workingHours,
  t,
}: {
  shiftDurationMin: number;
  workingHours: string;
  t: TFunction;
}) {
  return (
    <div className="grid grid-cols-2 gap-4">
      {/* Metric 1: Total shift hours */}
      <div className="bg-white dark:bg-slate-900 p-4 rounded-2xl border border-slate-100 dark:border-slate-800 shadow-sm space-y-1">
        <span className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest block">
          {t("shifts.modal.shift_duration", "Tổng thời gian ca")}
        </span>
        <div className="flex items-baseline gap-1 text-slate-700 dark:text-slate-200">
          <span className="text-xl font-black font-mono">
            {(shiftDurationMin / 60).toFixed(1)}
          </span>
          <span className="text-[10px] font-bold text-slate-500">{t("common.hours", "giờ")}</span>
        </div>
      </div>

      {/* Metric 2: Net working hours */}
      <div className="bg-white dark:bg-slate-900 p-4 rounded-2xl border border-slate-100 dark:border-slate-800 shadow-sm space-y-1">
        <span className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest block">
          {t("shifts.modal.working_hours", "Công thực tế")}
        </span>
        <div className="flex items-baseline gap-1 text-primary-600 dark:text-primary-400">
          <span className="text-xl font-black font-mono">
            {workingHours}
          </span>
          <span className="text-[10px] font-bold text-primary-500">{t("common.hours", "giờ")}</span>
        </div>
      </div>
    </div>
  );
}

export function ShiftTimelinePreview({
  allowEarlyOt,
  breakEnd,
  breakStart,
  isAutoDeduct,
  isBreakOvertime,
  isFixedBreak,
  maxEarlyOtMinutes,
  shiftBreak,
  shiftEnd,
  shiftGrace,
  shiftName,
  shiftStart,
}: ShiftTimelinePreviewProps) {
  const { t } = useTranslation();

  const startMin = timeToMinutes(shiftStart);
  const endMin = timeToMinutes(shiftEnd);

  const shiftDurationMin = calculateShiftDuration(shiftStart, shiftEnd, startMin, endMin);
  const breakDurationMin = calculateBreakDuration(isFixedBreak, breakStart, breakEnd, shiftBreak);

  // Active working minutes
  const workingMinutes = Math.max(0, shiftDurationMin - (isAutoDeduct || isFixedBreak ? breakDurationMin : 0));
  const workingHours = (workingMinutes / 60).toFixed(2);

  const isCrossMidnight = endMin < startMin;

  return (
    <div className="bg-slate-50 dark:bg-slate-800 p-8 rounded-[32px] border border-slate-100 dark:border-slate-700/80 space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Clock className="text-primary-500" size={16} />
          <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest">
            {t("shifts.modal.preview_title", "Xem trước sơ đồ ca")}
          </span>
        </div>
        <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider font-mono">
          24H TIMELINE
        </span>
      </div>

      {/* Visual Timeline Bar */}
      <div className="space-y-4">
        <TimelineBar
          allowEarlyOt={allowEarlyOt}
          breakEnd={breakEnd}
          breakStart={breakStart}
          endMin={endMin}
          isCrossMidnight={isCrossMidnight}
          isFixedBreak={isFixedBreak}
          maxEarlyOtMinutes={maxEarlyOtMinutes}
          shiftEnd={shiftEnd}
          shiftStart={shiftStart}
          startMin={startMin}
        />

        {/* Timeline Axis Labels */}
        <div className="flex justify-between px-1 text-[9px] font-extrabold text-slate-400 dark:text-slate-500">
          <span>00:00</span>
          <span>04:00</span>
          <span>08:00</span>
          <span>12:00</span>
          <span>16:00</span>
          <span>20:00</span>
          <span>24:00</span>
        </div>
      </div>

      <MetricsGrid
        shiftDurationMin={shiftDurationMin}
        t={t}
        workingHours={workingHours}
      />

      {/* Timing Details List */}
      <ShiftDetailsList
        allowEarlyOt={allowEarlyOt}
        breakEnd={breakEnd}
        breakStart={breakStart}
        isAutoDeduct={isAutoDeduct}
        isBreakOvertime={isBreakOvertime}
        isFixedBreak={isFixedBreak}
        maxEarlyOtMinutes={maxEarlyOtMinutes}
        shiftBreak={shiftBreak}
        shiftEnd={shiftEnd}
        shiftGrace={shiftGrace}
        shiftName={shiftName}
        shiftStart={shiftStart}
        t={t}
      />
    </div>
  );
}
