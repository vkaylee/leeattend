/**
 * PriorityChain — visual timeline explaining how a work shift was resolved.
 *
 * Renders the 5-tier resolution chain (Roster → Pattern → Rules → Fixed → None)
 * with the applied tier highlighted and overridden tiers greyed-out.
 *
 * ## Architecture note (why Dynamic Rules replace Dept Pattern / Dept Fixed)
 *
 * Legacy HR platforms (Deputy, BambooHR) typically model 6 tiers:
 *   P1 Roster → P2 Individual Pattern → P3 Dynamic Rules →
 *   P4 Department Pattern → P5 Fixed → P6 Department Fixed
 *
 * LeeAttend intentionally collapses P4 & P6 into P3. The RuleEvaluator
 * accepts department_id, tags, location_id, and employer_id conditions,
 * making dedicated department tiers redundant.  See backend
 * `shift_resolution.rs` module docs for the full rationale.
 */

import type { TFunction } from "i18next";

import { ArrowDown, Ban, CheckCircle2, ChevronRight, Clock, Layers, Shield } from 'lucide-react';
import React from 'react';

import { Typography } from "@/components/ui/typography";

import type { ScheduleTrace } from '../../lib/api/types';

// ---------------------------------------------------------------------------
// Tier map — single source of truth for display order & labels
// ---------------------------------------------------------------------------

interface TierMeta {
  /** i18n key suffix (combined with "shifts.traceability." prefix) */
  i18nKey: string;
  /** backend source labels that map to this tier */
  sources: string[];
  /** numeric priority (100 = highest) */
  priority: number;
  /** lucide icon component */
  Icon: React.FC<{ size?: number; className?: string }>;
}

const TIER_MAP: readonly TierMeta[] = [
  { i18nKey: 'p100_manual', sources: ['roster'],          priority: 100, Icon: Shield },
  { i18nKey: 'p80_pattern', sources: ['pattern'],         priority: 80,  Icon: Clock },
  { i18nKey: 'p60_rules',   sources: ['assignment_rule', 'rule'], priority: 60, Icon: Layers },
  { i18nKey: 'p20_fixed',   sources: ['fixed'],           priority: 20,  Icon: CheckCircle2 },
] as const;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function findAppliedPriority(source: string): number | null {
  const tier = TIER_MAP.find(t => t.sources.includes(source));
  return tier?.priority ?? null;
}

// ---------------------------------------------------------------------------
// Horizontal badge chain — compact visual for narrow layouts
// ---------------------------------------------------------------------------

interface HorizontalBadgeChainProps {
  source: string;
  t: TFunction;
}

export function HorizontalBadgeChain({ source, t }: HorizontalBadgeChainProps) {
  const isDayOff = source === 'none' || !source;

  return (
    <div className="flex flex-wrap items-center gap-1">
      {TIER_MAP.map((tier, idx) => {
        const isApplied = tier.sources.includes(source);
        return (
          <React.Fragment key={tier.i18nKey}>
            <div
              className={`
                flex items-center gap-1 px-2 py-1 rounded-lg text-[9px] font-extrabold uppercase tracking-wider transition-all
                ${isApplied
                  ? 'bg-emerald-100 dark:bg-emerald-900/40 text-emerald-700 dark:text-emerald-300 ring-1 ring-emerald-300 dark:ring-emerald-700 shadow-sm'
                  : 'bg-slate-50 dark:bg-slate-800/40 text-slate-400 dark:text-slate-500'}
              `}
            >
              <tier.Icon size={10} />
              <span>{t(`shifts.traceability.${tier.i18nKey}`)}</span>
              {isApplied && <span className="text-[8px]">✅</span>}
            </div>
            {idx < TIER_MAP.length - 1 && (
              <ChevronRight size={10} className="text-slate-300 dark:text-slate-600 shrink-0" />
            )}
          </React.Fragment>
        );
      })}

      {isDayOff && (
        <>
          <ChevronRight size={10} className="text-slate-300 dark:text-slate-600 shrink-0" />
          <div className="flex items-center gap-1 px-2 py-1 rounded-lg text-[9px] font-extrabold uppercase tracking-wider bg-slate-100 dark:bg-slate-800 text-slate-500">
            <Ban size={10} />
            <span>{t('shifts.traceability.p0_none')}</span>
          </div>
        </>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Vertical priority chain — detail view inside ScheduleTracePopover
// ---------------------------------------------------------------------------

interface PriorityChainVerticalProps {
  source: string;
  appliedRule: ScheduleTrace | undefined;
  t: TFunction;
}

export function PriorityChainVertical({ source, appliedRule, t }: PriorityChainVerticalProps) {
  const appliedPriority = appliedRule?.priority ?? findAppliedPriority(source);

  // When source === 'none', every tier was skipped — all tiers show as "above applied"
  const effectiveAppliedPriority = appliedPriority ?? 0;

  return (
    <div className="space-y-1.5">
      {/* Section title */}
      <div className="flex items-center justify-between">
        <span className="text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
          {t('shifts.traceability.priority_hierarchy')}
        </span>
        <span className="text-[8px] font-medium text-slate-400 dark:text-slate-500">
          {t('shifts.traceability.hierarchy_explanation')}
        </span>
      </div>

      {/* Vertical tree */}
      <div className="space-y-1">
        {TIER_MAP.map((tier) => {
          const isApplied = tier.sources.includes(source);
          const isSkipped = !isApplied && tier.priority > effectiveAppliedPriority;
          const isBelow = !isApplied && tier.priority < effectiveAppliedPriority;

          return (
            <div className="flex items-center gap-2" key={tier.i18nKey}>
              {/* Status icon */}
              <div className={`w-5 h-5 rounded-full flex items-center justify-center shrink-0 ${
                isApplied
                  ? 'bg-emerald-100 dark:bg-emerald-900/40 text-emerald-600 dark:text-emerald-300'
                  : 'bg-slate-50 dark:bg-slate-800/40 text-slate-300 dark:text-slate-600'
              }`}>
                {isApplied ? (
                  <CheckCircle2 size={12} />
                ) : (
                  <ArrowDown size={10} className={isSkipped ? 'opacity-40' : 'opacity-20'} />
                )}
              </div>

              {/* Tier label */}
              <div className={`flex items-center gap-1.5 text-[10px] font-bold flex-1 ${
                isApplied
                  ? 'text-emerald-700 dark:text-emerald-300'
                  : isSkipped
                    ? 'text-slate-400 dark:text-slate-500'
                    : 'text-slate-400 dark:text-slate-500'
              }`}>
                <tier.Icon size={11} />
                <span className="uppercase tracking-wide">{t(`shifts.traceability.${tier.i18nKey}`)}</span>
              </div>

              {/* Priority badge */}
              <span className={`text-[8px] font-extrabold px-1.5 py-0.5 rounded uppercase ${
                isApplied
                  ? 'bg-emerald-100 dark:bg-emerald-900/30 text-emerald-700 dark:text-emerald-300'
                  : 'bg-slate-50 dark:bg-slate-800 text-slate-400 dark:text-slate-500'
              }`}>
                P{tier.priority}
              </span>

              {/* Status label */}
              {isApplied && (
                <span className="text-[8px] font-extrabold px-1.5 py-0.5 rounded-full bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border border-emerald-100 dark:border-emerald-800 uppercase tracking-wider">
                  {t('shifts.traceability.applied')}
                </span>
              )}
              {isSkipped && (
                <span className="text-[8px] font-extrabold text-slate-400 dark:text-slate-500 uppercase tracking-wider">
                  skip
                </span>
              )}
              {isBelow && (
                <span className="text-[8px] font-extrabold text-slate-300 dark:text-slate-600 uppercase tracking-wider">
                  —
                </span>
              )}
            </div>
          );
        })}

        {/* Day-off indicator */}
        {(source === 'none' || !source) && (
          <div className="flex items-center gap-2">
            <div className="w-5 h-5 rounded-full flex items-center justify-center shrink-0 bg-slate-100 dark:bg-slate-800 text-slate-400">
              <Ban size={10} />
            </div>
            <div className="flex items-center gap-1.5 text-[10px] font-bold flex-1 text-slate-500">
              <Ban size={11} />
              <span className="uppercase tracking-wide">{t('shifts.traceability.p0_none')}</span>
            </div>
            <span className="text-[8px] font-extrabold px-1.5 py-0.5 rounded bg-slate-100 dark:bg-slate-800 text-slate-500 uppercase">
              OFF
            </span>
          </div>
        )}
      </div>

      {/* Rule reason (if applied rule has explanation) */}
      {appliedRule?.reason && (
        <div className="pt-2 mt-2 border-t border-emerald-100 dark:border-emerald-800/40">
          <Typography className="text-[9px] text-emerald-700 dark:text-emerald-300/80 font-medium leading-relaxed flex gap-1.5" variant="p">
            <span className="shrink-0">▸</span>
            <span className="italic">{appliedRule.reason}</span>
          </Typography>
        </div>
      )}

      {/* Educational note — why no Dept Pattern / Dept Fixed */}
      {(source === 'assignment_rule' || source === 'rule') && (
        <div className="pt-2 mt-2 border-t border-blue-100 dark:border-blue-900/40">
          <Typography className="text-[9px] text-blue-700 dark:text-blue-300 font-medium leading-relaxed flex gap-1.5" variant="p">
            <span className="shrink-0">💡</span>
            <span>{t('shifts.traceability.tier_subsumed_note')}</span>
          </Typography>
        </div>
      )}
    </div>
  );
}
