import { useQuery } from "@tanstack/react-query";
import { ChevronDown, Calculator } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { fetchPayrollCalculationRuns } from "@/lib/api/payroll";
import { Typography } from "@/components/ui/typography";

interface TraceStep { expression?: string; result?: string; rule?: string; step?: string }

export function CalculationTrace({ employmentId, year, month }: { employmentId: string; year: number; month: number }) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const { data, isLoading, isError } = useQuery({
    queryKey: ["payroll-calculation-runs", employmentId, year, month],
    queryFn: () => fetchPayrollCalculationRuns(employmentId, year, month),
    enabled: open,
  });

  return (
    <div className="rounded-xl border border-slate-200 dark:border-slate-700">
      <button className="flex w-full items-center justify-between p-4 text-left" onClick={() => { setOpen((value) => !value); }} type="button">
        <span className="flex items-center gap-2 text-sm font-semibold"><Calculator className="h-4 w-4 text-primary-600" />{t("payroll.calculation_trace.title", "Calculation trace")}</span>
        <ChevronDown className={`h-4 w-4 transition-transform ${open ? "rotate-180" : ""}`} />
      </button>
      {open && <div className="space-y-3 border-t border-slate-100 p-4 dark:border-slate-800">
        {isLoading && <Typography variant="small">{t("common.loading")}</Typography>}
        {isError && <Typography variant="small" className="text-red-600">{t("common.error")}</Typography>}
        {!isLoading && !isError && !data?.length && <Typography variant="small" className="text-slate-500">{t("payroll.calculation_trace.empty", "No calculation trace")}</Typography>}
        {data?.map((run) => {
          const trace = Array.isArray(run.calculation_trace) ? run.calculation_trace as TraceStep[] : [];
          return <div className="space-y-2" key={run.id}><Typography variant="small" className="font-semibold">{run.engine_version} · {run.created_at}</Typography>{trace.map((step, index) => <div className="rounded-lg bg-slate-50 p-3 text-xs dark:bg-slate-800" key={`${run.id}-${index}`}><div className="font-semibold">{step.step} · {step.rule}</div><code>{step.expression}</code><div>{step.result}</div></div>)}</div>;
        })}
      </div>}
    </div>
  );
}
