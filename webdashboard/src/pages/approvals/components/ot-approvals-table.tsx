import { CheckCircle, XCircle } from "lucide-react";
import { useTranslation } from "react-i18next";

interface OtSummary {
  claimed_ot_minutes: number;
  date: string;
  employee_id: string;
  entered_via?: string;
  id: string;
  ot_reason?: string;
  system_detected_ot_minutes: number;
}

interface OtApprovalsTableProps {
  handleApprove: (summary: OtSummary, approved: boolean) => Promise<void>;
  isProcessing: boolean;
  pendingSummaries: OtSummary[];
}

export function OtApprovalsTable({
  handleApprove,
  isProcessing,
  pendingSummaries,
}: OtApprovalsTableProps) {
  const { t } = useTranslation();

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left border-collapse">
        <thead>
          <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
              {t("ot_approvals.table.date")}
            </th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
              {t("ot_approvals.table.employee")}
            </th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
              {t("ot_approvals.table.source")}
            </th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">
              {t("ot_approvals.table.system")}
            </th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">
              {t("ot_approvals.table.claimed")}
            </th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
              {t("ot_approvals.table.reason")}
            </th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-right">
              {t("ot_approvals.table.actions")}
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
          {pendingSummaries.map((summary) => (
            <tr
              className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group"
              key={summary.id}
            >
              <td className="px-8 py-6 font-bold text-slate-700 dark:text-slate-200">
                {summary.date}
              </td>
              <td className="px-8 py-6 text-sm text-slate-600 dark:text-slate-300">
                {summary.employee_id}
              </td>
              <td className="px-8 py-6">
                <SourceBadge enteredVia={summary.entered_via} />
              </td>
              <td className="px-8 py-6 text-center text-sm font-mono text-slate-500 dark:text-slate-400">
                {summary.system_detected_ot_minutes}m
              </td>
              <td className="px-8 py-6 text-center text-sm font-bold font-mono text-amber-600 dark:text-amber-300">
                {summary.claimed_ot_minutes}m
              </td>
              <td className="px-8 py-6 text-sm text-slate-600 dark:text-slate-300 italic">
                &quot;{summary.ot_reason}&quot;
              </td>
              <td className="px-8 py-6 text-right">
                <div className="flex items-center justify-end gap-2">
                  <button
                    className="p-2 rounded-full hover:bg-emerald-50 dark:bg-emerald-950 text-slate-400 dark:text-slate-500 hover:text-emerald-600 dark:text-emerald-300 transition-all disabled:opacity-50"
                    disabled={isProcessing}
                    onClick={() => {
                      void handleApprove(summary, true);
                    }}
                    title={t("ot_approvals.actions.approve")}
                  >
                    <CheckCircle className="w-5 h-5" />
                  </button>
                  <button
                    className="p-2 rounded-full hover:bg-rose-50 dark:bg-rose-950 text-slate-400 dark:text-slate-500 hover:text-rose-600 dark:text-rose-300 transition-all disabled:opacity-50"
                    disabled={isProcessing}
                    onClick={() => {
                      void handleApprove(summary, false);
                    }}
                    title={t("ot_approvals.actions.reject")}
                  >
                    <XCircle className="w-5 h-5" />
                  </button>
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function SourceBadge({ enteredVia }: { enteredVia?: string }) {
  const { t } = useTranslation();
  if (enteredVia === "paper_form") {
    return (
      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-amber-100 dark:bg-amber-900 text-amber-700 dark:text-amber-300 border border-amber-200">
        {t("ot_approvals.badges.paper")}
      </span>
    );
  }
  return (
    <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border border-slate-200 dark:border-slate-700">
      {t("ot_approvals.badges.self")}
    </span>
  );
}
