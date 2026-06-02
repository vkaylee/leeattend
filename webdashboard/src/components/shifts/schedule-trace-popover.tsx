import type { TFunction } from "i18next";

import * as Popover from '@radix-ui/react-popover';
import { Info, CheckCircle2, AlertCircle, Shield, ChevronDown, ChevronRight, Clock, Coffee } from 'lucide-react';
import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

import { ScheduleTrace } from '../../lib/api/types';

interface ScheduleTracePopoverProps {
  endTime?: string | null;
  shiftName?: string | null;
  shiftSymbol?: string | null;
  source: string;
  startTime?: string | null;
  traces: ScheduleTrace[];
}

function AppliedShiftCard({
  shiftName,
  startTime,
  endTime,
  shiftSymbol,
  t,
}: {
  shiftName?: string | null;
  startTime?: string | null;
  endTime?: string | null;
  shiftSymbol?: string | null;
  t: TFunction;
}) {
  if (!shiftName) {
    return (
      <div className="p-3.5 bg-gradient-to-br from-slate-500/5 to-slate-600/5 dark:from-slate-500/10 dark:to-slate-600/10 border border-slate-200/40 dark:border-slate-800/40 rounded-2xl flex items-center justify-between shadow-sm backdrop-blur-sm">
        <div className="flex items-center gap-2.5">
          <div className="p-1.5 bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400 rounded-lg">
            <Coffee size={14} />
          </div>
          <div className="flex flex-col">
            <span className="text-[9px] font-extrabold uppercase tracking-widest text-slate-400 dark:text-slate-500 leading-none mb-1">
              {t("shifts.traceability.applied_shift", "Ca áp dụng")}
            </span>
            <span className="text-xs font-bold text-slate-500 dark:text-slate-450 leading-tight">
              {t("shifts.off_day", "Ngày nghỉ")}
            </span>
          </div>
        </div>
        <span className="text-[8px] font-extrabold uppercase tracking-widest text-slate-400 dark:text-slate-500 px-1.5 py-0.5 bg-slate-100 dark:bg-slate-800 rounded">
          OFF
        </span>
      </div>
    );
  }

  return (
    <div className="p-3.5 bg-gradient-to-br from-blue-500/10 to-indigo-500/10 dark:from-blue-500/20 dark:to-indigo-500/20 border border-blue-100/50 dark:border-blue-800/50 rounded-2xl space-y-2 shadow-sm backdrop-blur-sm">
      <div className="flex items-center justify-between">
        <span className="text-[9px] font-extrabold uppercase tracking-widest text-blue-600 dark:text-blue-400">
          {t("shifts.traceability.applied_shift", "Ca áp dụng")}
        </span>
        {shiftSymbol ? (
          <span className="text-[9px] font-extrabold px-1.5 py-0.5 rounded bg-blue-100 dark:bg-blue-900/60 text-blue-700 dark:text-blue-300">
            {shiftSymbol}
          </span>
        ) : null}
      </div>
      <div className="flex items-center gap-2.5">
        <div className="p-1.5 bg-blue-500/15 text-blue-600 dark:text-blue-400 rounded-lg">
          <Clock size={14} />
        </div>
        <div className="flex flex-col">
          <span className="text-xs font-extrabold text-slate-800 dark:text-slate-100 leading-tight">
            {shiftName}
          </span>
          {startTime || endTime ? (
            <span className="text-[10px] font-semibold text-slate-500 dark:text-slate-450 mt-0.5 font-mono">
              {startTime?.substring(0, 5) || "—"} - {endTime?.substring(0, 5) || "—"}
            </span>
          ) : null}
        </div>
      </div>
    </div>
  );
}

function AppliedRuleSection({ appliedRule, t }: { appliedRule: ScheduleTrace; t: TFunction }) {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <span className="text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
          {t("shifts.traceability.winner")}
        </span>
        <span className="text-[10px] font-bold px-2 py-0.5 rounded-full bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border border-emerald-100 dark:border-emerald-800 uppercase">
          {t("shifts.traceability.applied")}
        </span>
      </div>
      <div className="p-3 bg-emerald-50 dark:bg-emerald-950/30 border border-emerald-100 dark:border-emerald-800/40 rounded-xl space-y-1 hover:border-emerald-250 dark:hover:border-emerald-800/80 transition-colors">
        <div className="flex items-center gap-2 text-emerald-700 dark:text-emerald-300">
          <CheckCircle2 size={14} />
          <span className="text-xs font-bold">{appliedRule.rule_name}</span>
        </div>
        {appliedRule.reason ? (
          <Typography className="text-[10px] text-emerald-600 dark:text-emerald-300/80 leading-relaxed pl-5 font-medium" variant="p">
            {appliedRule.reason}
          </Typography>
        ) : null}
        <div className="pt-1 flex items-center gap-3 pl-5">
          <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase">
            {t("shifts.traceability.priority")}: {appliedRule.priority}
          </span>
        </div>
        
        {/* Smart Hint */}
        <div className="mt-3 pt-3 border-t border-emerald-100 dark:border-emerald-800/50">
          <Typography className="text-[9px] text-emerald-700 dark:text-emerald-300 font-medium leading-relaxed flex gap-2" variant="p">
            <span className="shrink-0 text-emerald-500">💡</span>
            <span>
              {appliedRule.priority < 100 ? 
                t("shifts.traceability.override_hint") :
                t("shifts.traceability.highest_priority_hint")
              }
            </span>
          </Typography>
        </div>
      </div>
    </div>
  );
}

function OverriddenRulesSection({ overriddenRules, showOverridden, setShowOverridden, t }: { overriddenRules: ScheduleTrace[]; setShowOverridden: (v: boolean) => void; showOverridden: boolean; t: TFunction }) {
  if (overriddenRules.length === 0) return null;

  return (
    <div className="space-y-2 pt-2 border-t border-slate-100 dark:border-slate-800">
      <Button className="flex items-center justify-between w-full group py-1.5 hover:bg-slate-50/50 dark:hover:bg-slate-850/50 rounded-lg px-1.5 transition-all" 
        onClick={() => { setShowOverridden(!showOverridden); }}
        variant="ghost"
      >
        <span className="text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 group-hover:text-slate-650 dark:group-hover:text-slate-200 transition-colors">
          {t("shifts.traceability.show_overridden")} ({overriddenRules.length})
        </span>
        {showOverridden ? (
          <ChevronDown className="text-slate-500 dark:text-slate-400 group-hover:text-slate-650 dark:group-hover:text-slate-250 transition-colors" size={14} />
        ) : (
          <ChevronRight className="text-slate-500 dark:text-slate-400 group-hover:text-slate-650 dark:group-hover:text-slate-250 transition-colors" size={14} />
        )}
      </Button>

      {showOverridden ? (
        <div className="space-y-2 animate-in slide-in-from-top-1.5 duration-200 ease-out">
          {overriddenRules.map((rule, index) => (
            <div className="p-3 bg-slate-50 dark:bg-slate-800/40 border border-slate-100 dark:border-slate-800/60 rounded-xl space-y-1 opacity-70 hover:opacity-100 hover:border-slate-200 dark:hover:border-slate-700 transition-all" key={index}>
              <div className="flex items-center gap-2 text-slate-600 dark:text-slate-300">
                <AlertCircle size={14} />
                <span className="text-xs font-bold">{rule.rule_name}</span>
              </div>
              {rule.reason ? (
                <Typography className="text-[10px] text-slate-500 dark:text-slate-450 leading-relaxed pl-5 italic" variant="p">
                  {rule.reason}
                </Typography>
              ) : null}
              <div className="pt-1 flex items-center gap-3 pl-5">
                <span className="text-[9px] font-bold text-slate-500 dark:text-slate-450 uppercase">
                  {t("shifts.traceability.priority")}: {rule.priority}
                </span>
              </div>
            </div>
          ))}
        </div>
      ) : null}
    </div>
  );
}

export const ScheduleTracePopover: React.FC<ScheduleTracePopoverProps> = ({
  traces,
  source,
  shiftName,
  startTime,
  endTime,
  shiftSymbol,
}) => {
  const { t } = useTranslation();
  const [showOverridden, setShowOverridden] = useState(false);

  const appliedRule = traces.find(t => t.status === 'applied');
  const overriddenRules = traces.filter(t => t.status === 'overridden');

  if (traces.length === 0) return null;

  return (
    <Popover.Root>
      <Popover.Trigger 
        className="w-[18px] h-[18px] flex items-center justify-center rounded-full hover:bg-slate-100 dark:hover:bg-slate-750 text-slate-500 dark:text-slate-400 transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary-500 cursor-pointer"
        data-testid="schedule-trace-trigger" 
        onClick={(e: React.MouseEvent) => { e.stopPropagation(); }}
      >
        <Info size={11} />
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content 
          className="z-[10000] w-80 bg-white/90 dark:bg-slate-900/90 backdrop-blur-md rounded-[24px] shadow-2xl border border-slate-100/80 dark:border-slate-800/80 p-4 animate-in fade-in zoom-in-95 duration-200"
          onClick={(e: React.MouseEvent) => { e.stopPropagation(); }}
          sideOffset={5}
        >
          <div className="flex flex-col gap-4">
            <div className="flex items-center gap-2 border-b border-slate-100 dark:border-slate-800 pb-3">
              <div className="p-2 bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 rounded-xl">
                <Shield size={16} />
              </div>
              <div>
                <Typography className="text-sm font-extrabold text-slate-800 dark:text-slate-200 uppercase tracking-tight" variant="h3">
                  {t("shifts.traceability.title")}
                </Typography>
                <Typography className="text-[10px] text-slate-400 dark:text-slate-500 font-bold uppercase tracking-widest" variant="p">
                  {t("shifts.traceability.source")}: {source}
                </Typography>
              </div>
            </div>

            {/* Resolved Shift Card */}
            <AppliedShiftCard 
              endTime={endTime}
              shiftName={shiftName}
              shiftSymbol={shiftSymbol}
              startTime={startTime}
              t={t}
            />

            {appliedRule ? <AppliedRuleSection appliedRule={appliedRule} t={t} /> : null}
            <OverriddenRulesSection overriddenRules={overriddenRules} setShowOverridden={setShowOverridden} showOverridden={showOverridden} t={t} />
          </div>
          <Popover.Arrow className="fill-white dark:fill-slate-900" />
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
};
