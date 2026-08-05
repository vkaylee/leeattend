/**
 * Client-side Excel export for the "No-Show Shifts" (Vắng đầu ca) tab.
 *
 * Uses the same styling conventions as `excel-builder.ts` (title row, date
 * row, styled header, auto-filter, frozen panes) but is self-contained so the
 * tab does not pull in the heavier report-stream machinery.
 */

import type { Worksheet, Cell } from "exceljs";
import type { TFunction } from "i18next";

// Dynamic imports keep exceljs + file-saver out of the initial bundle.
async function loadExcelJS() {
  const module = await import("exceljs");
  return "default" in module ? module.default : module;
}

async function loadFileSaver() {
  const { saveAs } = await import("file-saver");
  return saveAs;
}

/** A single enriched row as rendered in the Unrecorded Shifts table. */
export interface NoShowRow {
  department_name: string;
  employee_id_number: string;
  employee_name: string;
  scheduled_end: string | null;
  scheduled_start: string | null;
  actual_punch_in: string | null;
  actual_punch_out: string | null;
  shift_name: string | null;
  zone_label: string | null;
}

interface ExportOptions {
  /** The target calendar date label (yyyy-MM-dd), also embedded in the filename. */
  dateLabel: string;
  /** i18n translator — used for the sheet title, date label and column headers. */
  t: TFunction;
}

// Column order matches the table: code, name, department, shift type, shift in, shift out.
const COLUMNS: { key: keyof NoShowRow; i18nKey: string; width: number }[] = [
  { key: "employee_id_number", i18nKey: "employee_code", width: 18 },
  { key: "employee_name", i18nKey: "employee_name", width: 30 },
  { key: "department_name", i18nKey: "department", width: 24 },
  { key: "shift_name", i18nKey: "shift_type", width: 20 },
  { key: "scheduled_start", i18nKey: "shift_in", width: 16 },
  { key: "scheduled_end", i18nKey: "shift_out", width: 16 },
  { key: "actual_punch_in", i18nKey: "punch_in", width: 20 },
  { key: "actual_punch_out", i18nKey: "punch_out", width: 20 },
  { key: "zone_label", i18nKey: "zone", width: 22 },
];

/**
 * Build and download an .xlsx file of the no-show rows.
 *
 * @param rows   Enriched absent rows (already filtered by the caller).
 * @param options `{ t, date }` — translator and report date.
 */
export async function exportNoShowToExcel(
  rows: NoShowRow[],
  options: ExportOptions,
): Promise<void> {
  const ExcelJS = await loadExcelJS();
  const saveAs = await loadFileSaver();
  const { t, dateLabel } = options;

  const workbook = new ExcelJS.Workbook();
  workbook.creator = "LeeAttend";
  workbook.created = new Date();

  const worksheet = workbook.addWorksheet(t("unrecorded_shifts.title"));

  worksheet.columns = COLUMNS.map((col) => ({
    header: t(`unrecorded_shifts.columns.${col.i18nKey}`),
    key: col.key,
    width: col.width,
  }));

  // ── Title row (row 1) ─────────────────────────────────────────────────
  const titleCell = worksheet.getCell("A1");
  titleCell.value = t("unrecorded_shifts.title");
  titleCell.font = { bold: true, size: 14, color: { argb: "FF1F4E79" } };
  worksheet.mergeCells(1, 1, 1, COLUMNS.length);

  // ── Date row (row 2) ───────────────────────────────────────────────────
  const dateCell = worksheet.getCell("A2");
  dateCell.value = `${t("unrecorded_shifts.export_date")}: ${dateLabel}`;
  dateCell.font = { italic: true, color: { argb: "FF5B5B5B" } };
  worksheet.mergeCells(2, 1, 2, COLUMNS.length);

  // ── Header row (row 4, leaving row 3 blank for breathing room) ────────
  const headerRowNumber = 4;
  writeHeader(worksheet, headerRowNumber);

  // ── Data rows (starting at row 5) ─────────────────────────────────────
  const dataStartRow = headerRowNumber + 1;
  rows.forEach((row, index) => {
    const excelRow = worksheet.getRow(dataStartRow + index);
    COLUMNS.forEach((col, colIndex) => {
      const cell = excelRow.getCell(colIndex + 1);
      const value = row[col.key];
      cell.value = value ?? "—";
      cell.alignment = { horizontal: "center", vertical: "middle" };
    });
    excelRow.commit();
  });

  // ── Auto-filter over header + data ────────────────────────────────────
  if (rows.length > 0) {
    worksheet.autoFilter = {
      from: { row: headerRowNumber, column: 1 },
      to: { row: dataStartRow + rows.length - 1, column: COLUMNS.length },
    };
  }

  // ── Freeze the title + header rows ────────────────────────────────────
  worksheet.views = [{ state: "frozen", ySplit: headerRowNumber }];

  // ── Write to blob and trigger download ────────────────────────────────
  const buffer = await workbook.xlsx.writeBuffer();
  const blob = new Blob([buffer], {
    type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  });
  const fileSlug = dateLabel.replace(/[:.]/g, "-");
  saveAs(blob, `vang-dau-ca-${fileSlug}.xlsx`);
}

function writeHeader(worksheet: Worksheet, headerRowNumber: number) {
  const headerRow = worksheet.getRow(headerRowNumber);
  // worksheet.columns already holds the translated headers (set above), so
  // read them back to keep the styling pass in sync without re-translating.
  // During setup each column header is set to a single translated string;
  // the ExcelJS type allows string[] so coerce defensively.
  const headers = worksheet.columns.map((c) => c.header);
  headers.forEach((header, index) => {
    const cell: Cell = headerRow.getCell(index + 1);
    cell.value = typeof header === "string" ? header : String(header);
    cell.font = { bold: true, color: { argb: "FFFFFFFF" } };
    cell.fill = {
      type: "pattern",
      pattern: "solid",
      fgColor: { argb: "FF4472C4" },
    };
    cell.alignment = { horizontal: "center", vertical: "middle" };
  });
  headerRow.commit();
}
