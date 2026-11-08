import { Upload } from "lucide-react";
import { useTranslation } from "react-i18next";

import Modal from "@/components/ui/modal";

import { useTerminationImport } from "./hooks/use-termination-import";
import ImportErrorState from "./import-error-state";
import ImportPreviewStep from "./import-preview-step";
import ImportSuccessState from "./import-success-state";
import ImportUploadStep from "./import-upload-step";

interface EmployeeTerminationImportDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function EmployeeTerminationImportDialog({
  isOpen,
  onClose,}: EmployeeTerminationImportDialogProps) {
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
  } = useTerminationImport(onClose);

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
        <ImportSuccessState onClose={handleClose} successCount={successCount} />
      ) : jobError || confirming ? (
        <ImportErrorState jobError={jobError} jobId={jobId} onBack={() => { setJobError(null); }} />
      ) : step === 1 ? (
        <ImportUploadStep
          file={file}
          loading={loading}
          onClose={handleClose}
          onDownloadTemplate={handleDownloadTemplate}
          onFileChange={setFile}
          onUpload={handleUpload}
        />
      ) : (
        <ImportPreviewStep
          activeTab={activeTab}
          confirming={confirming}
          onClose={handleClose}
          onConfirm={handleConfirm}
          onExportErrors={exportErrorsToExcel}
          onTabChange={setActiveTab}
          previewData={previewData}
        />
      )}
    </Modal>
  );
}
