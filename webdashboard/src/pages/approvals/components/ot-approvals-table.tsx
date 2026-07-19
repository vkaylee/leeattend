import { CheckCircle, XCircle, Paperclip, Plus, AlertTriangle } from "lucide-react";
import { useTranslation } from "react-i18next";

import InfiniteScrollSentinel from "@/components/common/infinite-scroll-sentinel";
import Button from "@/components/ui/button";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";

export interface OtSummary {
  claimed_ot_minutes: number;
  date: string;
  employee_id_number?: string;
  employee_name?: string;
  employment_id: string;
  entered_via?: string | null;
  id: string;
  ot_reason?: string | null;
  paper_form_photo_url?: string | null;
  system_detected_ot_minutes: number;
}

interface OtApprovalsTableProps {
  fetchNextPage: () => void;
  handleApprove: (summary: OtSummary, approved: boolean) => Promise<void>;
  hasNextPage: boolean | undefined;
  isFetchingNextPage: boolean;
  isProcessing: boolean;
  onAddPhoto?: (summary: OtSummary) => void;
  onPreviewAttachment?: (fileUrl: string, title: string) => void;
  pendingSummaries: OtSummary[];
}

export function OtApprovalsTable({
  handleApprove,
  isProcessing,
  pendingSummaries,
  onAddPhoto,
  onPreviewAttachment,
  fetchNextPage,
  hasNextPage,
  isFetchingNextPage,
}: OtApprovalsTableProps) {
  const { t } = useTranslation();

  const loadMoreRef = useInfiniteScroll<HTMLTableRowElement>({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
  });

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
              {t("ot_approvals.table.evidence", { defaultValue: "Evidence" })}
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
              <td className="px-8 py-6">
                <div className="flex flex-col">
                  <span className="font-bold text-sm text-slate-800 dark:text-slate-200">
                    {summary.employee_name ?? summary.employment_id}
                  </span>
                  {!!summary.employee_id_number && (
                    <span className="text-[10px] font-mono text-slate-400 dark:text-slate-500 uppercase tracking-tight">
                      {summary.employee_id_number}
                    </span>
                  )}
                </div>
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
              <td className="px-8 py-6">
                <EvidenceCell 
                  onAddPhoto={onAddPhoto} 
                  onPreviewAttachment={onPreviewAttachment} 
                  summary={summary}
                />
              </td>
              <td className="px-8 py-6 text-sm text-slate-600 dark:text-slate-300 italic">
                &quot;{summary.ot_reason}&quot;
              </td>
              <td className="px-8 py-6 text-right">
                <div className="flex items-center justify-end gap-2">
                  <Button
                    className="h-auto w-auto p-2 rounded-full hover:bg-emerald-50 dark:bg-emerald-950 text-slate-400 dark:text-slate-500 hover:text-emerald-600 dark:text-emerald-300 transition-all disabled:opacity-50 shadow-none"
                    disabled={isProcessing}
                    icon={<CheckCircle className="w-5 h-5" />}
                    onClick={() => {
                      void handleApprove(summary, true);
                    }}
                    size="icon"
                    title={t("ot_approvals.actions.approve")}
                    variant="ghost"
                  />
                  <Button
                    className="h-auto w-auto p-2 rounded-full hover:bg-rose-50 dark:bg-rose-950 text-slate-400 dark:text-slate-500 hover:text-rose-600 dark:text-rose-300 transition-all disabled:opacity-50 shadow-none"
                    disabled={isProcessing}
                    icon={<XCircle className="w-5 h-5" />}
                    onClick={() => {
                      void handleApprove(summary, false);
                    }}
                    size="icon"
                    title={t("ot_approvals.actions.reject")}
                    variant="ghost"
                  />
                </div>
              </td>
            </tr>
          ))}
          <InfiniteScrollSentinel
            colSpan={8}
            isFetchingNextPage={isFetchingNextPage}
            sentinelRef={loadMoreRef}
          />
        </tbody>
      </table>
    </div>
  );
}

function EvidenceCell({
  summary,
  onAddPhoto,
  onPreviewAttachment,
}: {
  summary: OtSummary;
  onAddPhoto?: (summary: OtSummary) => void;
  onPreviewAttachment?: (fileUrl: string, title: string) => void;
}) {
  const { t } = useTranslation();

  if (summary.entered_via !== "paper_form") {
    return <span className="text-xs text-slate-400 dark:text-slate-500">—</span>;
  }

  if (summary.paper_form_photo_url) {
    return (
      <Button
        className="inline-flex items-center gap-1.5 text-xs font-bold text-primary-600 dark:text-primary-400 hover:underline cursor-pointer bg-transparent border-none p-0 h-auto normal-case shadow-none"
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        onClick={() => { onPreviewAttachment?.(summary.paper_form_photo_url!, `OT Claim Evidence - ${summary.employee_name ?? summary.employment_id}`); }}
        variant="ghost"
      >
        <Paperclip className="w-3.5 h-3.5" />
        {t("ot_approvals.evidence.view", { defaultValue: "View Photo" })}
      </Button>
    );
  }

  return (
    <div className="flex items-center gap-2">
      <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[9px] font-bold bg-amber-50 dark:bg-amber-900/20 text-amber-800 dark:text-amber-300 border border-amber-200 dark:border-amber-900/50">
        <AlertTriangle className="w-3 h-3 text-amber-500" />
        {t("ot_approvals.evidence.missing", { defaultValue: "No Photo" })}
      </span>
      {onAddPhoto ? (
        <Button
          className="h-auto px-2 py-1 rounded-lg text-[9px] font-bold uppercase tracking-wider bg-slate-100 hover:bg-primary-50 hover:text-primary-600 text-slate-600 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-primary-950/30 dark:hover:text-primary-400 shadow-none border-none transition-all flex items-center gap-1 normal-case"
          onClick={() => { onAddPhoto(summary); }}
          variant="ghost"
        >
          <Plus className="w-3 h-3" strokeWidth={3} />
          {t("ot_approvals.evidence.add", { defaultValue: "Add Photo" })}
        </Button>
      ) : null}
    </div>
  );
}

function SourceBadge({ enteredVia }: { enteredVia?: string | null }) {
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

