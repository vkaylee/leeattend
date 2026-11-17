import type { TFunction } from "i18next";

import { useQuery } from "@tanstack/react-query";
import { Calculator, ChevronDown } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import type { PayrollCalculationRun } from "@/lib/api/types-payroll";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { fetchPayrollCalculationRuns } from "@/lib/api/payroll";
import { payrollKeys } from "@/lib/query-keys";

interface TraceStep {
  expression?: string;
  result?: string;
  rule?: string;
  step?: string;
}

interface CalculationTraceProps {
  employmentId: string;
  month: number;
  year: number;
}

interface CalculationTraceBodyProps {
  data: PayrollCalculationRun[] | undefined;
  isError: boolean;
  isLoading: boolean;
  translate: TFunction;
}

interface CalculationTraceStatusProps {
  hasRuns: boolean;
  isError: boolean;
  isLoading: boolean;
  translate: TFunction;
}

interface CalculationRunProps {
  run: PayrollCalculationRun;
}

interface TraceStepCardProps {
  runId: string;
  step: TraceStep;
  stepIndex: number;
}

function getTraceSteps(value: unknown): TraceStep[] {
  return Array.isArray(value) ? value as TraceStep[] : [];
}

function CalculationTraceStatus({ hasRuns, isError, isLoading, translate }: CalculationTraceStatusProps) {
  return (
    <>
      {isLoading ? <Typography variant="small">{translate("common.loading")}</Typography> : null}
      {isError ? <Typography className="text-red-600" variant="small">{translate("common.error")}</Typography> : null}
      {!isLoading && !isError && !hasRuns ? (
        <Typography className="text-slate-500" variant="small">
          {translate("payroll.calculation_trace.empty", "No calculation trace")}
        </Typography>
      ) : null}
    </>
  );
}

function TraceStepCard({ runId, step, stepIndex }: TraceStepCardProps) {
  return (
    <div className="rounded-lg bg-slate-50 p-3 text-xs dark:bg-slate-800" key={`${runId}-${stepIndex}`}>
      <div className="font-semibold">{step.step} · {step.rule}</div>
      <code>{step.expression}</code>
      <div>{step.result}</div>
    </div>
  );
}

function CalculationRun({ run, translate }: CalculationRunProps & { translate: TFunction }) {
  const trace = getTraceSteps(run.calculation_trace);
  const partial = trace.some((step) => !step.step || !step.rule || !step.expression || !step.result);
  const provenanceIncomplete = !run.formula_version_id || run.input_snapshot == null || run.output_snapshot == null;

  return (
    <div className="space-y-2" key={run.id}>
      {partial ? <Typography className="text-amber-700" variant="small">{translate("payroll.calculation_trace.partial")}</Typography> : null}
      {provenanceIncomplete ? <Typography className="text-amber-700" variant="small">{translate("payroll.calculation_trace.provenance_incomplete")}</Typography> : null}
      <Typography className="font-semibold" variant="small">{run.engine_version} · {run.created_at}</Typography>
      {run.formula_version_id ? <Typography variant="small">{run.formula_version_id}</Typography> : null}
      {trace.map((step, index) => (
        <TraceStepCard key={`${run.id}-${index}`} runId={run.id} step={step} stepIndex={index} />
      ))}
      {run.input_snapshot != null ? <Typography variant="small">{translate("payroll.calculation_trace.input_snapshot")}</Typography> : null}
      {run.output_snapshot != null ? <Typography variant="small">{translate("payroll.calculation_trace.output_snapshot")}</Typography> : null}
    </div>
  );
}

function CalculationTraceRuns({ runs, translate }: { runs: PayrollCalculationRun[]; translate: TFunction }) {
  return <>{runs.map((run) => <CalculationRun key={run.id} run={run} translate={translate} />)}</>;
}

function CalculationTraceBody({ data, isError, isLoading, translate }: CalculationTraceBodyProps) {
  const runs = data ?? [];

  return (
    <>
      <CalculationTraceStatus
        hasRuns={runs.length > 0}
        isError={isError}
        isLoading={isLoading}
        translate={translate}
      />
      <CalculationTraceRuns runs={runs} translate={translate} />
    </>
  );
}

export function CalculationTrace({ employmentId, month, year }: CalculationTraceProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const { data, isError, isLoading } = useQuery({
    enabled: open,
    queryFn: () => fetchPayrollCalculationRuns(employmentId, year, month),
    queryKey: payrollKeys.calculationRuns(employmentId, year, month),
  });

  return (
    <div className="rounded-xl border border-slate-200 dark:border-slate-700">
      <Button
        className="flex w-full items-center justify-between border-0 bg-transparent p-4 text-left font-normal text-foreground shadow-none hover:bg-transparent"
        onClick={() => {
          setOpen((value) => !value);
        }}
        type="button"
        variant="ghost"
      >
        <span className="flex items-center gap-2 text-sm font-semibold">
          <Calculator className="h-4 w-4 text-primary-600" />
          {t("payroll.calculation_trace.title", "Calculation trace")}
        </span>
        <ChevronDown className={`h-4 w-4 transition-transform ${open ? "rotate-180" : ""}`} />
      </Button>
      {open ? (
        <div className="space-y-3 border-t border-slate-100 p-4 dark:border-slate-800">
          <CalculationTraceBody data={data} isError={isError} isLoading={isLoading} translate={t} />
        </div>
      ) : null}
    </div>
  );
}
