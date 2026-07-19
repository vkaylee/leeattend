import { useState } from "react";
import { useTranslation } from "react-i18next";

import { useBulkUploadPaperOt } from "@/hooks/use-attendance";

import type { UploadResult } from "./bulk-upload-types";

export function useBulkUploadForm(onSuccess?: () => void) {
  const { t } = useTranslation();
  const { bulkUploadPaperOt, isUploading } = useBulkUploadPaperOt();

  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [uploadResult, setUploadResult] = useState<UploadResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isDragOver, setIsDragOver] = useState(false);

  const resetForm = () => {
    setSelectedFile(null);
    setUploadResult(null);
    setError(null);
    setIsDragOver(false);
  };

  const handleFileChange = (file: File | null) => {
    if (!file) return;
    if (!file.name.endsWith(".csv")) {
      setError(t("approvals.bulk_upload.error_csv_only"));
      return;
    }
    setSelectedFile(file);
    setError(null);
    setUploadResult(null);
  };

  const handleUpload = async () => {
    if (!selectedFile) return;
    setError(null);
    try {
      const result = await bulkUploadPaperOt(selectedFile);
      setUploadResult(result);
      if (result.success_count > 0) onSuccess?.();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Failed to upload file");
    }
  };

  return {
    error,
    handleFileChange,
    handleUpload,
    isDragOver,
    isUploading,
    resetForm,
    selectedFile,
    setIsDragOver,
    setSelectedFile,
    t,
    uploadResult,
  };
}
