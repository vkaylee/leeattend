import React, { useState } from 'react';
import * as Popover from '@radix-ui/react-popover';
import { Info, CheckCircle2, AlertCircle, Shield, Building2, User, Zap, ChevronDown, ChevronRight } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { ScheduleTrace } from '../../lib/api/types';
import { cn } from '../../lib/utils';

interface ScheduleTracePopoverProps {
  traces: ScheduleTrace[];
  source: string;
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
        <button 
          onClick={(e) => e.stopPropagation()}
          className="p-1 rounded-full hover:bg-slate-100 text-slate-500 transition-colors"
        >
          <Info size={14} />
        </button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content 
          className="z-50 w-80 bg-white rounded-2xl shadow-xl border border-slate-100 p-4 animate-in fade-in zoom-in-95 duration-200"
          sideOffset={5}
          onClick={(e) => e.stopPropagation()}
        >
          <div className="flex flex-col gap-4">
            <div className="flex items-center gap-2 border-b border-slate-50 pb-3">
              <div className="p-2 bg-blue-50 text-blue-600 rounded-lg">
                <Shield size={16} />
              </div>
              <div>
                <h3 className="text-sm font-bold text-slate-700 uppercase tracking-tight">
                  {t("shifts.traceability.title")}
                </h3>
                <p className="text-[10px] text-slate-500 font-bold uppercase tracking-widest">
                  {t("shifts.traceability.source")}: {source}
                </p>
              </div>
            </div>

            {/* Applied Rule */}
            {appliedRule && (
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-[10px] font-bold uppercase tracking-widest text-slate-500">
                    {t("shifts.traceability.winner")}
                  </span>
                  <span className="text-[10px] font-bold px-2 py-0.5 rounded-full bg-emerald-50 text-emerald-600 border border-emerald-100 uppercase">
                    {t("shifts.traceability.applied")}
                  </span>
                </div>
                <div className="p-3 bg-emerald-50/30 border border-emerald-100 rounded-xl space-y-1">
                  <div className="flex items-center gap-2 text-emerald-700">
                    <CheckCircle2 size={14} />
                    <span className="text-xs font-bold">{appliedRule.rule_name}</span>
                  </div>
                  {appliedRule.reason && (
                    <p className="text-[10px] text-emerald-600/80 leading-relaxed pl-5">
                      {appliedRule.reason}
                    </p>
                  )}
                  <div className="pt-1 flex items-center gap-3 pl-5">
                    <span className="text-[9px] font-bold text-slate-500 uppercase">
                      {t("shifts.traceability.priority")}: {appliedRule.priority}
                    </span>
                  </div>
                  
                  {/* Smart Hint */}
                  <div className="mt-3 pt-3 border-t border-emerald-100/50">
                    <p className="text-[9px] text-emerald-700 font-medium leading-relaxed flex gap-2">
                      <span className="shrink-0 text-emerald-500">💡</span>
                      <span>
                        {appliedRule.priority < 100 ? 
                          t("shifts.traceability.override_hint") :
                          t("shifts.traceability.highest_priority_hint")
                        }
                      </span>
                    </p>
                  </div>
                </div>
              </div>
            )}

            {/* Overridden Rules Toggle */}
            {overriddenRules.length > 0 && (
              <div className="space-y-2 pt-2 border-t border-slate-50">
                <button 
                  onClick={() => setShowOverridden(!showOverridden)}
                  className="flex items-center justify-between w-full group"
                >
                  <span className="text-[10px] font-bold uppercase tracking-widest text-slate-500 group-hover:text-slate-600 transition-colors">
                    {t("shifts.traceability.show_overridden")} ({overriddenRules.length})
                  </span>
                  {showOverridden ? <ChevronDown size={14} className="text-slate-500" /> : <ChevronRight size={14} className="text-slate-500" />}
                </button>

                {showOverridden && (
                  <div className="space-y-2 animate-in slide-in-from-top-1 duration-200">
                    {overriddenRules.map((rule, idx) => (
                      <div key={idx} className="p-3 bg-slate-50 border border-slate-100 rounded-xl space-y-1 opacity-70">
                        <div className="flex items-center gap-2 text-slate-600">
                          <AlertCircle size={14} />
                          <span className="text-xs font-bold">{rule.rule_name}</span>
                        </div>
                        {rule.reason && (
                          <p className="text-[10px] text-slate-500 leading-relaxed pl-5 italic">
                            {rule.reason}
                          </p>
                        )}
                        <div className="pt-1 flex items-center gap-3 pl-5">
                          <span className="text-[9px] font-bold text-slate-500 uppercase">
                            {t("shifts.traceability.priority")}: {rule.priority}
                          </span>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>
          <Popover.Arrow className="fill-white" />
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
};
