import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import type { ConfirmTerminationRow, TerminationPreviewResponse } from "@/lib/api/employees";

import { toast } from "@/components/ui/toast-imperative";
import {
  confirmTerminationImport,
  uploadTerminationImportPreview,
} from "@/lib/api/employees";
import { fetchJobStatus } from "@/lib/api/leaves";
import { logError } from "@/lib/logger";

const POLL_INTERVAL_MS = 2000;
const MAX_POLLS = 60;

/**
 * Owns the upload → preview → confirm → poll state machine for bulk
 * termination imports, plus the close/reset transition. UI concerns that do
 * not mutate state (template download, error export) stay in the composing
 * `useTerminationImport` hook.
 */
export function useImportStateMachine(wsId: string, onClose: () => void) {
  const { t } = useTranslation("employees");
  const queryClient = useQueryClient();

  const [file, setFile] = useState<File | null>(null);
  const [loading, setLoading] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const [step, setStep] = useState<1 | 2>(1);
  const [previewData, setPreviewData] = useState<TerminationPreviewResponse | null>(null);
  const [activeTab, setActiveTab] = useState<"valid" | "invalid">("valid");
  const [importSuccess, setImportSuccess] = useState(false);
  const [successCount, setSuccessCount] = useState(0);
  const [jobId, setJobId] = useState<string | null>(null);
  const [, setJobStatus] = useState<string | null>(null);
  const [jobError, setJobError] = useState<string | null>(null);

  const handleUpload = async () => {
    if (!file) return;
    setLoading(true);
    try {
      const data = await uploadTerminationImportPreview(wsId, file);
      setPreviewData(data);
      setStep(2);
      setActiveTab(data.valid_rows.length > 0 ? "valid" : "invalid");
    } catch (err) {
      logError(err, { component: "EmployeeTerminationImport" });
      toast.error(t("employees.bulk_termination.upload_error", { defaultValue: "Failed to upload file" }));
    } finally {
      setLoading(false);
    }
  };

  const pollJob = async (id: string, initialCount: number) => {
    for (let i = 0; i < MAX_POLLS; i++) {
      try {
        const statusRes = await fetchJobStatus(wsId, id);
        setJobStatus(statusRes.status);
        if (statusRes.status === "completed") {
          setSuccessCount(initialCount);
          setImportSuccess(true);
          void queryClient.invalidateQueries();
          setConfirming(false);
          setJobId(null);
          return;
        }
        if (statusRes.status === "failed") {
          setJobError(statusRes.error_message ?? "Đã xảy ra lỗi khi xử lý nghỉ việc.");
          setConfirming(false);
          setJobId(null);
          return;
        }
      } catch (err) {
        logError(err, { component: "EmployeeTerminationImport" });
      }
      await new Promise((resolve) => setTimeout(resolve, POLL_INTERVAL_MS));
    }
    setJobError("Hết thời gian chờ xử lý tác vụ nghỉ việc.");
    setConfirming(false);
    setJobId(null);
  };

  const handleConfirm = async () => {
    if (!previewData || previewData.valid_rows.length === 0) return;
    setConfirming(true);
    setJobError(null);
    try {
      const rows: ConfirmTerminationRow[] = previewData.valid_rows.map((row) => ({
        employment_id: row.employment_id,
        termination_date: row.termination_date,
        termination_reason: row.termination_reason,
        termination_notes: row.termination_notes,
      }));
      const res = await confirmTerminationImport(wsId, rows);
      if (res.job_id) {
        setJobId(res.job_id);
        setJobStatus("queued");
        void pollJob(res.job_id, res.success_count);
      } else {
        setSuccessCount(res.success_count);
        setImportSuccess(true);
        void queryClient.invalidateQueries();
        setConfirming(false);
      }
    } catch (err) {
      logError(err, { component: "EmployeeTerminationImport" });
      toast.error(t("employees.bulk_termination.confirm_error", { defaultValue: "Connection error." }));
      setJobError("Lỗi kết nối hoặc yêu cầu không hợp lệ.");
      setConfirming(false);
    }
  };

  const handleClose = () => {
    setStep(1);
    setFile(null);
    setPreviewData(null);
    setImportSuccess(false);
    setSuccessCount(0);
    setJobId(null);
    setJobStatus(null);
    setJobError(null);
    onClose();
  };

  return {
    file, setFile,
    loading, confirming,
    step, previewData,
    activeTab, setActiveTab,
    importSuccess, successCount,
    jobId, jobError,
    setJobError,
    handleUpload,
    handleConfirm,
    handleClose,
  };
}
