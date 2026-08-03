import { beforeEach, describe, expect, it, vi } from "vitest";

import type { InvalidTerminationRow } from "@/lib/api/api-generated-reexports";

import { buildTerminationErrorWorkbook } from "../termination-error-export";

const mocks = vi.hoisted(() => {
  const writeBuffer = vi.fn();
  const worksheet = {
    columns: [] as unknown[],
    getRow: vi.fn(),
    addRow: vi.fn(),
    eachRow: vi.fn(),
  };
  const headerRow = { eachCell: vi.fn() };
  const Workbook = class {
    creator = "";
    created = null;
    addWorksheet = vi.fn(() => worksheet);
    xlsx = { writeBuffer };
  };
  return { writeBuffer, worksheet, headerRow, Workbook };
});

vi.mock("exceljs", () => ({
  default: { Workbook: mocks.Workbook },
  Workbook: mocks.Workbook,
}));

const INVALID_ROWS: InvalidTerminationRow[] = [
  { row_number: 2, employee_code: "EMP002", raw_termination_date: "bad-date", reason: "Invalid date format" },
  { row_number: 3, employee_code: null, raw_termination_date: null, reason: "Missing code" },
];

describe("buildTerminationErrorWorkbook", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.writeBuffer.mockResolvedValue(new Uint8Array([1, 2, 3]));
    mocks.worksheet.columns = [];
    mocks.worksheet.getRow.mockReturnValue(mocks.headerRow);
    mocks.headerRow.eachCell.mockImplementation((callback: (cell: unknown) => void) => {
      callback({ font: {}, fill: {}, alignment: {} });
    });
    mocks.worksheet.eachRow.mockImplementation((callback: (row: unknown, rowNum: number) => void) => {
      callback({ getCell: () => ({ alignment: {}, font: {} }) }, 2);
    });
  });

  it("creates a workbook with the error sheet and returns the xlsx buffer", async () => {
    const buffer = await buildTerminationErrorWorkbook(INVALID_ROWS);

    expect(mocks.worksheet.columns).toHaveLength(4);
    expect(mocks.worksheet.getRow).toHaveBeenCalledWith(1);
    expect(mocks.headerRow.eachCell).toHaveBeenCalled();
    expect(mocks.worksheet.eachRow).toHaveBeenCalled();
    expect(mocks.writeBuffer).toHaveBeenCalledTimes(1);
    expect(buffer).toEqual(new Uint8Array([1, 2, 3]));
  });

  it("writes one row per invalid row with the mapped fields", async () => {
    await buildTerminationErrorWorkbook(INVALID_ROWS);

    expect(mocks.worksheet.addRow).toHaveBeenCalledTimes(2);
    expect(mocks.worksheet.addRow).toHaveBeenNthCalledWith(1, {
      row_number: 2,
      employee_code: "EMP002",
      termination_date: "bad-date",
      reason: "Invalid date format",
    });
    expect(mocks.worksheet.addRow).toHaveBeenNthCalledWith(2, {
      row_number: 3,
      employee_code: "",
      termination_date: "",
      reason: "Missing code",
    });
  });

  it("writes no data rows when there are no invalid rows", async () => {
    const buffer = await buildTerminationErrorWorkbook([]);

    expect(mocks.worksheet.addRow).not.toHaveBeenCalled();
    expect(buffer).toEqual(new Uint8Array([1, 2, 3]));
  });
});
