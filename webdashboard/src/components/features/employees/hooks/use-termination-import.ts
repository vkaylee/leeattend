import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { toast } from "sonner";

import type { ConfirmTerminationRow, TerminationPreviewResponse } from "@/lib/api/employees";

import {
  uploadTerminationImportPreview,
  confirmTerminationImport,
  downloadTerminationTemplate,
} from "@/lib/api/employees";
import { fetchJobStatus } from "@/lib/api/leaves";
import { logError } from "@/lib/logger";

export function useTerminationImport(wsId: string, onClose: () => void) {
  const { t, i18n } = useTranslation("employees");
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

  const handleDownloadTemplate = async () => {
    try {
      await downloadTerminationTemplate(wsId, i18n.language);
    } catch (err) {
      logError(err, { component: "EmployeeTerminationImport" });
      toast.error(t("employees.bulk_termination.download_error", { defaultValue: "Failed to download template" }));
    }
  };

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
    const POLL_INTERVAL = 2000;
    const MAX_POLLS = 60;
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
          setJobError(statusRes.error_message || "Đã xảy ra lỗi khi xử lý nghỉ việc.");
          setConfirming(false);
          setJobId(null);
          return;
        }
      } catch (err) {
        logError(err, { component: "EmployeeTerminationImport" });
      }
      await new Promise((resolve) => setTimeout(resolve, POLL_INTERVAL));
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

  const exportErrorsToExcel = async () => {
    if (!previewData || previewData.invalid_rows.length === 0) return;
    try {
      const ExcelJS = await import("exceljs").then((m) =>
        "default" in m ? m.default : m
      );
      const { saveAs } = await import("file-saver");
      const workbook = new ExcelJS.Workbook();
      workbook.creator = "LeeAttend";
      workbook.created = new Date();
      const worksheet = workbook.addWorksheet("Lỗi Import");
      worksheet.columns = [
        { header: "Dòng", key: "row_number", width: 10 },
        { header: "Mã nhân viên", key: "employee_code", width: 20 },
        { header: "Ngày nghỉ việc", key: "termination_date", width: 20 },
        { header: "Lý do lỗi", key: "reason", width: 50 },
      ];
      const headerRow = worksheet.getRow(1);
      headerRow.eachCell((cell: unknown) => {
        const c = cell as { font: Record<string, unknown>; fill: Record<string, unknown>; alignment: Record<string, unknown> };
        c.font = { bold: true, color: { argb: "FFFFFFFF" } };
        c.fill = { type: "pattern", pattern: "solid", fgColor: { argb: "FFC00000" } };
        c.alignment = { horizontal: "center", vertical: "middle" };
      });
      previewData.invalid_rows.forEach((r) => {
        worksheet.addRow({
          row_number: r.row_number,
          employee_code: r.employee_code ?? "",
          termination_date: (r as unknown as Record<string, unknown>).raw_termination_date ?? "",
          reason: r.reason,
        });
      });
      worksheet.eachRow((row: unknown, rowNum: number) => {
        if (rowNum > 1) {
          const cell1 = (row as { getCell: (n: number) => Record<string, unknown> }).getCell(1);
          cell1.alignment = { horizontal: "center" };
          cell1.font = { bold: true, color: { argb: "FFC00000" } };
        }
      });
      const buffer = await workbook.xlsx.writeBuffer();
      const blob = new Blob([buffer], { type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" });
      saveAs(blob, `loi_import_nghi_viec_${Date.now()}.xlsx`);
    } catch (err) {
      logError(err, { component: "EmployeeTerminationImport" });
      toast.error(t("employees.bulk_termination.export_error", { defaultValue: "Failed to export errors." }));
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
    handleDownloadTemplate,
    handleUpload,
    handleConfirm,
    exportErrorsToExcel,
    handleClose,
    setJobError: setJobError,
  };
}
