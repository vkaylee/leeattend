import { History } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { fetchPayrollFormulaVersions } from "@/lib/api/payroll";
import { Typography } from "@/components/ui/typography";

export function PayrollFormulaHistory({ ruleId }: { ruleId: string }) {
  const { t } = useTranslation();
  const { data, isLoading, isError } = useQuery({
    queryKey: ["payroll-formula-versions", ruleId],
    queryFn: () => fetchPayrollFormulaVersions(ruleId),
  });

  if (isLoading) return <div className="p-4 text-sm text-slate-500">{t("common.loading")}</div>;
  if (isError) return <div className="p-4 text-sm text-red-600">{t("common.error")}</div>;
  if (!data?.length) return <div className="p-4 text-sm text-slate-500">{t("payroll.formula_history.empty", "No versions")}</div>;

  return (
    <div className="space-y-3" data-testid="payroll-formula-history">
      {data.map((version) => (
        <div className="rounded-xl border border-slate-200 p-4 dark:border-slate-700" key={version.version_id}>
          <div className="flex items-center gap-2">
            <History className="h-4 w-4 text-primary-600" />
            <Typography variant="p" className="font-semibold">v{version.version} · {version.status}</Typography>
          </div>
          <code className="mt-2 block rounded-lg bg-slate-50 p-2 text-xs dark:bg-slate-800">{version.expression}</code>
          <Typography variant="small" className="mt-2 block text-slate-500">{version.effective_from}</Typography>
        </div>
      ))}
    </div>
  );
}
