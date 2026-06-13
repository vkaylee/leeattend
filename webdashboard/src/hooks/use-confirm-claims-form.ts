import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import type { DailyAttendanceReport } from "@/lib/api";

import { useToast } from "@/components/ui/toast";
import { useClaimOt, useEnterPaperOt } from "@/hooks/use-attendance";

type ReportRow = DailyAttendanceReport & { employee_id_number?: string };

interface ClaimItem {
  claimed_minutes: number;
  computed_minutes: number;
  date: string;
  employee_id: string;
  employee_id_number: string;
  employee_name: string;
  employment_id: string;
  reason: string;
}

interface UseConfirmClaimsFormProps {
  initialReasons?: Record<string, string>;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
  open: boolean;
  selectedRows: ReportRow[];
}

export function useConfirmClaimsForm({
  open,
  onOpenChange,
  selectedRows,
  initialReasons = {},
  onSuccess,
}: UseConfirmClaimsFormProps) {
  const { t } = useTranslation(["attendance", "common"]);
  const toast = useToast();

  const { enterPaperOt, isSubmitting: isSubmittingPaper } = useEnterPaperOt();
  const { claimOt, isSubmitting: isSubmittingClaim } = useClaimOt();

  const isSubmitting = isSubmittingPaper || isSubmittingClaim;

  const [claims, setClaims] = useState<ClaimItem[]>([]);
  const [photoUrl, setPhotoUrl] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);
  const [showConfirmDiscrepancies, setShowConfirmDiscrepancies] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Derived state sync when open state changes (outside useEffect to prevent cascade render warnings)
  const [prevOpen, setPrevOpen] = useState(open);
  if (open !== prevOpen) {
    setPrevOpen(open);
    if (open) {
      setClaims(
        selectedRows.map((r) => {
          const key = `${r.employment_id}-${r.date}`;
          const computed_minutes = Math.round((r.computed_ot_hours) * 60);
          return {
            employee_id: r.employee_id,
            employment_id: r.employment_id,
            date: r.date,
            employee_name: r.employee_name,
            employee_id_number: r.employee_id_number ?? "",
            computed_minutes,
            claimed_minutes: computed_minutes,
            reason: initialReasons[key] ?? t("ot_approvals.delegated_claim.default_reason"),
          };
        })
      );
      setPhotoUrl(null);
      setFileName(null);
      setShowConfirmDiscrepancies(false);
      setError(null);
    }
  }

  const handleMinutesChange = (index: number, val: number) => {
    setClaims((prev) =>
      prev.map((c, i) => (i === index ? { ...c, claimed_minutes: val } : c))
    );
  };

  const handleReasonChange = (index: number, val: string) => {
    setClaims((prev) =>
      prev.map((c, i) => (i === index ? { ...c, reason: val } : c))
    );
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setFileName(file.name);
      setPhotoUrl(`/uploads/mock_${file.name}`);
    }
  };

  const validateForm = () => {
    const hasInvalidMinutes = claims.some((c) => c.claimed_minutes <= 0);
    if (hasInvalidMinutes) {
      setError(t("approvals.paper_ot.error_minutes"));
      return false;
    }

    const hasEmptyReason = claims.some((c) => !c.reason.trim());
    if (hasEmptyReason) {
      setError(t("approvals.paper_ot.error_reason"));
      return false;
    }

    return true;
  };

  const handleSubmit = async (e?: React.SyntheticEvent) => {
    e?.preventDefault();
    setError(null);

    if (!validateForm()) return;

    // Check for discrepancies
    const discrepanciesList = claims.filter(
      (c) => c.claimed_minutes !== c.computed_minutes
    );

    if (discrepanciesList.length > 0 && !showConfirmDiscrepancies) {
      setShowConfirmDiscrepancies(true);
      return;
    }

    try {
      const promises = claims.map((c) => {
        if (photoUrl) {
          return enterPaperOt({
            employeeId: c.employee_id,
            date: c.date,
            claimed_minutes: c.claimed_minutes,
            reason: c.reason,
            paper_form_photo_url: photoUrl,
          });
        } else {
          return claimOt({
            employeeId: c.employee_id,
            date: c.date,
            claimed_minutes: c.claimed_minutes,
            reason: c.reason,
          });
        }
      });

      await Promise.all(promises);

      toast.success(t("ot_approvals.delegated_claim.messages.success"));
      onOpenChange(false);
      onSuccess?.();
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "Failed to claim overtime";
      setError(message);
    }
  };

  return {
    claims,
    photoUrl,
    fileName,
    showConfirmDiscrepancies,
    setShowConfirmDiscrepancies,
    error,
    setError,
    isSubmitting,
    handleMinutesChange,
    handleReasonChange,
    handleFileChange,
    handleSubmit,
    setPhotoUrl,
    setFileName,
  };
}
