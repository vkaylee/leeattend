import { Sparkles, CheckCircle2, Target, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { IncentiveTier, PayrollIncentivePolicyResponse, RuleCondition } from "@/lib/api/types";

// ── Tier Rules Renderer ──────────────────────────────────────────────

function TierRulesRenderer({ tier }: { tier: IncentiveTier }) {
  if (!tier.rule_config) return <span className="text-[7px] font-bold text-slate-500 dark:text-slate-400 uppercase italic">Default</span>;

  if (Array.isArray(tier.rule_config)) {
    if (tier.rule_config.length === 0) return <span className="text-[7px] font-bold text-slate-500 dark:text-slate-400 uppercase italic">Default</span>;
    return (
      <>
        {tier.rule_config.map((r: RuleCondition, i) => (
          <span className="text-[7px] font-bold text-slate-500 dark:text-slate-400 uppercase" key={i}>
            {r.metric.replace(/_/g, ' ')} {r.operator} {r.value}
          </span>
        ))}
      </>
    );
  }

  // Legacy flat object
  const config = tier.rule_config as Record<string, string | number | undefined | null>;
  const rules: string[] = [];
  if (config.max_late_early_count != null) rules.push(`Late < ${String(config.max_late_early_count)}`);
  if (config.max_unpaid_leave_days != null) rules.push(`Unpaid < ${String(config.max_unpaid_leave_days)}d`);
  if (config.max_sick_leave_days != null) rules.push(`Sick < ${String(config.max_sick_leave_days)}d`);
  if (config.max_total_leave_days != null) rules.push(`Total < ${String(config.max_total_leave_days)}d`);

  if (rules.length === 0) return <span className="text-[7px] font-bold text-slate-500 dark:text-slate-400 uppercase italic">Default</span>;
  return (
    <>
      {rules.map((r, i) => (
        <span className="text-[7px] font-bold text-slate-500 dark:text-slate-400 uppercase" key={i}>{r}</span>
      ))}
    </>
  );
}

// ── Bonus Policy Card ────────────────────────────────────────────────

interface BonusPolicyCardProps {
  onClone: (id: string) => void;
  onDelete: (id: string) => void;
  policy: PayrollIncentivePolicyResponse;
}

export function BonusPolicyCard({ policy, onClone, onDelete }: BonusPolicyCardProps) {
  const { t } = useTranslation();

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] p-5 sm:p-8 shadow-sm hover:shadow-md transition-all group relative overflow-hidden">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 relative z-10">
        <div className="flex items-center gap-6">
          <div className="w-16 h-16 rounded-[1.5rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-inner border border-primary-100 dark:border-primary-800/50">
            <Target size={32} />
          </div>
          <div>
            <Typography className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h3">{policy.name}</Typography>
            <div className="flex items-center gap-4 mt-1.5">
              <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest bg-slate-50 dark:bg-slate-800 px-3 py-1 rounded-lg border border-slate-100 dark:border-slate-800">
                BASE: {new Intl.NumberFormat('vi-VN').format(policy.base_amount)} VND
              </span>
              {policy.is_active ? (
                <span className="flex items-center gap-1.5 text-[10px] font-bold text-emerald-600 dark:text-emerald-300 uppercase tracking-widest">
                  <CheckCircle2 size={12} /> {t("common.active")}
                </span>
              ) : (
                <span className="flex items-center gap-1.5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {t("common.inactive", { defaultValue: "INACTIVE" })}
                </span>
              )}
              {!!policy.effective_from && (
                <span className="text-[9px] font-bold text-slate-400 uppercase tracking-widest px-2 py-0.5 rounded bg-slate-100 dark:bg-slate-800">
                  {policy.effective_from}
                </span>
              )}
            </div>
          </div>
        </div>

        <div className="flex flex-wrap gap-3">
          {[...policy.tiers].sort((a: IncentiveTier, b: IncentiveTier) => a.priority - b.priority).map((tier: IncentiveTier) => (
            <div className="bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 flex flex-col items-center min-w-[140px] shadow-inner" key={tier.id}>
              <span className="text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2">Tier {tier.priority}</span>
              <span className="text-sm font-bold text-primary-600 dark:text-primary-400 font-mono tracking-tighter">
                +{new Intl.NumberFormat('vi-VN').format(tier.payout_amount)}
              </span>
              <div className="mt-2 flex flex-col items-center gap-1 opacity-60">
                <TierRulesRenderer tier={tier} />
              </div>
            </div>
          ))}
        </div>

        <div className="flex flex-col gap-2">
          <Button className="p-3 rounded-2xl text-slate-400 dark:text-slate-500 hover:text-primary-500 hover:bg-primary-50 dark:hover:bg-primary-900/50 transition-all active:scale-90 flex items-center justify-center"
            onClick={() => { onClone(policy.id); }}
            title="Clone Policy"
            variant="ghost"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><rect height="13" rx="2" ry="2" width="13" x="9" y="9"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
          </Button>
          <Button className="p-3 rounded-2xl text-slate-400 dark:text-slate-500 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-950 transition-all active:scale-90 flex items-center justify-center"
            onClick={() => { onDelete(policy.id); }}
            title="Delete Policy"
            variant="ghost"
          >
            <Trash2 size={20} />
          </Button>
        </div>
      </div>
      <Sparkles className="absolute -right-8 -bottom-8 w-40 h-40 text-primary-500 opacity-[0.03] -rotate-12 group-hover:rotate-0 transition-transform duration-1000 pointer-events-none" />
    </div>
  );
}
