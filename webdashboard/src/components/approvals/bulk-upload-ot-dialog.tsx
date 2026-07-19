import * as Dialog from "@radix-ui/react-dialog";
import { useTranslation } from "react-i18next";

import { useConfirm } from "@/hooks/use-confirm";
import { downloadCsvErrorReport } from "@/lib/csv-utils";

import { BulkUploadDialogContent } from "./bulk-upload/bulk-upload-dialog-content";
import { useBulkUploadForm } from "./bulk-upload/use-bulk-upload-form";

interface Props {
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
  open: boolean;
}

export function BulkUploadOtDialog({ open, onOpenChange, onSuccess }: Props) {
  const { confirm, ConfirmationDialog } = useConfirm();
  const { t: translate } = useTranslation();
  const form = useBulkUploadForm(onSuccess);
  const {
    t,
    uploadResult,
    resetForm,
    isDragOver,
    setIsDragOver,
    handleFileChange,
    selectedFile,
    setSelectedFile,
    isUploading,
    handleUpload,
    error,
  } = form;

  const handleClose = async () => {
    if (uploadResult && uploadResult.failed_count > 0) {
      const ok = await confirm({
        title: translate("common.warning", { defaultValue: "Warning" }),
        message: translate("approvals.bulk_upload.confirm_close_with_errors", {
          defaultValue:
            "Some rows failed. Are you sure you want to close?",
        }),
        variant: "danger",
      });
      if (!ok) return;
    }
    resetForm();
    onOpenChange(false);
  };

  return (
    <Dialog.Root
      onOpenChange={(o) => {
        if (!o) {
          void handleClose();
        }
      }}
      open={open}
    >
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40 z-50 backdrop-blur-sm" />
        <BulkUploadDialogContent
          error={error}
          handleClose={() => {
            void handleClose();
          }}
          handleFileChange={handleFileChange}
          handleUpload={() => {
            void handleUpload();
          }}
          isDragOver={isDragOver}
          isUploading={isUploading}
          onDownloadErrorReport={() => {
            if (uploadResult) downloadCsvErrorReport(uploadResult.errors);
          }}
          selectedFile={selectedFile}
          setIsDragOver={setIsDragOver}
          setSelectedFile={setSelectedFile}
          t={t}
          uploadResult={uploadResult}
        />
      </Dialog.Portal>
      <ConfirmationDialog />
    </Dialog.Root>
  );
}
