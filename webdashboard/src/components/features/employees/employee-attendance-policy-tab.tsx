import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { format } from "date-fns";
import { Clock, Loader2, ShieldCheck } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { fetchEmployeeRelationships } from "@/lib/api/employees";
import { resolveWorkPolicyByRelationship } from "@/lib/api/payroll";
import { payrollKeys } from "@/lib/query-keys";

interface Props {
  employeeId: string;
}

export default function EmployeeAttendancePolicyTab({ employeeId }: Props) {
  const { t } = useTranslation();
  const today = format(new Date(), "yyyy-MM-dd");

  const { data: relationships = [], isLoading: isEmpLoading } = useQuery({
    queryKey: ["employee-relationships-policy", employeeId],
    queryFn: () => fetchEmployeeRelationships(employeeId),
    enabled: Boolean(employeeId),
  });

  const activeRelId = relationships.find((r) => r.status === "active")?.id ?? relationships[0]?.id;

  const { data: resolvedPolicy, isLoading: isPolicyLoading } = useQuery({
    queryKey: payrollKeys.workPolicyResolved(activeRelId, today),
    queryFn: () => resolveWorkPolicyByRelationship(activeRelId as string, today),
    enabled: Boolean(activeRelId),
    retry: false,
  });

  if (isEmpLoading || isPolicyLoading) {
    return (
      <div className="flex items-center justify-center p-16">
        <Loader2 className="h-8 w-8 animate-spin text-primary-500" />
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6 animate-in fade-in duration-300" data-testid="attendance-policy-tab">
      <div className="flex flex-wrap items-center justify-between gap-4 px-2">
        <div className="flex items-center gap-4">
          <div className="flex h-10 w-10 items-center justify-center rounded-2xl border border-primary-100 bg-primary-50 text-primary-600 shadow-sm dark:border-primary-800/50 dark:bg-primary-950 dark:text-primary-400">
            <ShieldCheck size={20} />
          </div>
          <div>
            <Typography className="text-lg font-bold tracking-tight text-slate-800 dark:text-slate-100" variant="h3">
              {t("payroll.work_policies.title", "Work Policy")}
            </Typography>
            <Typography className="text-xs text-slate-500 dark:text-slate-400" variant="p">
              {t("payroll.work_policies.managed_in_settings", "Attendance, work credit, pay basis, and overtime rules are resolved from the employee relationship's published work policy.")}
            </Typography>
          </div>
        </div>
        <Link search={{} as never} to="/settings/payroll">
          <Button size="sm" variant="outline">{t("payroll.work_policies.open_settings", "Manage Work Policies")}</Button>
        </Link>
      </div>

      {resolvedPolicy ? (
        <div className="rounded-2xl border border-slate-200 bg-white p-6 shadow-xs dark:border-slate-800 dark:bg-slate-900">
          <div className="flex flex-wrap items-center justify-between gap-2 border-b border-slate-100 pb-4 dark:border-slate-800">
            <div>
              <Typography className="text-base font-bold text-slate-800 dark:text-slate-100" variant="h4">
                {resolvedPolicy.group_name} ({resolvedPolicy.group_code})
              </Typography>
              <Typography className="text-xs text-slate-500 dark:text-slate-400" variant="p">
                v{resolvedPolicy.version_no} · {resolvedPolicy.effective_from} → {resolvedPolicy.effective_to ?? t("common.present", "Present")}
              </Typography>
            </div>
            <span className="rounded-full bg-emerald-50 px-3 py-1 text-xs font-semibold text-emerald-700 dark:bg-emerald-950/60 dark:text-emerald-300">
              {resolvedPolicy.status}
            </span>
          </div>
          <div className="mt-4 grid grid-cols-1 gap-4 text-sm sm:grid-cols-3">
            <div>
              <span className="block text-xs text-slate-500">{t("payroll.work_policies.minutes_per_credit", "Minutes / Credit")}</span>
              <span className="font-mono font-semibold text-slate-800 dark:text-slate-100">{resolvedPolicy.minutes_per_credit}m</span>
            </div>
            <div>
              <span className="block text-xs text-slate-500">{t("payroll.work_policies.pay_method", "Pay Method")}</span>
              <span className="font-semibold text-slate-800 dark:text-slate-100">{resolvedPolicy.pay_method}</span>
            </div>
            <div>
              <span className="block text-xs text-slate-500">{t("payroll.work_policies.rounding", "Rounding / Work Unit")}</span>
              <span className="font-mono font-semibold text-slate-800 dark:text-slate-100">
                {resolvedPolicy.work_unit_minutes}m · {resolvedPolicy.rounding_policy}
              </span>
            </div>
          </div>
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center rounded-2xl border border-dashed border-slate-200 bg-slate-50 p-10 text-center dark:border-slate-800 dark:bg-slate-900/40">
          <Clock className="mb-3 h-8 w-8 text-slate-400" />
          <Typography className="text-sm font-semibold text-slate-700 dark:text-slate-200" variant="h4">
            {t("payroll.work_policies.no_active_membership", "No published work policy assigned")}
          </Typography>
          <Typography className="mt-1 max-w-md text-xs text-slate-500 dark:text-slate-400" variant="p">
            {t("payroll.work_policies.assign_hint", "Assign this employee relationship to a published Work Policy Group in Payroll Settings.")}
          </Typography>
        </div>
      )}
    </div>
  );
}
