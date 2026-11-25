import { Link } from "@tanstack/react-router";
import { Clock, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

import { BulkReprocessSection } from "./time-settings/bulk-reprocess";
import { useBulkReprocess } from "./time-settings/hooks";
import { ReprocessModal } from "./time-settings/reprocess-modal";

export default function TimeSettings() {
  const { t } = useTranslation();
  const {
    isPending: isBulkReprocessPending,
    jobStatus,
    pollingJobId,
    reprocessEnd,
    reprocessStart,
    submitBulkReprocess,
    setReprocessEnd,
    setReprocessStart,
    setShowReprocessConfirm,
    showReprocessConfirm,
  } = useBulkReprocess(t);

  return (
    <div className="space-y-6 animate-in fade-in duration-300">
      <div className="flex flex-wrap items-center justify-between gap-4 rounded-2xl border border-slate-200 bg-white p-6 shadow-2xs dark:border-slate-800 dark:bg-slate-900">
        <div className="flex items-start gap-4">
          <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl bg-primary-50 text-primary-600 dark:bg-primary-950/60 dark:text-primary-400">
            <Clock className="h-5 w-5" />
          </div>
          <div className="space-y-1">
            <Typography className="text-lg font-bold text-slate-800 dark:text-slate-100" variant="h1">
              {t("common.settings.categories.time", "Time & Attendance Settings")}
            </Typography>
            <Typography className="text-sm text-slate-500 dark:text-slate-400" variant="p">
              {t("payroll.work_policies.time_settings_moved", "Attendance rounding, early arrival, grace period, and break policies are now versioned inside Work Policy Groups.")}
            </Typography>
          </div>
        </div>
        <Link search={{} as never} to="/settings/payroll">
          <Button variant="primary">{t("payroll.work_policies.open_settings", "Manage Work Policies")}</Button>
        </Link>
      </div>

      <div className="rounded-2xl border border-slate-200 bg-white p-6 shadow-2xs dark:border-slate-800 dark:bg-slate-900">
        <div className="mb-4 flex items-center gap-2 text-slate-800 dark:text-slate-100">
          <RefreshCw className="h-4 w-4 text-primary-600" />
          <Typography className="text-base font-bold" variant="h2">
            {t("reports.reprocess.bulk_tooltip", "Bulk Attendance Reprocess")}
          </Typography>
        </div>
        <BulkReprocessSection
          reprocessEnd={reprocessEnd}
          reprocessStart={reprocessStart}
          setReprocessEnd={setReprocessEnd}
          setReprocessStart={setReprocessStart}
          setShowReprocessConfirm={setShowReprocessConfirm}
          t={t}
        />
      </div>

      <ReprocessModal
        isOpen={showReprocessConfirm}
        isPending={isBulkReprocessPending}
        jobStatus={jobStatus}
        onClose={() => {
          if (!isBulkReprocessPending && !pollingJobId) {
            setShowReprocessConfirm(false);
          }
        }}
        onSubmit={submitBulkReprocess}
        pollingJobId={pollingJobId}
        reprocessEnd={reprocessEnd}
        reprocessStart={reprocessStart}
        t={t}
      />
    </div>
  );
}
