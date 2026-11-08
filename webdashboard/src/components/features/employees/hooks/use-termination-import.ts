import { useTranslation } from "react-i18next";

import { toast } from "@/components/ui/toast-imperative";
import { downloadTerminationTemplate } from "@/lib/api/employees";
import { logError } from "@/lib/logger";
import { buildTerminationErrorWorkbook } from "@/lib/termination-error-export";

import { useImportStateMachine } from "./use-import-state-machine";

export function useTerminationImport(onClose: () => void) {
  const { t, i18n } = useTranslation("employees");
  const state = useImportStateMachine(onClose);

  const handleDownloadTemplate = async () => {
    try {
      await downloadTerminationTemplate(i18n.language);
    } catch (err) {
      logError(err, { component: "EmployeeTerminationImport" });
      toast.error(t("employees.bulk_termination.download_error", { defaultValue: "Failed to download template" }));
    }
  };

  const exportErrorsToExcel = async () => {
    if (!state.previewData || state.previewData.invalid_rows.length === 0) return;
    try {
      const buffer = await buildTerminationErrorWorkbook(state.previewData.invalid_rows);
      const { saveAs } = await import("file-saver");
      const blob = new Blob([buffer], {
        type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      });
      saveAs(blob, `loi_import_nghi_viec_${Date.now()}.xlsx`);
    } catch (err) {
      logError(err, { component: "EmployeeTerminationImport" });
      toast.error(t("employees.bulk_termination.export_error", { defaultValue: "Failed to export errors." }));
    }
  };

  return {
    file: state.file, setFile: state.setFile,
    loading: state.loading, confirming: state.confirming,
    step: state.step, previewData: state.previewData,
    activeTab: state.activeTab, setActiveTab: state.setActiveTab,
    importSuccess: state.importSuccess, successCount: state.successCount,
    jobId: state.jobId, jobError: state.jobError,
    handleDownloadTemplate,
    handleUpload: state.handleUpload,
    handleConfirm: state.handleConfirm,
    exportErrorsToExcel,
    handleClose: state.handleClose,
    setJobError: state.setJobError,
  };
}
