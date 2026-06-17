import { AlertTriangle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Skeleton } from "@/components/ui/skeleton";
import { Typography } from "@/components/ui/typography";
import { QuotaSummary } from "@/lib/api/overtime-requests";

type ExtendedQuota = QuotaSummary & { monthly_exceeded?: boolean; yearly_exceeded?: boolean };

function QuotaDetails({ quota }: { quota: ExtendedQuota }) {
  const monthlyUsed = quota.monthly_used;
  const monthlyLimit = quota.monthly_limit;
  const yearlyUsed = quota.yearly_used;
  const yearlyLimit = quota.yearly_limit;

  const monthlyPercent = Math.min(100, (monthlyUsed / (monthlyLimit ?? 1)) * 100);
  const yearlyPercent = Math.min(100, (yearlyUsed / (yearlyLimit ?? 1)) * 100);

  const isExceeded = [quota.monthly_exceeded, quota.yearly_exceeded].some(Boolean);

  return (
    <div className="space-y-6">
      <div className="space-y-2">
        <div className="flex justify-between items-center text-sm font-bold">
          <span className="text-slate-600 dark:text-slate-400">Monthly Usage</span>
          <span className="text-slate-800 dark:text-slate-200">{monthlyUsed} / {monthlyLimit ?? "∞"} hrs</span>
        </div>
        <div className="h-2 w-full bg-slate-100 dark:bg-slate-800 rounded-full overflow-hidden">
          <div
            className="h-full bg-indigo-500 rounded-full transition-all duration-500"
            style={{ width: `${monthlyPercent}%` }}
          />
        </div>
      </div>

      <div className="space-y-2">
        <div className="flex justify-between items-center text-sm font-bold">
          <span className="text-slate-600 dark:text-slate-400">Yearly Usage</span>
          <span className="text-slate-800 dark:text-slate-200">{yearlyUsed} / {yearlyLimit ?? "∞"} hrs</span>
        </div>
        <div className="h-2 w-full bg-slate-100 dark:bg-slate-800 rounded-full overflow-hidden">
          <div
            className="h-full bg-emerald-500 rounded-full transition-all duration-500"
            style={{ width: `${yearlyPercent}%` }}
          />
        </div>
      </div>

      {isExceeded ? (
        <div className="mt-4 p-4 bg-rose-50 dark:bg-rose-900/30 border border-rose-100 dark:border-rose-800 rounded-2xl flex items-start gap-3">
          <AlertTriangle className="w-5 h-5 text-rose-600 shrink-0 mt-0.5" />
          <div className="space-y-1">
            <Typography className="text-sm font-bold text-rose-800 dark:text-rose-200">
              Quota Exceeded
            </Typography>
            <Typography className="text-xs text-rose-600 dark:text-rose-400 font-medium">
              You have exceeded your overtime limits. Further requests may require special approval.
            </Typography>
          </div>
        </div>
      ) : null}
    </div>
  );
}

export function OtQuotaWidget({ quota, isLoading }: { quota?: ExtendedQuota | null; isLoading: boolean }) {
  const { t } = useTranslation();

  return (
    <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 p-8 shadow-sm">
      <div className="flex items-center gap-3 mb-6">
        <div className="w-10 h-10 rounded-full bg-indigo-50 dark:bg-indigo-900/30 flex items-center justify-center">
          <AlertTriangle className="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
        </div>
        <div>
          <Typography className="text-lg font-bold text-slate-800 dark:text-slate-100" variant="h3">
            {t("ess.ot.quota_title", { defaultValue: "Overtime Quota" })}
          </Typography>
          <Typography className="text-[10px] font-bold text-slate-500 uppercase tracking-widest mt-0.5" variant="p">
            {t("ess.ot.quota_subtitle", { defaultValue: "Your limits and usage" })}
          </Typography>
        </div>
      </div>

      {isLoading ? (
        <div className="space-y-6 animate-pulse">
          <div className="space-y-2">
            <div className="flex justify-between">
              <Skeleton className="h-4 w-24 bg-slate-200 dark:bg-slate-800 rounded-md" />
              <Skeleton className="h-4 w-16 bg-slate-200 dark:bg-slate-800 rounded-md" />
            </div>
            <Skeleton className="h-2 w-full bg-slate-200/50 dark:bg-slate-800/50 rounded-full" />
          </div>
          <div className="space-y-2">
            <div className="flex justify-between">
              <Skeleton className="h-4 w-24 bg-slate-200 dark:bg-slate-800 rounded-md" />
              <Skeleton className="h-4 w-16 bg-slate-200 dark:bg-slate-800 rounded-md" />
            </div>
            <Skeleton className="h-2 w-full bg-slate-200/50 dark:bg-slate-800/50 rounded-full" />
          </div>
        </div>
      ) : !quota ? (
        <div className="text-center p-6 border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-2xl">
          <Typography className="text-slate-500 font-medium text-sm">
            {t("ess.ot.no_quota", { defaultValue: "No quota configured" })}
          </Typography>
        </div>
      ) : (
        <QuotaDetails quota={quota} />
      )}
    </div>
  );
}
