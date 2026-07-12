import { Upload } from "lucide-react";
import { useTranslation } from "react-i18next";

import { useTerminationImport } from "./hooks/use-termination-import";
import ImportUploadStep from "./import-upload-step";
import ImportPreviewStep from "./import-preview-step";
import ImportSuccessState from "./import-success-state";
import ImportErrorState from "./import-error-state";

import Modal from "@/components/ui/modal";

interface EmployeeTerminationImportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  wsId: string;
}

export default function EmployeeTerminationImportDialog({
  isOpen,
  onClose,
  wsId,
}: EmployeeTerminationImportDialogProps) {
  const { t } = useTranslation("employees");
  const {
    file, loading, confirming, step, previewData,
    activeTab, setActiveTab, importSuccess, successCount,
    jobId, jobError,
    setFile,
    setJobError,
    handleDownloadTemplate,
    handleUpload,
    handleConfirm,
    exportErrorsToExcel,
    handleClose,
  } = useTerminationImport(wsId, onClose);

  return (
    <Modal
      icon={<Upload size={24} />}
      isOpen={isOpen}
      maxWidth={step === 1 ? "lg" : "xl"}
      onClose={handleClose}
      subtitle={t("employees.bulk_termination.dialog_subtitle")}
      title={t("employees.bulk_termination.dialog_title")}
    >
      {importSuccess ? (
        <ImportSuccessState successCount={successCount} onClose={handleClose} />
      ) : jobError || confirming ? (
        <ImportErrorState jobError={jobError} jobId={jobId} onBack={() => setJobError?.(null as unknown as string)} />
      ) : step === 1 ? (
        <ImportUploadStep
          file={file}
          loading={loading}
          onFileChange={setFile}
          onDownloadTemplate={handleDownloadTemplate}
          onUpload={handleUpload}
          onClose={handleClose}
        />
      ) : (
        <ImportPreviewStep
          previewData={previewData}
          activeTab={activeTab}
          confirming={confirming}
          onTabChange={setActiveTab}
          onExportErrors={exportErrorsToExcel}
          onConfirm={handleConfirm}
          onClose={handleClose}
        />
      )}
    </Modal>
  );
}
