import { AlertTriangle, Clock, Gauge } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import type { ComplianceViolation } from "@/lib/api/errors";

import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { Skeleton } from "@/components/ui/skeleton";
import { Typography } from "@/components/ui/typography";

interface ComplianceWarningModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: (overrideReason: string) => void;
  violations: ComplianceViolation[];
  isLoading: boolean;
}

const VIOLATION_STYLES: Record<
  ComplianceViolation["violation_type"],
  { icon: React.ComponentType<{ size?: number; className?: string }>; classes: string }
> = {
  overlap: {
    icon: AlertTriangle,
    classes:
      "bg-rose-50 text-rose-700 border-rose-200 dark:bg-rose-950/50 dark:text-rose-300 dark:border-rose-800/50",
  },
  rest_period: {
    icon: Clock,
    classes:
      "bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-950/50 dark:text-amber-300 dark:border-amber-800/50",
  },
  weekly_hours: {
    icon: Gauge,
    classes:
      "bg-orange-50 text-orange-700 border-orange-200 dark:bg-orange-950/50 dark:text-orange-300 dark:border-orange-800/50",
  },
  monthly_ot: {
    icon: Gauge,
    classes:
      "bg-slate-50 text-slate-700 border-slate-200 dark:bg-slate-800/50 dark:text-slate-300 dark:border-slate-700/50",
  },
};

/** Map violation_type to an i18n key fragment for the label. */
function getViolationLabelI18nKey(type: ComplianceViolation["violation_type"]): string {
  switch (type) {
    case "overlap":
      return "shifts.compliance.violation_overlap";
    case "rest_period":
      return "shifts.compliance.violation_rest_period";
    case "weekly_hours":
      return "shifts.compliance.violation_weekly_hours";
    case "monthly_ot":
      return "shifts.compliance.violation_monthly_ot";
  }
}

/** Attempt to interpolate i18n template params if present on the violation message. */
function interpolateMessage(t: (key: string, options?: Record<string, unknown>) => string, violation: ComplianceViolation): string {
  const i18nKey = getViolationLabelI18nKey(violation.violation_type);
  const details = violation.details;
  // Build interpolation params if the i18n key expects them (rest_period, weekly_hours, monthly_ot)
  if (violation.violation_type === "rest_period") {
    return t(i18nKey, { gap: details.total_hours, required: details.limit_hours });
  }
  if (violation.violation_type === "weekly_hours" || violation.violation_type === "monthly_ot") {
    return t(i18nKey, { current: details.total_hours, limit: details.limit_hours });
  }
  // Fallback: try i18n, then the raw message from the backend
  return t(i18nKey, { defaultValue: violation.message });
}

export function ComplianceWarningModal({
  isOpen,
  onClose,
  onConfirm,
  violations,
  isLoading,
}: ComplianceWarningModalProps) {
  const { t } = useTranslation();
  const [overrideReason, setOverrideReason] = useState("");

  const handleConfirm = () => {
    const trimmed = overrideReason.trim();
    if (!trimmed || isLoading) return;
    onConfirm(trimmed);
  };

  const handleClose = () => {
    if (isLoading) return;
    setOverrideReason("");
    onClose();
  };

  const handleReasonChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    setOverrideReason(e.target.value);
  };

  // ── Body Content ──────────────────────────────────────────────────────

  const renderBody = () => {
    // Loading skeleton
    if (isLoading) {
      return (
        <div className="space-y-3" data-testid="compliance-loading">
          <Skeleton className="h-14 w-full rounded-xl" />
          <Skeleton className="h-14 w-full rounded-xl" />
          <Skeleton className="h-14 w-full rounded-xl" />
          <Skeleton className="h-24 w-full rounded-2xl mt-4" />
        </div>
      );
    }

    // Empty state (edge case — should not normally happen)
    if (violations.length === 0) {
      return (
        <div className="flex flex-col items-center justify-center py-8 text-center">
          <Typography variant="body" className="text-slate-400 dark:text-slate-500">
            {t("shifts.compliance.no_violations", "No compliance issues to display.")}
          </Typography>
        </div>
      );
    }

    return (
      <>
        {/* Violation list */}
        <div
          className="space-y-2 max-h-48 overflow-y-auto pr-1 roster-scrollbar"
          id="compliance-violation-list"
          role="list"
          aria-label={t("shifts.compliance.warning_title")}
        >
          {violations.map((v, idx) => {
            const style = VIOLATION_STYLES[v.violation_type];
            const Icon = style.icon;
            const label = interpolateMessage(t, v);

            return (
              <div
                key={`${v.violation_type}-${idx}`}
                className={`flex items-start gap-3 rounded-xl border px-4 py-3 text-sm ${style.classes}`}
                role="listitem"
              >
                <Icon className="mt-0.5 flex-shrink-0" size={16} />
                <span className="font-medium leading-snug">{label}</span>
              </div>
            );
          })}
        </div>

        {/* Override reason textarea */}
        <FormField
          aria-describedby="compliance-violation-list"
          data-testid="compliance-override-reason"
          id="compliance-override-reason"
          label={t("shifts.compliance.override_reason_label")}
          onChange={handleReasonChange}
          placeholder={t("shifts.compliance.override_reason_placeholder")}
          required
          rows={4}
          type="textarea"
          value={overrideReason}
        />
      </>
    );
  };

  // ── Render ────────────────────────────────────────────────────────────

  return (
    <Modal
      icon={
        <AlertTriangle className="h-5 w-5 text-amber-500" size={24} />
      }
      isOpen={isOpen}
      maxWidth="lg"
      onClose={handleClose}
      subtitle={violations.length > 0
        ? t("shifts.compliance.violation_count", { count: violations.length, defaultValue: `{{count}} violation(s)` })
        : undefined}
      testId="compliance-warning-modal"
      title={t("shifts.compliance.warning_title")}
    >
      <div className="flex flex-col gap-4">
        {renderBody()}

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800">
          <Button
            aria-label={t("shifts.compliance.cancel")}
            disabled={isLoading}
            onClick={handleClose}
            variant="outline"
          >
            {t("shifts.compliance.cancel")}
          </Button>
          <Button
            aria-label={t("shifts.compliance.confirm_override")}
            disabled={!overrideReason.trim() || isLoading}
            loading={isLoading}
            onClick={handleConfirm}
            variant="primary"
          >
            {t("shifts.compliance.confirm_override")}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
