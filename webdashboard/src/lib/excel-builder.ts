/**
 * Client-side Excel generation using ExcelJS
 * Matches backend excel_writer.rs column structure for consistency
 */

import type { SummaryRow, DetailRow, AttendanceRow } from "@/types/report-stream";

// Dynamic imports to reduce initial bundle size
async function loadExcelJS() {
  const module = await import("exceljs");
  // Handle both ESM default and CommonJS exports
  return "default" in module ? module.default : module;
}

async function loadFileSaver() {
  const { saveAs } = await import("file-saver");
  return saveAs;
}

export interface ExcelColumn {
  header: string;
  key: string;
  width: number;
}

// Column definitions matching backend excel_writer.rs
const SUMMARY_COLUMNS: ExcelColumn[] = [
  { header: "Employee Name", key: "employee_name", width: 35 },
  { header: "Std Units", key: "total_std_units", width: 15 },
  { header: "OT Units", key: "total_ot_units", width: 15 },
  { header: "Total Leave Days", key: "total_leave_days", width: 18 },
];

const DETAIL_COLUMNS: ExcelColumn[] = [
  { header: "Date", key: "date", width: 12 },
  { header: "Employee Name", key: "employee_name", width: 30 },
  { header: "Scheduled Start", key: "scheduled_start", width: 15 },
  { header: "Scheduled End", key: "scheduled_end", width: 15 },
  { header: "Actual In", key: "actual_in", width: 12 },
  { header: "Actual Out", key: "actual_out", width: 12 },
  { header: "Std Units", key: "std_work_units", width: 12 },
  { header: "OT Units", key: "ot_work_units", width: 12 },
  { header: "OT Status", key: "ot_claim_status", width: 12 },
  { header: "Status", key: "status", width: 12 },
];

export type ReportType = "summary" | "detail";

export interface GenerateOptions {
  dateRange?: { start: string; end: string };
  filename?: string;
  onProgress?: (percent: number) => void;
  reportType: ReportType;
  workspaceName?: string;
}

/**
 * Generate Excel file from attendance data
 * Uses chunked processing with progress callbacks for large datasets
 */
export async function generateExcel(
  rows: AttendanceRow[],
  options: GenerateOptions
): Promise<void> {
  const { reportType, workspaceName, dateRange, filename, onProgress } = options;
  const ExcelJS = await loadExcelJS();
  const saveAs = await loadFileSaver();

  onProgress?.(0);

  const workbook = new ExcelJS.Workbook();
  workbook.creator = "LeeAttend";
  workbook.created = new Date();

  const sheetName = reportType === "summary" ? "Summary" : "Detail";
  const worksheet = workbook.addWorksheet(sheetName);

  const columns = reportType === "summary" ? SUMMARY_COLUMNS : DETAIL_COLUMNS;
  worksheet.columns = columns;

  // Title row (row 1)
  const titleText = reportType === "summary"
    ? `Attendance Summary Report${workspaceName ? ` - ${workspaceName}` : ""}`
    : `Attendance Detail Report${workspaceName ? ` - ${workspaceName}` : ""}`;

  const titleCell = worksheet.getCell("A1");
  titleCell.value = titleText;
  titleCell.font = { bold: true, size: 14, color: { argb: "FF1F4E79" } };
  worksheet.mergeCells(1, 1, 1, columns.length);

  // Date range row (row 2)
  if (dateRange) {
    const rangeCell = worksheet.getCell("A2");
    rangeCell.value = `Period: ${dateRange.start} to ${dateRange.end}`;
    rangeCell.font = { italic: true, color: { argb: "FF5B5B5B" } };
    worksheet.mergeCells(2, 1, 2, columns.length);
  }

  onProgress?.(10);

  // Header row (row 4)
  const headerRowNumber = 4;
  const headerRow = worksheet.getRow(headerRowNumber);
  columns.forEach((col, index) => {
    const cell = headerRow.getCell(index + 1);
    cell.value = col.header;
    cell.font = { bold: true, color: { argb: "FFFFFFFF" } };
    cell.fill = {
      type: "pattern",
      pattern: "solid",
      fgColor: { argb: "FF4472C4" },
    };
    cell.alignment = { horizontal: "center", vertical: "middle" };
  });
  headerRow.commit();

  onProgress?.(20);

  // Data rows - process in chunks for progress and UI responsiveness
  const chunkSize = 1000;
  const dataStartRow = headerRowNumber + 1;

  for (let index = 0; index < rows.length; index += chunkSize) {
    const chunk = rows.slice(index, index + chunkSize);

    chunk.forEach((row, chunkIndex) => {
      const rowNumber = dataStartRow + index + chunkIndex;
      const excelRow = worksheet.getRow(rowNumber);

      columns.forEach((col, colIndex) => {
        const value = row[col.key as keyof AttendanceRow] ?? "";
        const cell = excelRow.getCell(colIndex + 1);
        cell.value = value;

        // Status column styling (detail report)
        if (col.key === "status" && reportType === "detail") {
          const statusColors: Record<string, string> = {
            present: "FF2E7D32",
            late: "FFF57C00",
            absent: "FFD32F2F",
            off: "FF757575",
          };
          const color = statusColors[String(value).toLowerCase()];
          if (color) {
            cell.font = { color: { argb: color } };
          }
          cell.alignment = { horizontal: "center" };
        }
      });

      excelRow.commit();
    });

    // Update progress (20% to 80% for data rows)
    const progress = 20 + Math.floor((Math.min(index + chunkSize, rows.length) / rows.length) * 60);
    onProgress?.(progress);

    // Yield to UI thread
    await new Promise((resolve) => setTimeout(resolve, 0));
  }

  onProgress?.(85);

  // Auto-filter on data range
  if (rows.length > 0) {
    worksheet.autoFilter = {
      from: { row: headerRowNumber, column: 1 },
      to: { row: dataStartRow + rows.length - 1, column: columns.length },
    };
  }

  // Freeze header rows
  worksheet.views = [{ state: "frozen", ySplit: headerRowNumber }];

  onProgress?.(90);

  // Generate and download
  const buffer = await workbook.xlsx.writeBuffer();
  const blob = new Blob([buffer], {
    type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  });

  const timestamp = new Date().toISOString().slice(0, 10);
  const defaultFilename = `attendance-${reportType}-${timestamp}.xlsx`;
  saveAs(blob, filename || defaultFilename);

  onProgress?.(100);
}

/**
 * Type guard to check if row is SummaryRow
 */
export function isSummaryRow(row: AttendanceRow): row is SummaryRow {
  return "total_std_units" in row;
}

/**
 * Type guard to check if row is DetailRow
 */
export function isDetailRow(row: AttendanceRow): row is DetailRow {
  return "date" in row && "status" in row;
}
