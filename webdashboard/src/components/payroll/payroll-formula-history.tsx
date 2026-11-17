import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { History, Plus } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { ApiError } from "@/lib/api/errors";
import { createPayrollFormulaVersion, fetchPayrollFormulaRules, fetchPayrollFormulaVersions, publishPayrollFormulaRule, retirePayrollFormulaRule } from "@/lib/api/payroll";
import type { PayrollFormulaRuleVersion } from "@/lib/api/types-payroll";
import { payrollKeys } from "@/lib/query-keys";

type FormulaAction = "publish" | "retire";

function isConflict(error: unknown): boolean {
  return error instanceof ApiError && error.status === 409;
}

function runFormulaAction(ruleId: string, action: FormulaAction) {
  return action === "publish" ? publishPayrollFormulaRule(ruleId) : retirePayrollFormulaRule(ruleId);
}

function CreateFormulaVersionForm({
  expression,
  effectiveFrom,
  isConflictError,
  isError,
  isPending,
  onEffectiveFromChange,
  onExpressionChange,
  onSubmit,
  ruleId,
}: {
  effectiveFrom: string;
  expression: string;
  isConflictError: boolean;
  isError: boolean;
  isPending: boolean;
  onEffectiveFromChange: (value: string) => void;
  onExpressionChange: (value: string) => void;
  onSubmit: () => void;
  ruleId: string;
}) {
  const { t } = useTranslation();
  const expressionId = `formula-expression-${ruleId}`;
  const effectiveFromId = `formula-effective-from-${ruleId}`;
  const errorId = `formula-error-${ruleId}`;

  return (
    <form
      className="space-y-2 rounded-xl border border-dashed border-slate-300 p-4 dark:border-slate-600"
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit();
      }}
    >
      <div className="flex items-center gap-2">
        <Plus className="h-4 w-4 text-primary-600" />
        <Typography className="font-semibold" variant="p">{t("payroll.formula_history.create_version", "Create draft version")}</Typography>
      </div>
      <label className="block text-xs font-medium" htmlFor={expressionId}>{t("payroll.formula_history.expression", "Expression")}</label>
      <textarea
        aria-describedby={isConflictError ? errorId : undefined}
        className="min-h-20 w-full rounded-lg border border-slate-200 bg-white p-2 text-xs dark:border-slate-700 dark:bg-slate-900"
        id={expressionId}
        onChange={(event) => { onExpressionChange(event.target.value); }}
        required
        value={expression}
      />
      <label className="block text-xs font-medium" htmlFor={effectiveFromId}>{t("payroll.formula_history.effective_from", "Effective from")}</label>
      <input
        className="w-full rounded-lg border border-slate-200 bg-white p-2 text-xs dark:border-slate-700 dark:bg-slate-900"
        id={effectiveFromId}
        onChange={(event) => { onEffectiveFromChange(event.target.value); }}
        required
        type="date"
        value={effectiveFrom}
      />
      {isError ? <p className="text-xs text-red-600" id={errorId}>{isConflictError ? t("payroll.formula_history.conflict", "Formula version conflicts with an existing version.") : t("common.error")}</p> : null}
      <Button disabled={isPending} type="submit" variant="primary">{isPending ? t("common.saving") : t("payroll.formula_history.create_version", "Create draft version")}</Button>
    </form>
  );
}

function FormulaVersionCard({
  actionConflict,
  actionPending,
  onAction,
  version,
}: {
  actionConflict: boolean;
  actionPending: boolean;
  onAction: (action: FormulaAction) => void;
  version: PayrollFormulaRuleVersion;
}) {
  const { t } = useTranslation();
  const isDraft = version.status === "draft";
  const isPublished = version.status === "published";

  return (
    <div className="rounded-xl border border-slate-200 p-4 dark:border-slate-700">
      <div className="flex items-center gap-2">
        <History className="h-4 w-4 text-primary-600" />
        <Typography className="font-semibold" variant="p">v{version.version} · {version.status}</Typography>
      </div>
      <code className="mt-2 block rounded-lg bg-slate-50 p-2 text-xs dark:bg-slate-800">{version.expression}</code>
      <Typography className="mt-2 block text-slate-500" variant="small">{version.effective_from}</Typography>
      {isDraft ? <Button disabled={actionPending} onClick={() => { onAction("publish"); }} size="sm" type="button" variant="outline">{t("payroll.formula_history.publish", "Publish")}</Button> : null}
      {isPublished ? <Button disabled={actionPending} onClick={() => { onAction("retire"); }} size="sm" type="button" variant="destructive">{t("payroll.formula_history.retire", "Retire")}</Button> : null}
      {actionConflict ? <p className="mt-2 text-xs text-red-600" role="alert">{t("payroll.formula_history.action_conflict", "Formula status changed. Refresh history and retry.")}</p> : null}
    </div>
  );
}

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
    mutationFn: ({ action }: { action: FormulaAction }) => runFormulaAction(ruleId, action),
    onSuccess: () => { void queryClient.invalidateQueries({ queryKey: payrollKeys.formulaVersions(ruleId) }); },
  });

  if (isLoading) return <div className="p-4 text-sm text-slate-500">{t("common.loading")}</div>;
  if (isError) return <div className="p-4 text-sm text-red-600">{t("common.error")}</div>;

  return (
    <div className="space-y-3" data-testid="payroll-formula-history">
      <CreateFormulaVersionForm
        effectiveFrom={effectiveFrom}
        expression={expression}
        isConflictError={isConflict(createMutation.error)}
        isError={createMutation.isError}
        isPending={createMutation.isPending}
        onEffectiveFromChange={setEffectiveFrom}
        onExpressionChange={setExpression}
        onSubmit={() => { createMutation.mutate(); }}
        ruleId={ruleId}
      />
      {!data?.length ? <div className="p-4 text-sm text-slate-500">{t("payroll.formula_history.empty", "No versions")}</div> : null}
      {data?.map((version) => (
        <FormulaVersionCard
          actionConflict={isConflict(actionMutation.error)}
          actionPending={actionMutation.isPending}
          key={version.version_id}
          onAction={(action) => { actionMutation.mutate({ action }); }}
          version={version}
        />
      ))}
    </div>
  );
}
