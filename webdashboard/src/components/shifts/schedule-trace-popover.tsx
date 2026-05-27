import type { TFunction } from "i18next";

import * as Popover from '@radix-ui/react-popover';
import { Info, CheckCircle2, AlertCircle, Shield, ChevronDown, ChevronRight } from 'lucide-react';
import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

import { ScheduleTrace } from '../../lib/api/types';

interface ScheduleTracePopoverProps {
  source: string;
  traces: ScheduleTrace[];
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
      <div className="p-3 bg-emerald-50 dark:bg-emerald-950/30 border border-emerald-100 dark:border-emerald-800 rounded-xl space-y-1">
        <div className="flex items-center gap-2 text-emerald-700 dark:text-emerald-300">
          <CheckCircle2 size={14} />
          <span className="text-xs font-bold">{appliedRule.rule_name}</span>
        </div>
        {appliedRule.reason ? (
          <Typography className="text-[10px] text-emerald-600 dark:text-emerald-300/80 leading-relaxed pl-5" variant="p">
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
    <div className="space-y-2 pt-2 border-t border-slate-50 dark:border-slate-800">
      <Button className="flex items-center justify-between w-full group" 
        onClick={() => { setShowOverridden(!showOverridden); }}
        variant="ghost"
      >
        <span className="text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 group-hover:text-slate-600 dark:hover:text-slate-300 dark:text-slate-600 transition-colors">
          {t("shifts.traceability.show_overridden")} ({overriddenRules.length})
        </span>
        {showOverridden ? <ChevronDown className="text-slate-500 dark:text-slate-400" size={14} /> : <ChevronRight className="text-slate-500 dark:text-slate-400" size={14} />}
      </Button>

      {showOverridden ? (
        <div className="space-y-2 animate-in slide-in-from-top-1 duration-200">
          {overriddenRules.map((rule, index) => (
            <div className="p-3 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-xl space-y-1 opacity-70" key={index}>
              <div className="flex items-center gap-2 text-slate-600 dark:text-slate-300">
                <AlertCircle size={14} />
                <span className="text-xs font-bold">{rule.rule_name}</span>
              </div>
              {rule.reason ? (
                <Typography className="text-[10px] text-slate-500 dark:text-slate-400 leading-relaxed pl-5 italic" variant="p">
                  {rule.reason}
                </Typography>
              ) : null}
              <div className="pt-1 flex items-center gap-3 pl-5">
                <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase">
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

export const ScheduleTracePopover: React.FC<ScheduleTracePopoverProps> = ({ traces, source }) => {
  const { t } = useTranslation();
  const [showOverridden, setShowOverridden] = useState(false);

  const appliedRule = traces.find(t => t.status === 'applied');
  const overriddenRules = traces.filter(t => t.status === 'overridden');

  if (traces.length === 0) return null;

  return (
    <Popover.Root>
      <Popover.Trigger asChild>
        <Button className="p-1 rounded-full hover:bg-slate-100 dark:hover:bg-slate-700 text-slate-500 dark:text-slate-400 transition-colors" 
          onClick={(e: React.MouseEvent) => { e.stopPropagation(); }}
          variant="ghost"
        >
          <Info size={14} />
        </Button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content 
          className="z-50 w-80 bg-white dark:bg-slate-900 rounded-2xl shadow-xl border border-slate-100 dark:border-slate-800 p-4 animate-in fade-in zoom-in-95 duration-200"
          onClick={(e: React.MouseEvent) => { e.stopPropagation(); }}
          sideOffset={5}
        >
          <div className="flex flex-col gap-4">
            <div className="flex items-center gap-2 border-b border-slate-50 dark:border-slate-800 pb-3">
              <div className="p-2 bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 rounded-lg">
                <Shield size={16} />
              </div>
              <div>
                <Typography className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h3">
                  {t("shifts.traceability.title")}
                </Typography>
                <Typography className="text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase tracking-widest" variant="p">
                  {t("shifts.traceability.source")}: {source}
                </Typography>
              </div>
            </div>

            {appliedRule ? <AppliedRuleSection appliedRule={appliedRule} t={t} /> : null}
            <OverriddenRulesSection overriddenRules={overriddenRules} setShowOverridden={setShowOverridden} showOverridden={showOverridden} t={t} />
          </div>
          <Popover.Arrow className="fill-white" />
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
};
