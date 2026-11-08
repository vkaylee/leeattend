import { Upload } from "lucide-react";
import { useState, useEffect, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";

import type { LeaveImportJob } from "@/lib/api/leaves";

import Modal from "@/components/ui/modal";
import { uploadLeaveExcel, fetchLatestLeaveImport } from "@/lib/api";
import { logError } from "@/lib/logger";

import { LeaveImportProgressView } from "./leave-import-progress-view";
import { LeaveImportUploadView } from "./leave-import-upload-view";

interface LeaveImportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onImportComplete?: () => void;
}

function useLeaveImportStatus(isOpen: boolean, onImportComplete?: () => void) {
  const [importJob, setImportJob] = useState<LeaveImportJob | null>(null);
  const pollingRef = useRef<number | undefined>(undefined);

  // Check for latest import when dialog opens
  useEffect(() => {
    if (!isOpen) return;
    void fetchLatestLeaveImport().then((job) => {
      if (job) {
        const created = new Date(job.created_at);
        const now = new Date();
        if (now.getTime() - created.getTime() < 24 * 60 * 60 * 1000) {
          setImportJob(job);
        }
      }
    }).catch((e: unknown) => { logError(e, { component: 'LeaveImportDialog.fetchLatest' }); });
  }, [isOpen]);

  // Polling for job status
  const startPolling = useCallback(() => {
    clearInterval(pollingRef.current);
    pollingRef.current = window.setInterval(() => {
      void (async () => {
        try {
          const job = await fetchLatestLeaveImport();
          if (!job) return;
          setImportJob(job);
          if (job.status !== "processing" && job.status !== "pending") {
            window.clearInterval(pollingRef.current);
            pollingRef.current = undefined;
            onImportComplete?.();
          }
        } catch (e: unknown) {
          logError(e, { component: 'LeaveImportDialog.polling' });
        }
      })();
    }, 2000);
  }, [onImportComplete]);

  useEffect(() => {
    if (importJob?.status === "processing" || importJob?.status === "pending") {
      startPolling();
    }
    return () => {
      window.clearInterval(pollingRef.current);
    };
  }, [importJob?.status,startPolling]);

  const isProcessing = Boolean(importJob && (importJob.status === "processing" || importJob.status === "pending"));
  const isFinished = Boolean(importJob && (importJob.status === "completed" || importJob.status === "failed" || importJob.status === "partially_failed"));

  return { importJob, setImportJob, startPolling, isProcessing, isFinished };
}

function getErrorInfo(job: LeaveImportJob) {
  if (!job.error_log) return null;
  try {
    return JSON.parse(job.error_log) as { success_count?: number; error_count?: number; error_file_path?: string };
  } catch { /* JSON.parse on error_log — invalid JSON returns null gracefully */ return null; }
}

function useLeaveImportUpload({file,
  setFile,
  startPolling,
  setImportJob
}: {
  file: File | null;
  setFile: (file: File | null) => void;
  setImportJob: (job: LeaveImportJob) => void;
  startPolling: () => void;
}) {
  const [uploading, setUploading] = useState(false);

  const handleUpload = async () => {
    if (!file) return;
    setUploading(true);
    try {
      const result = await uploadLeaveExcel(file);
      setImportJob({
        id: result.job_id,
        status: "pending",
        total_rows: 0,
        processed_rows: 0,
        error_log: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });
      setFile(null);
      startPolling();
    } catch (e: unknown) {
      // Upload failure — error shown by parent toast handler
      logError(e, { component: 'LeaveImportDialog.upload' });
    } finally {
      setUploading(false);
    }
  };

  return { uploading, handleUpload };
}

export default function LeaveImportDialog({
  isOpen,
  onClose,
  onImportComplete,
}: LeaveImportDialogProps) {
  const { t, i18n } = useTranslation();
  const [file, setFile] = useState<File | null>(null);
  const [downloadingTemplate, setDownloadingTemplate] = useState(false);
  const { importJob, setImportJob, startPolling, isProcessing, isFinished } = useLeaveImportStatus(isOpen, onImportComplete);
  const { uploading, handleUpload } = useLeaveImportUpload({file, setFile, startPolling, setImportJob });

  // Reset state when dialog closes
  const [previousIsOpen, setPreviousIsOpen] = useState(isOpen);
  if (isOpen !== previousIsOpen) {
    setPreviousIsOpen(isOpen);
    if (!isOpen) setFile(null);
  }

  return (
    <Modal
      icon={<Upload size={24} />}
      isOpen={isOpen}
      maxWidth="lg"
      onClose={onClose}
      subtitle={t("leave_import.subtitle", "Upload file Excel để tạo đơn nghỉ phép hàng loạt")}
      testId="leave-import-dialog"
      title={t("leave_import.title", "Import đơn nghỉ phép")}
    >
      {importJob && (isProcessing || isFinished) ? (
        <LeaveImportProgressView getErrorInfo={getErrorInfo}
        importJob={importJob}
        isFinished={isFinished}
        isProcessing={isProcessing}
        onClose={onClose}
        setImportJob={setImportJob}
        t={t}  />
      ) : (
        <LeaveImportUploadView downloadingTemplate={downloadingTemplate}
        file={file}
        handleUpload={handleUpload}
        i18n={i18n}
        onClose={onClose}
        setDownloadingTemplate={setDownloadingTemplate}
        setFile={setFile}
        t={t}
        uploading={uploading}  />
      )}
    </Modal>
  );
}
