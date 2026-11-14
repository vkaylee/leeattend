import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { History, Plus } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { ApiError } from "@/lib/api/errors";
import { createPayrollFormulaVersion, fetchPayrollFormulaRules, fetchPayrollFormulaVersions, publishPayrollFormulaRule, retirePayrollFormulaRule } from "@/lib/api/payroll";
import { payrollKeys } from "@/lib/query-keys";

export function PayrollFormulaHistory({ ruleId }: { ruleId: string }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [expression, setExpression] = useState("");
  const [effectiveFrom, setEffectiveFrom] = useState("");
  const { data, isLoading, isError } = useQuery({
    queryKey: payrollKeys.formulaVersions(ruleId),
    queryFn: () => fetchPayrollFormulaVersions(ruleId),
  });
  const { data: rules } = useQuery({ queryKey: payrollKeys.rules(), queryFn: fetchPayrollFormulaRules });
  const rule = rules?.find((candidate) => candidate.id === ruleId);

  const createMutation = useMutation({
    mutationFn: () => createPayrollFormulaVersion(ruleId, { expression: expression.trim(), effective_from: effectiveFrom }),
    onSuccess: () => {
      setExpression("");
      setEffectiveFrom("");
      void queryClient.invalidateQueries({ queryKey: payrollKeys.formulaVersions(ruleId) });
    },
  });
  const actionMutation = useMutation({
    mutationFn: ({ action }: { action: "publish" | "retire" }) => action === "publish" ? publishPayrollFormulaRule(ruleId) : retirePayrollFormulaRule(ruleId),
    onSuccess: () => { void queryClient.invalidateQueries({ queryKey: payrollKeys.formulaVersions(ruleId) }); },
  });
  const isConflict = createMutation.error instanceof ApiError && createMutation.error.status === 409;
  const actionConflict = actionMutation.error instanceof ApiError && actionMutation.error.status === 409;

  if (isLoading) return <div className="p-4 text-sm text-slate-500">{t("common.loading")}</div>;
  if (isError) return <div className="p-4 text-sm text-red-600">{t("common.error")}</div>;
  if (rule?.status === "retired") return <div className="p-4 text-sm text-slate-500">{t("payroll.formula_history.retired_read_only", "This formula is retired and read-only.")}</div>;

  return (
    <div className="space-y-3" data-testid="payroll-formula-history">
      <form className="space-y-2 rounded-xl border border-dashed border-slate-300 p-4 dark:border-slate-600" onSubmit={(event) => { event.preventDefault(); createMutation.mutate(); }}>
        <div className="flex items-center gap-2">
          <Plus className="h-4 w-4 text-primary-600" />
          <Typography className="font-semibold" variant="p">{t("payroll.formula_history.create_version", "Create draft version")}</Typography>
        </div>
        <label className="block text-xs font-medium" htmlFor={`formula-expression-${ruleId}`}>{t("payroll.formula_history.expression", "Expression")}</label>
        <textarea aria-describedby={isConflict ? `formula-error-${ruleId}` : undefined} className="min-h-20 w-full rounded-lg border border-slate-200 bg-white p-2 text-xs dark:border-slate-700 dark:bg-slate-900" id={`formula-expression-${ruleId}`} onChange={(event) => { setExpression(event.target.value); }} required value={expression} />
        <label className="block text-xs font-medium" htmlFor={`formula-effective-from-${ruleId}`}>{t("payroll.formula_history.effective_from", "Effective from")}</label>
        <input className="w-full rounded-lg border border-slate-200 bg-white p-2 text-xs dark:border-slate-700 dark:bg-slate-900" id={`formula-effective-from-${ruleId}`} onChange={(event) => { setEffectiveFrom(event.target.value); }} required type="date" value={effectiveFrom} />
        {createMutation.isError ? <p className="text-xs text-red-600" id={`formula-error-${ruleId}`}>{isConflict ? t("payroll.formula_history.conflict", "Formula version conflicts with an existing version.") : t("common.error")}</p> : null}
        <Button disabled={createMutation.isPending} type="submit" variant="primary">{createMutation.isPending ? t("common.saving") : t("payroll.formula_history.create_version", "Create draft version")}</Button>
      </form>
      {!data?.length ? <div className="p-4 text-sm text-slate-500">{t("payroll.formula_history.empty", "No versions")}</div> : null}
      {data?.map((version) => (
        <div className="rounded-xl border border-slate-200 p-4 dark:border-slate-700" key={version.version_id}>
          <div className="flex items-center gap-2">
            <History className="h-4 w-4 text-primary-600" />
            <Typography className="font-semibold" variant="p">v{version.version} · {version.status}</Typography>
          </div>
          <code className="mt-2 block rounded-lg bg-slate-50 p-2 text-xs dark:bg-slate-800">{version.expression}</code>
          <Typography className="mt-2 block text-slate-500" variant="small">{version.effective_from}</Typography>
          {version.status === "draft" ? <Button disabled={actionMutation.isPending} onClick={() => { actionMutation.mutate({ action: "publish" }); }} size="sm" type="button" variant="outline">{t("payroll.formula_history.publish", "Publish")}</Button> : null}
          {version.status === "published" ? <Button disabled={actionMutation.isPending} onClick={() => { actionMutation.mutate({ action: "retire" }); }} size="sm" type="button" variant="destructive">{t("payroll.formula_history.retire", "Retire")}</Button> : null}
          {actionMutation.isError ? <p className="mt-2 text-xs text-red-600" role="alert">{actionConflict ? t("payroll.formula_history.action_conflict", "Formula status changed. Refresh history and retry.") : t("common.error")}</p> : null}
        </div>
      ))}
    </div>
  );
}
