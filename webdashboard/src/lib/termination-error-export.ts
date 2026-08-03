import type { InvalidTerminationRow } from "@/lib/api/api-generated-reexports";

/**
 * Builds an xlsx workbook (as a byte buffer) listing the invalid rows of a
 * termination-import preview. Pure and side-effect-free so it is unit-testable;
 * the caller owns file download via file-saver.
 */
export async function buildTerminationErrorWorkbook(
  invalidRows: InvalidTerminationRow[],
) {
  const ExcelJS = await import("exceljs").then((module) =>
    "default" in module ? module.default : module,
  );

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
    const styled = cell as { font: Record<string, unknown>; fill: Record<string, unknown>; alignment: Record<string, unknown> };
    styled.font = { bold: true, color: { argb: "FFFFFFFF" } };
    styled.fill = { type: "pattern", pattern: "solid", fgColor: { argb: "FFC00000" } };
    styled.alignment = { horizontal: "center", vertical: "middle" };
  });

  invalidRows.forEach((row) => {
    worksheet.addRow({
      row_number: row.row_number,
      employee_code: row.employee_code ?? "",
      termination_date: row.raw_termination_date ?? "",
      reason: row.reason,
    });
  });

  worksheet.eachRow((row: unknown, rowNum: number) => {
    if (rowNum > 1) {
      const firstCell = (row as { getCell: (n: number) => Record<string, unknown> }).getCell(1);
      firstCell.alignment = { horizontal: "center" };
      firstCell.font = { bold: true, color: { argb: "FFC00000" } };
    }
  });

  return workbook.xlsx.writeBuffer();
}
