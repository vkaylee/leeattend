import { useQuery } from "@tanstack/react-query";
import { CalendarDays, Check, Info, Users } from "lucide-react";
import { useTranslation } from "react-i18next";

import { EmployeeSearch } from "@/components/common/employee-search";
import Button from "@/components/ui/button";
import { ComplianceAlertBadge } from "@/components/ui/compliance-alert-badge";
import { INPUT_BASE, LABEL_BASE } from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { PatternMatrixPreview } from "@/components/ui/pattern-matrix-preview";
import { Typography } from "@/components/ui/typography";
import { usePatternCompliance } from "@/hooks/attendance/use-pattern-compliance";
import { fetchPatternDetails } from "@/lib/api";
import { WorkingPatternDetail } from "@/lib/api/types";
import { cn } from "@/lib/utils";
import { type ComplianceIssue } from "@/lib/utils/shift-compliance-utils";

import { useWorkingPatterns } from "../../hooks/use-working-patterns";
import { createQueryKey } from "@/lib/query-keys";


// ── Helpers ────────────────────────────────────────────────────────────

function getComplianceBadge(issue: ComplianceIssue, t: (k: string, options?: Record<string, unknown>) => string): { badgeType: "overlap" | "rest" | "hours"; msg: string } {
  if (issue.type === "overlap") {
    return { badgeType: "overlap", msg: t("patterns.conflicts.overlap", { shiftA: issue.shiftA, shiftB: issue.shiftB }) };
  }
  if (issue.type === "rest") {
    return { badgeType: "rest", msg: t("patterns.conflicts.rest_warning", { hours: issue.restHours }) };
  }
  return { badgeType: "hours", msg: t("patterns.conflicts.weekly_hours_exceeded", { total: Math.round(issue.totalMinutes / 60), limit: Math.round(issue.limitMinutes / 60) }) };
}

// ── AnchorDateSection ──────────────────────────────────────────────────

function AnchorDateSection({
  isRotation,
  assignAnchorDate,
  setAssignAnchorDate,
}: {
  isRotation: boolean;
  assignAnchorDate: string;
  setAssignAnchorDate: (d: string) => void;
}) {
  const { t } = useTranslation();
  if (!isRotation) return null;
  return (
    <div className="space-y-4 animate-in fade-in slide-in-from-top-2 duration-300">
      <label className={LABEL_BASE}>{t("patterns.form.anchor_date_label").toUpperCase()}</label>
      <div className="relative group">
        <div className="absolute inset-y-0 left-0 pl-5 flex items-center pointer-events-none text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors"><CalendarDays size={18} /></div>
        <input className={cn(INPUT_BASE, "pl-14 font-bold tracking-tight shadow-inner")} onChange={(e) => { setAssignAnchorDate(e.target.value); }} type="date" value={assignAnchorDate} />
      </div>
      <Typography className="text-[9px] text-slate-500 dark:text-slate-400 font-bold uppercase italic leading-relaxed px-1 opacity-60" variant="p">
        {t("patterns.form.anchor_date_note") || "Anchor Date determines Day 1 of the rotating cycle."}
      </Typography>
    </div>
  );
}

interface BulkAssignModalProps {
  hook: ReturnType<typeof useWorkingPatterns>;
}

export function BulkAssignModal({ hook }: BulkAssignModalProps) {
  const { t } = useTranslation();
  const {
    assignStartDate,
    handleBulkAssign,
    isBulkModalOpen,
    patterns,
    selectedEmployeeIds,
    selectedPatternId,
    setAssignStartDate,
    setIsBulkModalOpen,
    setSelectedEmployeeIds,
    setSelectedPatternId,
    assignAnchorDate,
    setAssignAnchorDate,
    departments,
    shifts,
  } = hook;

  const selectedPattern = patterns.find((p) => p.id === selectedPatternId);
  const isRotation = selectedPattern?.pattern_type === "rotation";

  // Fetch pattern details for visual preview
  const { data: previewDetails } = useQuery<WorkingPatternDetail[]>({
    queryKey: createQueryKey("pattern-preview", selectedPatternId),
    queryFn: () => fetchPatternDetails(selectedPatternId),
    enabled: !!selectedPatternId,
  });

  // Compute client-side compliance issues from preview details
  const complianceIssues = usePatternCompliance(previewDetails, shifts);

  const hasConflicts = complianceIssues.length > 0;

  return (
    <Modal icon={<Users size={24} />} isOpen={isBulkModalOpen} maxWidth="2xl" onClose={() => { setIsBulkModalOpen(false); }} subtitle={t("patterns.modal.bulk_subtitle")} title={t("patterns.bulk_assign_btn")}>
      <div className="space-y-10 pt-4 animate-in fade-in duration-500">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-10 px-1">
          <div className="space-y-4">
            <label className={LABEL_BASE}>{t("patterns.form.select_pattern_label").toUpperCase()}</label>
            <div className="grid grid-cols-1 gap-3 max-h-80 overflow-y-auto pr-2 scrollbar-hide">
              {patterns.map((p) => (
                <Button className={cn("flex items-center justify-between p-5 rounded-2xl border transition-all text-left group", selectedPatternId === p.id ? "bg-white dark:bg-slate-900 border-primary-600 shadow-xl shadow-primary-100 dark:shadow-black/30 ring-2 ring-primary-600/10" : "bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900 hover:border-primary-200 dark:border-primary-800")} key={p.id} onClick={() => { setSelectedPatternId(p.id); }} type="button" variant="ghost">
                  <div className="flex items-center gap-4">
                    <div className={cn("w-10 h-10 rounded-xl flex items-center justify-center transition-colors", selectedPatternId === p.id ? "bg-primary-600 text-white" : "bg-white dark:bg-slate-900 text-slate-300 dark:text-slate-600 border border-slate-100 dark:border-slate-800 group-hover:text-primary-500 dark:group-hover:text-primary-400")}><CalendarDays size={20} /></div>
                    <div>
                      <span className={cn("font-bold text-xs uppercase tracking-tight block", selectedPatternId === p.id ? "text-slate-700 dark:text-slate-200" : "text-slate-500 dark:text-slate-400")}>{p.name}</span>
                      <span className="text-[9px] text-slate-400 dark:text-slate-500 font-medium">
                        {p.pattern_type === "rotation"
                          ? `${t("patterns.form.pattern_type_rotation") || "Rotation"} (${p.cycle_length_days}d)`
                          : t("patterns.form.pattern_type_weekly") || "Weekly"}
                      </span>
                    </div>
                  </div>
                  {selectedPatternId === p.id && <Check className="text-primary-600 dark:text-primary-400" size={18} />}
                </Button>
              ))}
            </div>
          </div>
          <div className="space-y-6">
            <div className="space-y-4">
              <label className={LABEL_BASE}>{t("patterns.form.start_date_label").toUpperCase()}</label>
              <div className="relative group">
                <div className="absolute inset-y-0 left-0 pl-5 flex items-center pointer-events-none text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors"><Info size={18} /></div>
                <input className={cn(INPUT_BASE, "pl-14 font-bold tracking-tight shadow-inner")} onChange={(e) => { setAssignStartDate(e.target.value); }} type="date" value={assignStartDate} />
              </div>
              <Typography className="text-[9px] text-slate-500 dark:text-slate-400 font-bold uppercase italic leading-relaxed px-1 opacity-60" variant="p">{t("patterns.form.start_date_note")}</Typography>
            </div>

            <AnchorDateSection
              assignAnchorDate={assignAnchorDate}
              isRotation={isRotation}
              setAssignAnchorDate={setAssignAnchorDate}
            />
          </div>
        </div>

        {/* Live pattern preview with compliance alerts */}
        {!!selectedPatternId && (
          <div className="space-y-4 px-1 animate-in fade-in slide-in-from-top-2 duration-300">
            {/* Compliance badges */}
            {hasConflicts ? (
              <div className="flex flex-wrap gap-2">
                {complianceIssues.map((issue, i) => {
                  const { badgeType, msg } = getComplianceBadge(issue, t);
                  return (
                    <ComplianceAlertBadge
                      key={i}
                      message={msg}
                      type={badgeType}
                      variant="badge"
                    />
                  );
                })}
              </div>
            ) : null}

            {/* Matrix preview */}
            <div className="bg-slate-50 dark:bg-slate-800/40 rounded-2xl p-4 border border-slate-100 dark:border-slate-700/50">
              <div className="flex items-center gap-2 mb-3">
                <CalendarDays
                  className="text-primary-500 dark:text-primary-400"
                  size={14}
                />
                <span className="text-[10px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">
                  {selectedPattern?.name}
                </span>
              </div>
              <PatternMatrixPreview
                cycleLength={selectedPattern?.cycle_length_days ?? 7}
                details={previewDetails}
                isLoading={!previewDetails}
                patternType={isRotation ? "rotation" : "weekly"}
                shifts={shifts}
                variant="modal"
              />
            </div>
          </div>
        )}

        <div className="space-y-4 px-1">
          <label className={LABEL_BASE}>{t("patterns.form.select_employees_label").toUpperCase()}</label>
          <EmployeeSearch className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 rounded-[2rem] p-8 shadow-inner" departments={departments} multiple={true} onChange={(v) => { setSelectedEmployeeIds(v as string[]); }} value={selectedEmployeeIds}  />
        </div>
        <div className="pt-8 border-t border-slate-50 dark:border-slate-800 grid grid-cols-2 gap-6">
          <Button onClick={() => { setIsBulkModalOpen(false); }} size="lg" variant="outline">{t("common.cancel").toUpperCase()}</Button>
          <Button disabled={!selectedPatternId || selectedEmployeeIds.length === 0} onClick={() => { handleBulkAssign(); }} size="lg" variant="secondary">{t("patterns.modal.assign_btn").toUpperCase()}</Button>
        </div>
      </div>
    </Modal>
  );
}
