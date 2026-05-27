import { Loader2, AlertTriangle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";

export function OtQuotaWidget({ quota, isLoading }: { quota: any; isLoading: boolean }) {
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
        <div className="flex justify-center p-8">
          <Loader2 className="w-6 h-6 animate-spin text-slate-400" />
        </div>
      ) : !quota ? (
        <div className="text-center p-6 border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-2xl">
          <Typography className="text-slate-500 font-medium text-sm">
            {t("ess.ot.no_quota", { defaultValue: "No quota configured" })}
          </Typography>
        </div>
      ) : (
        <div className="space-y-6">
          <div className="space-y-2">
            <div className="flex justify-between items-center text-sm font-bold">
              <span className="text-slate-600 dark:text-slate-400">Monthly Usage</span>
              <span className="text-slate-800 dark:text-slate-200">{quota.monthly_used || 0} / {quota.monthly_limit || '∞'} hrs</span>
            </div>
            <div className="h-2 w-full bg-slate-100 dark:bg-slate-800 rounded-full overflow-hidden">
              <div 
                className="h-full bg-indigo-500 rounded-full transition-all duration-500"
                style={{ width: `${Math.min(100, ((quota.monthly_used || 0) / (quota.monthly_limit || 1)) * 100)}%` }}
              />
            </div>
          </div>

          <div className="space-y-2">
            <div className="flex justify-between items-center text-sm font-bold">
              <span className="text-slate-600 dark:text-slate-400">Yearly Usage</span>
              <span className="text-slate-800 dark:text-slate-200">{quota.yearly_used || 0} / {quota.yearly_limit || '∞'} hrs</span>
            </div>
            <div className="h-2 w-full bg-slate-100 dark:bg-slate-800 rounded-full overflow-hidden">
              <div 
                className="h-full bg-emerald-500 rounded-full transition-all duration-500"
                style={{ width: `${Math.min(100, ((quota.yearly_used || 0) / (quota.yearly_limit || 1)) * 100)}%` }}
              />
            </div>
          </div>

          {!!(quota.monthly_exceeded || quota.yearly_exceeded) && (
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
          )}
        </div>
      )}
    </div>
  );
}
