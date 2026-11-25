import { ChevronDown, ChevronRight, Play } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import type { WorkPolicyBatchSimulationResponse } from "@/lib/api/payroll";

interface WorkPolicySimulationPanelProps {
  canSimulate: boolean;
  defaultRelationshipIds: string[];
  isRunning: boolean;
  onRunSimulation: (year: number, month: number, relationshipIds: string[]) => void;
  relationshipLabels: Map<string, string>;
  result: WorkPolicyBatchSimulationResponse | null;
}

export function WorkPolicySimulationPanel({
  canSimulate,
  defaultRelationshipIds,
  isRunning,
  onRunSimulation,
  relationshipLabels,
  result,
}: WorkPolicySimulationPanelProps) {
  const { t } = useTranslation();
  const now = new Date();
  const [year, setYear] = useState(now.getUTCFullYear());
  const [month, setMonth] = useState(now.getUTCMonth() + 1);
  const [expandedRel, setExpandedRel] = useState<string | null>(null);
  const [mountedAt] = useState(() => Date.now());

  const isExpired = result ? new Date(result.expires_at).getTime() < mountedAt : false;

  return (
    <div className="space-y-4 rounded-2xl border border-slate-200 bg-white p-6 dark:border-slate-800 dark:bg-slate-900" data-testid="work-policy-simulation-panel">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <Typography className="text-sm font-bold text-slate-800 dark:text-slate-100" variant="h3">
            {t("payroll.work_policies.simulation_title", "Batch Impact Simulation (Read-Only)")}
          </Typography>
          <p className="text-xs text-slate-500 dark:text-slate-400">
            {t("payroll.work_policies.simulation_desc", "Compares current published policy against this version on identical attendance & salary inputs without modifying payroll data.")}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <input
            aria-label={t("common.year", "Year")}
            className="w-24 rounded-xl border border-slate-200 bg-white px-3 py-1.5 text-xs dark:border-slate-700 dark:bg-slate-950"
            max={2100}
            min={2000}
            onChange={(e) => { setYear(Number(e.target.value) || now.getUTCFullYear()); }}
            type="number"
            value={year}
          />
          <input
            aria-label={t("common.month", "Month")}
            className="w-16 rounded-xl border border-slate-200 bg-white px-3 py-1.5 text-xs dark:border-slate-700 dark:bg-slate-950"
            max={12}
            min={1}
            onChange={(e) => { setMonth(Number(e.target.value) || 1); }}
            type="number"
            value={month}
          />
          <Button
            data-testid="wp-run-simulation-btn"
            disabled={!canSimulate || isRunning || defaultRelationshipIds.length === 0}
            onClick={() => { onRunSimulation(year, month, defaultRelationshipIds.slice(0, 100)); }}
            size="sm"
            variant="primary"
          >
            <Play className="mr-1 h-3.5 w-3.5" />
            {isRunning ? t("common.loading", "Running...") : t("payroll.work_policies.run_simulation", "Run Simulation")}
          </Button>
        </div>
      </div>

      {!canSimulate ? (
        <p className="text-xs text-amber-600 dark:text-amber-400">
          {t("payroll.work_policies.simulation_permission_hint", "Requires payroll:sensitive:read permission to run impact simulations.")}
        </p>
      ) : null}

      {result ? (
        <div className="space-y-3" data-testid="wp-simulation-results">
          <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
            <div className="rounded-xl bg-slate-50 p-3 dark:bg-slate-800/50">
              <div className="text-[11px] text-slate-500">{t("payroll.work_policies.sim_total", "Total")}</div>
              <div className="font-mono text-lg font-bold text-slate-800 dark:text-slate-100">{result.total}</div>
            </div>
            <div className="rounded-xl bg-slate-50 p-3 dark:bg-slate-800/50">
              <div className="text-[11px] text-slate-500">{t("payroll.work_policies.sim_computed", "Computed")}</div>
              <div className="font-mono text-lg font-bold text-emerald-600">{result.computed}</div>
            </div>
            <div className="rounded-xl bg-slate-50 p-3 dark:bg-slate-800/50">
              <div className="text-[11px] text-slate-500">{t("payroll.work_policies.sim_affected", "Affected")}</div>
              <div className="font-mono text-lg font-bold text-primary-600">{result.affected}</div>
            </div>
            <div className="rounded-xl bg-slate-50 p-3 dark:bg-slate-800/50">
              <div className="text-[11px] text-slate-500">{t("payroll.work_policies.sim_uncomputable", "Uncomputable")}</div>
              <div className="font-mono text-lg font-bold text-amber-600">{result.uncomputable}</div>
            </div>
          </div>

          <div className="flex flex-wrap items-center gap-3 font-mono text-[11px] text-slate-500">
            <span>fingerprint: {result.input_fingerprint.slice(0, 12)}</span>
            <span>evaluated: {result.evaluated_at}</span>
            {isExpired ? (
              <span className="rounded bg-amber-100 px-2 py-0.5 font-semibold text-amber-800 dark:bg-amber-950 dark:text-amber-200">
                {t("payroll.work_policies.sim_expired", "Expired — re-run recommended")}
              </span>
            ) : null}
          </div>

          <div className="divide-y divide-slate-100 rounded-xl border border-slate-100 dark:divide-slate-800 dark:border-slate-800">
            {result.items.map((item) => {
              const isOpen = expandedRel === item.employment_relationship_id;
              return (
                <div className="p-3 text-xs" key={item.employment_relationship_id}>
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <Button
                      className="h-auto gap-2 p-0 text-xs font-semibold text-slate-800 dark:text-slate-100"
                      onClick={() => { setExpandedRel(isOpen ? null : item.employment_relationship_id); }}
                      size="sm"
                      variant="ghost"
                    >
                      {isOpen ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                      <span>{relationshipLabels.get(item.employment_relationship_id) ?? item.employment_relationship_id}</span>
                    </Button>
                    {item.status === "uncomputable" ? (
                      <span className="rounded-full bg-amber-50 px-2.5 py-0.5 font-semibold text-amber-700 dark:bg-amber-950/60 dark:text-amber-300">
                        uncomputable · {item.reason}
                      </span>
                    ) : (
                      <div className="flex flex-wrap items-center gap-4 font-mono">
                        <span>Net: {item.baseline_net} → {item.candidate_net}</span>
                        <span className={item.affected ? "font-bold text-primary-600" : "text-slate-400"}>
                          Δ {item.net_delta}
                        </span>
                      </div>
                    )}
                  </div>
                  {isOpen && item.trace && item.trace.length > 0 ? (
                    <div className="mt-2 space-y-1 rounded-lg bg-slate-50 p-2.5 font-mono text-[11px] dark:bg-slate-800/40" data-testid="wp-simulation-trace">
                      {item.trace.map((step, i) => (
                        <div className="flex flex-wrap justify-between gap-2" key={`${step.step}-${i}`}>
                          <span className="text-slate-600 dark:text-slate-300">[{step.step}] {step.rule}: {step.expression}</span>
                          <span className="font-bold text-slate-800 dark:text-slate-100">{step.result}</span>
                        </div>
                      ))}
                    </div>
                  ) : null}
                </div>
              );
            })}
          </div>
        </div>
      ) : null}
    </div>
  );
}
