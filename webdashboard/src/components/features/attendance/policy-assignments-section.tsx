import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Layers } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { Typography } from "@/components/ui/typography";
import { fetchWorkPolicyGroups } from "@/lib/api/payroll";
import { payrollKeys } from "@/lib/query-keys";

export interface PolicyAssignmentsOptions {
  defaultFilter?: string;
  hideFilterTabs?: boolean;
  q?: string;
}

export function PolicyAssignmentsSection({ options: _options = {} }: { options?: PolicyAssignmentsOptions }) {
  const { t } = useTranslation();
  const { data: groups = [], isLoading } = useQuery({
    queryKey: payrollKeys.workPolicyGroups(),
    queryFn: () => fetchWorkPolicyGroups(),
  });

  return (
    <div className="space-y-4 rounded-2xl border border-slate-200 bg-white p-6 dark:border-slate-800 dark:bg-slate-900" data-testid="policy-assignments-section">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-3">
          <Layers className="h-5 w-5 text-primary-600" />
          <div>
            <Typography className="text-base font-bold text-slate-800 dark:text-slate-100" variant="h3">
              {t("payroll.work_policies.title", "Work Policy Groups")}
            </Typography>
            <Typography className="text-xs text-slate-500 dark:text-slate-400" variant="p">
              {t("payroll.work_policies.managed_in_settings", "Attendance and payroll policies are assigned per employment relationship via Work Policy Groups.")}
            </Typography>
          </div>
        </div>
        <Link search={{} as never} to="/settings/payroll">
          <Button size="sm" variant="primary">{t("payroll.work_policies.open_settings", "Manage Work Policies")}</Button>
        </Link>
      </div>

      {isLoading ? (
        <Skeleton className="h-24 w-full rounded-xl" />
      ) : groups.length === 0 ? (
        <p className="text-xs italic text-slate-400">{t("payroll.work_policies.empty_groups", "No work policy groups configured.")}</p>
      ) : (
        <div className="divide-y divide-slate-100 rounded-xl border border-slate-100 dark:divide-slate-800 dark:border-slate-800">
          {groups.map((g) => (
            <div className="flex flex-wrap items-center justify-between gap-2 px-4 py-3 text-xs" key={g.id}>
              <div>
                <span className="font-mono font-bold text-primary-600">{g.code}</span>{" "}
                <span className="font-semibold text-slate-800 dark:text-slate-100">{g.name}</span>
              </div>
              <div className="flex items-center gap-3 text-slate-500">
                <span>{g.current_published ? `v${g.current_published.version_no} (published)` : "draft"}</span>
                <span>{g.active_membership_count} {t("payroll.work_policies.members_count", "assigned")}</span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
