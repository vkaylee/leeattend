import { useQuery } from "@tanstack/react-query";
import {
  CheckCircle2,
  Clock,
  GitBranch,
  History,
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import type { ShiftVersionHistoryItem } from "@/lib/api/types";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
import { getShiftVersions } from "@/lib/api/shifts";

import { IntervalDiff } from "./shift-revision-diff";

// ── Types ──────────────────────────────────────────────────────────

interface ShiftRevisionHistoryDialogProps {
  isOpen: boolean;
  onClose: () => void;
  shiftId: string;
  shiftName: string;
  wsId: string;
}

// ── Badge helpers ──────────────────────────────────────────────────

const BADGE_STYLES: Record<string, string> = {
  correction:
    "bg-amber-50 dark:bg-amber-950/30 text-amber-600 dark:text-amber-400 border-amber-100 dark:border-amber-900",
  create:
    "bg-blue-50 dark:bg-blue-950/30 text-blue-600 dark:text-blue-400 border-blue-100 dark:border-blue-900",
  revision:
    "bg-emerald-50 dark:bg-emerald-950/30 text-emerald-600 dark:text-emerald-400 border-emerald-100 dark:border-emerald-900",
};

function changeTypeBadge(changeType: string, t: (k: string) => string): { label: string; style: string } {
  const key = changeType.toLowerCase();
  if (key === "correct" || key === "correction") {
    return { label: t("shifts.revisions.badge_correction"), style: BADGE_STYLES.correction };
  }
  if (key === "revise" || key === "revision") {
    return { label: t("shifts.revisions.badge_revision"), style: BADGE_STYLES.revision };
  }
  return { label: t("shifts.revisions.badge_create"), style: BADGE_STYLES.create };
}

// ── Sub-components ─────────────────────────────────────────────────

function LoadingSkeleton() {
  return (
    <div className="flex flex-col gap-4 p-4 animate-pulse">
      {[1, 2, 3].map((i) => (
        <div className="flex gap-4" key={i}>
          <div className="w-3 h-20 bg-slate-200 dark:bg-slate-800 rounded-full shrink-0" />
          <div className="flex-1 space-y-3 py-1">
            <div className="h-4 bg-slate-200 dark:bg-slate-800 rounded w-1/4" />
            <div className="h-3 bg-slate-100 dark:bg-slate-900 rounded w-1/2" />
            <div className="h-3 bg-slate-100 dark:bg-slate-900 rounded w-3/4" />
          </div>
        </div>
      ))}
    </div>
  );
}

function EmptyState({ message }: { message: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-12 px-6 text-center">
      <div className="w-16 h-16 rounded-2xl bg-slate-100 dark:bg-slate-800 flex items-center justify-center mb-4">
        <GitBranch className="w-8 h-8 text-slate-300 dark:text-slate-600" />
      </div>
      <Typography
        className="text-sm font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest"
        variant="p"
      >
        {message}
      </Typography>
    </div>
  );
}

interface VersionCardProps {
  isLatest: boolean;
  previous: ShiftVersionHistoryItem | null;
  t: (key: string, fallback?: string) => string;
  version: ShiftVersionHistoryItem;
}

function VersionCard({ isLatest, previous, t, version }: VersionCardProps) {
  const badge = changeTypeBadge(version.change_type, t);
  const effectiveTo = version.effective_to
    ? format(new Date(version.effective_to), "MMM dd, yyyy")
    : t("shifts.revisions.effective_present", "Present");

  return (
    <div className="relative flex gap-5 group">
      {/* Timeline Dot */}
      <div
        className={`
          relative z-10 w-3 h-3 mt-1.5 rounded-full shrink-0 border-2 transition-colors
          ${isLatest
            ? "bg-primary-500 border-white dark:border-slate-900 shadow-[0_0_0_4px_rgba(59,130,246,0.15)]"
            : "bg-slate-300 dark:bg-slate-600 border-white dark:border-slate-900 group-hover:bg-primary-300"
          }
        `}
      />

      {/* Content Card */}
      <div
        className={`
          flex-1 p-4 rounded-2xl border transition-all duration-200
          ${isLatest
            ? "bg-white dark:bg-slate-900 border-primary-100 dark:border-primary-900/50 shadow-sm shadow-primary-500/5"
            : "bg-slate-50 dark:bg-slate-900/50 border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900"
          }
        `}
      >
        {/* Header: version number + badges */}
        <div className="flex items-center gap-2 mb-1 flex-wrap">
          <span className="font-bold text-slate-700 dark:text-slate-200 text-sm">
            v{version.version_number}
          </span>
          <span
            className={`inline-flex items-center px-1.5 py-0.5 rounded-md text-[9px] font-black tracking-widest uppercase border ${badge.style}`}
          >
            {badge.label}
          </span>
          {isLatest ? (
            <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-md bg-primary-50 dark:bg-primary-950/30 text-primary-600 dark:text-primary-400 text-[9px] font-black tracking-widest uppercase">
              <CheckCircle2 className="w-3 h-3" />
              {t("shifts.revisions.active_version", "Active")}
            </span>
          ) : null}
        </div>

        {/* Effective date range */}
        <div className="flex items-center gap-1.5 text-xs text-slate-500 dark:text-slate-400 mb-2">
          <Clock className="w-3 h-3" />
          <span>
            {format(new Date(version.effective_from), "MMM dd, yyyy")}
            {" — "}
            {effectiveTo}
          </span>
        </div>

        {/* Author + timestamp */}
        <Typography
          className="text-xs text-slate-400 dark:text-slate-500 font-medium"
          variant="small"
        >
          {format(new Date(version.created_at), "MMM dd, yyyy 'at' HH:mm a")}
        </Typography>

        {/* Change reason quote */}
        {version.change_reason ? (
          <div className="mt-3 pl-3 border-l-2 border-slate-200 dark:border-slate-700 italic text-xs text-slate-500 dark:text-slate-400 leading-relaxed">
            {version.change_reason}
          </div>
        ) : null}

        {/* Interval diff */}
        <IntervalDiff current={version} previous={previous} t={t} />
      </div>
    </div>
  );
}

// ── Main Component ─────────────────────────────────────────────────

export function ShiftRevisionHistoryDialog({
  isOpen,
  onClose,
  shiftId,
  shiftName,
  wsId,
}: ShiftRevisionHistoryDialogProps) {
  const { t } = useTranslation();

  const {
    data: versions,
    isLoading,
  } = useQuery({
    queryKey: ["shift-versions", wsId, shiftId],
    queryFn: () => getShiftVersions(wsId, shiftId),
    enabled: isOpen && Boolean(wsId && shiftId),
    staleTime: 30_000,
  });

  return (
    <Modal
      icon={<History className="w-5 h-5 text-primary-500" />}
      isOpen={isOpen}
      maxWidth="2xl"
      onClose={onClose}
      subtitle={t("shifts.revisions.subtitle", "Revision history for {{name}}", { name: shiftName })}
      title={t("shifts.revisions.title", "Version History")}
    >
      <div className="px-1 py-2">
        {/* Loading */}
        {isLoading ? <LoadingSkeleton /> : null}

        {/* Empty */}
        {!isLoading && (!versions || versions.length === 0) ? (
          <EmptyState message={t("shifts.revisions.empty_state", "No version history available for this shift.")} />
        ) : null}

        {/* Timeline */}
        {!isLoading && versions && versions.length > 0 ? (
          <div className="relative pl-3">
            {/* Vertical line */}
            <div className="absolute left-[15px] top-2 bottom-2 w-0.5 bg-slate-200 dark:bg-slate-800 rounded-full" />

            <div className="space-y-6">
              {versions.map((version, idx) => (
                <VersionCard
                  isLatest={idx === 0}
                  key={version.id}
                  previous={versions[idx + 1] ?? null}
                  t={t}
                  version={version}
                />
              ))}
            </div>
          </div>
        ) : null}
      </div>

      {/* Footer */}
      <div className="flex items-center justify-end px-4 py-3 border-t border-slate-100 dark:border-slate-800">
        <Button
          className="px-4 py-2 text-[10px] font-bold uppercase tracking-widest text-slate-500 hover:text-slate-700 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-xl transition-all cursor-pointer"
          onClick={onClose}
          type="button"
          variant="ghost"
        >
          {t("common.close")}
        </Button>
      </div>
    </Modal>
  );
}
