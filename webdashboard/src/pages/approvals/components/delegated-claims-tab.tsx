import { Check, CheckCircle, Search } from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { ConfirmClaimsDialog } from "@/components/approvals/confirm-claims-dialog";
import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import type { DailyAttendanceReport } from "@/lib/api";
import { cn } from "@/lib/utils";

type ReportRow = DailyAttendanceReport & { employee_id_number?: string };

interface DelegatedClaimsTabProps {
  reportData: ReportRow[];
  isLoading: boolean;
  refetch: () => Promise<unknown>;
}

function LoadingState() {
  const { t } = useTranslation();
  return (
    <div className="p-32 flex flex-col items-center gap-6 animate-pulse">
      <div className="w-12 h-12 border-4 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
      <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em]">
        {t("common.loading")}
      </span>
    </div>
  );
}

export function DelegatedClaimsTab({
  reportData,
  isLoading,
  refetch,
}: DelegatedClaimsTabProps) {
  const { t } = useTranslation(["common", "attendance"]);
  const toast = useToast();
  const [confirmClaimsOpen, setConfirmClaimsOpen] = useState(false);
  const [activeRowsToClaim, setActiveRowsToClaim] = useState<ReportRow[]>([]);

  const claimableRows = reportData.filter(
    (r) => r.ot_claim_status === "pending_employee" && r.computed_ot_hours > 0
  );

  const [searchQuery, setSearchQuery] = useState("");
  const [selectedRows, setSelectedRows] = useState<Set<string>>(new Set());
  const [reasons, setReasons] = useState<Record<string, string>>({});

  const filteredRows = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();
    if (!query) return claimableRows;
    return claimableRows.filter((row) => {
      const nameMatch = row.employee_name.toLowerCase().includes(query);
      const idMatch = row.employee_id_number?.toLowerCase().includes(query);
      return nameMatch || idMatch;
    });
  }, [claimableRows, searchQuery]);

  const toggleSelectRow = (key: string) => {
    const next = new Set(selectedRows);
    if (next.has(key)) {
      next.delete(key);
    } else {
      next.add(key);
    }
    setSelectedRows(next);
  };

  const isAllFilteredSelected = useMemo(() => {
    if (filteredRows.length === 0) return false;
    return filteredRows.every((r) => selectedRows.has(`${r.employment_id}-${r.date}`));
  }, [filteredRows, selectedRows]);

  const toggleSelectAll = () => {
    const next = new Set(selectedRows);
    if (isAllFilteredSelected) {
      for (const r of filteredRows) {
        next.delete(`${r.employment_id}-${r.date}`);
      }
    } else {
      for (const r of filteredRows) {
        next.add(`${r.employment_id}-${r.date}`);
      }
    }
    setSelectedRows(next);
  };

  const handleReasonChange = (key: string, val: string) => {
    setReasons({ ...reasons, [key]: val });
  };

  const handleClaim = (row: ReportRow) => {
    setActiveRowsToClaim([row]);
    setConfirmClaimsOpen(true);
  };

  const handleBulkClaim = () => {
    if (selectedRows.size === 0) {
      toast.error(t("ot_approvals.delegated_claim.messages.select_at_least_one", { defaultValue: "Please select at least one entry" }));
      return;
    }

    const selectedList = claimableRows.filter((r) =>
      selectedRows.has(`${r.employment_id}-${r.date}`)
    );
    setActiveRowsToClaim(selectedList);
    setConfirmClaimsOpen(true);
  };

  if (isLoading) {
    return <LoadingState />;
  }

  if (claimableRows.length === 0) {
    return (
      <div className="py-40 text-center flex flex-col items-center border-4 border-dashed border-slate-50 dark:border-slate-800 m-8 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30">
        <div className="w-24 h-24 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-xl border border-slate-100 dark:border-slate-800 mb-8">
          <CheckCircle className="text-emerald-200" size={40} />
        </div>
        <Typography className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h3">
          {t("ot_approvals.delegated_claim.empty_state.title", { defaultValue: "No pending entries to claim" })}
        </Typography>
        <Typography className="text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-widest mt-2 opacity-50 italic px-6 max-w-md" variant="p">
          {t("ot_approvals.delegated_claim.empty_state.description", { defaultValue: "All overtime has been claimed, or no overtime hours were computed for the selected period." })}
        </Typography>
      </div>
    );
  }

  return (
    <div className="p-8 space-y-6">
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 border-b border-slate-50 dark:border-slate-800 pb-6 w-full">
        <div>
          <Typography className="text-lg font-bold text-slate-800 dark:text-slate-200" variant="h3">
            {t("ot_approvals.delegated_claim.title", { defaultValue: "Delegated Claims" })}
          </Typography>
          <Typography className="text-xs text-slate-500 mt-1" variant="p">
            {t("ot_approvals.delegated_claim.subtitle", { defaultValue: "Claim overtime hours on behalf of workshop/production personnel who lack system accounts" })}
          </Typography>
        </div>
        <div className="flex items-center gap-3">
          {selectedRows.size > 0 && (
            <Button
              className="normal-case tracking-normal gap-2 h-10 px-6 bg-primary-600 hover:bg-primary-700 text-white rounded-xl font-bold text-sm shadow-md transition-all duration-300 transform active:scale-95 disabled:opacity-50 animate-in fade-in"
              onClick={handleBulkClaim}
              variant="primary"
            >
              <Check className="w-4 h-4" />
              {t("ot_approvals.delegated_claim.actions.claim_selected", {
                count: selectedRows.size,
                defaultValue: `Claim ${selectedRows.size} Selected`
              })}
            </Button>
          )}
        </div>
      </div>

      <div className="flex flex-col sm:flex-row gap-4 items-stretch sm:items-center justify-between">
        <div className="relative group max-w-md w-full">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-455 dark:text-slate-500 w-4.5 h-4.5 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" />
          <input
            aria-label={t("common.search", { defaultValue: "Search..." })}
            className="w-full pl-11 pr-4 py-2.5 bg-slate-50/50 dark:bg-slate-800/30 border border-slate-200 dark:border-slate-700/50 rounded-xl outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-500/50 dark:focus:border-primary-400/50 transition-all text-xs font-semibold shadow-sm placeholder-slate-400"
            onChange={(e) => { setSearchQuery(e.target.value); }}
            placeholder={t("ot_approvals.delegated_claim.search_placeholder", { defaultValue: "Search by employee name or ID..." })}
            type="text"
            value={searchQuery}
          />
        </div>
        {!!searchQuery && (
          <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider bg-slate-100/80 dark:bg-slate-800/80 px-3 py-1.5 rounded-lg border border-slate-100 dark:border-slate-800/50 w-fit">
            {t("ot_approvals.delegated_claim.table.filtered_count", {
              count: filteredRows.length,
              defaultValue: `Found ${filteredRows.length} entries`
            })}
          </span>
        )}
      </div>

      <div className="overflow-x-auto rounded-3xl border border-slate-100 dark:border-slate-800">
        <table className="w-full border-collapse text-left">
          <thead>
            <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
              <th className="p-4 pl-6 w-12">
                <label className={cn(
                  "relative inline-flex items-center justify-center w-5 h-5 rounded border-2 transition-all cursor-pointer hover:border-primary-400",
                  isAllFilteredSelected
                    ? "bg-primary-500 border-primary-500 text-white"
                    : "bg-transparent border-slate-200 dark:border-slate-700 text-transparent"
                )}>
                  <input
                    checked={isAllFilteredSelected}
                    className="sr-only"
                    onChange={toggleSelectAll}
                    type="checkbox"
                  />
                  <Check className="w-3.5 h-3.5" strokeWidth={3} />
                </label>
              </th>
              <th className="p-4 font-bold text-xs text-slate-500 uppercase tracking-wider">
                {t("ot_approvals.delegated_claim.table.employee", { defaultValue: "Employee" })}
              </th>
              <th className="p-4 font-bold text-xs text-slate-500 uppercase tracking-wider">
                {t("ot_approvals.delegated_claim.table.date", { defaultValue: "Date" })}
              </th>
              <th className="p-4 font-bold text-xs text-slate-500 uppercase tracking-wider">
                {t("ot_approvals.delegated_claim.table.actual_hours", { defaultValue: "Actual Hours" })}
              </th>
              <th className="p-4 font-bold text-xs text-slate-500 uppercase tracking-wider">
                {t("ot_approvals.delegated_claim.table.computed_ot", { defaultValue: "System OT" })}
              </th>
              <th className="p-4 font-bold text-xs text-slate-500 uppercase tracking-wider">
                {t("ot_approvals.delegated_claim.table.reason", { defaultValue: "Claim Reason" })}
              </th>
              <th className="p-4 pr-6 text-right font-bold text-xs text-slate-500 uppercase tracking-wider">
                {t("ot_approvals.table.actions", { defaultValue: "Actions" })}
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
            {filteredRows.length === 0 ? (
              <tr>
                <td className="p-8 text-center text-slate-450 dark:text-slate-500 text-xs font-semibold" colSpan={7}>
                  {t("ot_approvals.delegated_claim.no_search_results", { defaultValue: "No matching entries found" })}
                </td>
              </tr>
            ) : (
              filteredRows.map((row) => {
                const key = `${row.employment_id}-${row.date}`;
                const isChecked = selectedRows.has(key);
                return (
                  <tr className="hover:bg-slate-50/50 dark:hover:bg-slate-800/20 transition-colors" key={key}>
                    <td className="p-4 pl-6">
                      <label className={cn(
                        "relative inline-flex items-center justify-center w-5 h-5 rounded border-2 transition-all cursor-pointer hover:border-primary-400",
                        isChecked
                          ? "bg-primary-500 border-primary-500 text-white"
                          : "bg-transparent border-slate-200 dark:border-slate-700 text-transparent"
                      )}>
                        <input
                          checked={isChecked}
                          className="sr-only"
                          onChange={() => { toggleSelectRow(key); }}
                          type="checkbox"
                        />
                        <Check className="w-3.5 h-3.5" strokeWidth={3} />
                      </label>
                    </td>
                    <td className="p-4">
                      <div className="flex flex-col">
                        <span className="font-bold text-sm text-slate-800 dark:text-slate-200">
                          {row.employee_name}
                        </span>
                        {!!row.employee_id_number && (
                          <span className="text-[10px] font-mono text-slate-400 dark:text-slate-500 uppercase tracking-tight">
                            {row.employee_id_number}
                          </span>
                        )}
                      </div>
                    </td>
                    <td className="p-4 text-sm text-slate-600 dark:text-slate-400">
                      {row.date}
                    </td>
                    <td className="p-4 text-sm text-slate-600 dark:text-slate-400">
                      {row.computed_std_hours ? `${row.computed_std_hours} hrs` : "-"}
                    </td>
                    <td className="p-4">
                      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-amber-50 dark:bg-amber-900/20 text-amber-800 dark:text-amber-300">
                        {row.computed_ot_hours} hrs
                      </span>
                    </td>
                    <td className="p-4">
                      <input
                        className="w-full max-w-xs px-3 py-1.5 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-700 rounded-lg text-xs text-slate-800 dark:text-slate-200 placeholder-slate-400 focus:outline-none focus:ring-1 focus:ring-primary-500 focus:border-primary-500 transition-all"
                        onChange={(e) => { handleReasonChange(key, e.target.value); }}
                        placeholder={t("ot_approvals.delegated_claim.table.reason", { defaultValue: "Enter reason..." })}
                        type="text"
                        value={reasons[key] ?? ""}
                      />
                    </td>
                    <td className="p-4 pr-6 text-right">
                      <Button
                        className="normal-case tracking-normal bg-slate-100 dark:bg-slate-800 hover:bg-primary-50 dark:hover:bg-primary-950/30 hover:text-primary-600 dark:hover:text-primary-400 text-slate-700 dark:text-slate-300 font-bold text-xs transition-all duration-300 disabled:opacity-50 shadow-none border-none"
                        onClick={() => { handleClaim(row); }}
                        size="sm"
                        variant="ghost"
                      >
                        {t("ot_approvals.delegated_claim.actions.claim", { defaultValue: "Claim" })}
                      </Button>
                    </td>
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>

      <ConfirmClaimsDialog
        initialReasons={reasons}
        onOpenChange={setConfirmClaimsOpen}
        onSuccess={() => {
          const claimedKeys = new Set(activeRowsToClaim.map((r) => `${r.employment_id}-${r.date}`));
          setSelectedRows((prev) => {
            const next = new Set(prev);
            for (const key of claimedKeys) {
              next.delete(key);
            }
            return next;
          });
          refetch().catch(console.error);
        }}
        open={confirmClaimsOpen}
        selectedRows={activeRowsToClaim}
      />
    </div>
  );
}
