import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import type { TFunction } from "i18next";

import type { NoShowRow } from "../excel-no-show";
import { exportNoShowToExcel } from "../excel-no-show";

// ── ExcelJS mock ────────────────────────────────────────────────────────
// Cells are stored in a map keyed by "row:col" so tests can assert on the
// values the production code wrote (a fresh-object-per-call mock would lose
// them). resetMockState() clears everything between tests.
const cellStore = new Map<string, { value: unknown; font: {}; fill: {}; alignment: {} }>();

function makeCell(row: number, col: number) {
  const key = `${row}:${col}`;
  if (!cellStore.has(key)) {
    cellStore.set(key, { value: null, font: {}, fill: {}, alignment: {} });
  }
  return cellStore.get(key)!;
}

function resetMockState() {
  cellStore.clear();
  mockWorksheet.columns = [];
  mockWorksheet.autoFilter = undefined;
  mockWorksheet.views = [];
}

const mockWriteBuffer = vi.fn(() => Promise.resolve(new ArrayBuffer(0)));
const mockCommit = vi.fn();
const mockGetCell = vi.fn((row: number, col?: number) =>
  // Support both getCell("A1") (string) and getCell(row, col) (number).
  typeof row === "string"
    ? makeCell(1, 1)
    : makeCell(row, col ?? 1),
);
const mockGetRow = vi.fn((rowNumber: number) => ({
  getCell: (col: number) => mockGetCell(rowNumber, col),
  commit: mockCommit,
}));

const mockWorksheet = {
  columns: [] as Array<{ header: unknown; key: string; width?: number }>,
  getCell: mockGetCell,
  getRow: mockGetRow,
  mergeCells: vi.fn(),
  autoFilter: undefined as unknown,
  views: [] as Array<{ state: string; ySplit: number }>,
};

const mockWorkbook = {
  creator: "",
  created: null,
  addWorksheet: vi.fn(() => mockWorksheet),
  xlsx: { writeBuffer: mockWriteBuffer },
};

class MockWorkbook {
  creator = "";
  created = null;
  addWorksheet = mockWorkbook.addWorksheet;
  xlsx = mockWorkbook.xlsx;
}

vi.mock("exceljs", () => ({
  default: { Workbook: MockWorkbook },
  Workbook: MockWorkbook,
}));

// ── file-saver mock ──────────────────────────────────────────────────────
const mockSaveAs = vi.fn();
vi.mock("file-saver", () => ({ saveAs: mockSaveAs }));

// ── i18n helper ──────────────────────────────────────────────────────────
// Cast through `unknown` so the unbranded arrow function satisfies
// TFunction's `$TFunctionBrand` (the brand is a compile-time-only marker).
const t = ((key: string) => key) as unknown as TFunction;

const ROWS: NoShowRow[] = [
  {
    employee_id_number: "EMP001",
    employee_name: "Alice",
    department_name: "Engineering",
    shift_name: "Morning Shift",
    scheduled_start: "08:00:00",
    scheduled_end: "17:00:00",
  },
  {
    employee_id_number: "EMP002",
    employee_name: "Bob",
    department_name: "HR",
    shift_name: null,
    scheduled_start: null,
    scheduled_end: null,
  },
];

describe("exportNoShowToExcel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    resetMockState();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("creates a worksheet named after the translated title", async () => {
    await exportNoShowToExcel(ROWS, { t, date: "2026-07-10" });

    expect(mockWorkbook.addWorksheet).toHaveBeenCalledWith(
      "unrecorded_shifts.title",
    );
  });

  it("sets 6 columns matching the table order", async () => {
    await exportNoShowToExcel(ROWS, { t, date: "2026-07-10" });

    expect(mockWorksheet.columns).toHaveLength(6);
    expect(mockWorksheet.columns[0].header).toBe(
      "unrecorded_shifts.columns.employee_code",
    );
    expect(mockWorksheet.columns[1].header).toBe(
      "unrecorded_shifts.columns.employee_name",
    );
    expect(mockWorksheet.columns[2].header).toBe(
      "unrecorded_shifts.columns.department",
    );
    expect(mockWorksheet.columns[3].header).toBe(
      "unrecorded_shifts.columns.shift_type",
    );
    expect(mockWorksheet.columns[4].header).toBe(
      "unrecorded_shifts.columns.shift_in",
    );
    expect(mockWorksheet.columns[5].header).toBe(
      "unrecorded_shifts.columns.shift_out",
    );
  });

  it("writes a merged title row and a merged date row", async () => {
    await exportNoShowToExcel(ROWS, { t, date: "2026-07-10" });

    expect(mockGetCell).toHaveBeenCalledWith("A1");
    expect(mockGetCell).toHaveBeenCalledWith("A2");
    // Both rows span all 6 columns.
    expect(mockWorksheet.mergeCells).toHaveBeenCalledWith(1, 1, 1, 6);
    expect(mockWorksheet.mergeCells).toHaveBeenCalledWith(2, 1, 2, 6);
  });

  it("writes the header row at row 4 and data starting at row 5", async () => {
    await exportNoShowToExcel(ROWS, { t, date: "2026-07-10" });

    expect(mockGetRow).toHaveBeenCalledWith(4); // header
    expect(mockGetRow).toHaveBeenCalledWith(5); // first data row
    expect(mockGetRow).toHaveBeenCalledWith(6); // second data row
  });

  it("renders null shift times as an em-dash", async () => {
    await exportNoShowToExcel(ROWS, { t, date: "2026-07-10" });

    // Bob is the second row → data row 6. shift_name is null → "—", and his
    // start/end are also null → "—". With the new shift_type column inserted
    // before start/end, the mapped columns are: 4=shift_name, 5=start, 6=end.
    expect(makeCell(6, 4).value).toBe("—"); // shift_name
    expect(makeCell(6, 5).value).toBe("—"); // scheduled_start
    expect(makeCell(6, 6).value).toBe("—"); // scheduled_end
    // Alice (row 5) keeps her real values.
    expect(makeCell(5, 1).value).toBe("EMP001");
    expect(makeCell(5, 4).value).toBe("Morning Shift"); // shift_name
    expect(makeCell(5, 5).value).toBe("08:00:00");
  });

  it("sets auto-filter and freezes panes when rows exist", async () => {
    await exportNoShowToExcel(ROWS, { t, date: "2026-07-10" });

    expect(mockWorksheet.autoFilter).toEqual({
      from: { row: 4, column: 1 },
      to: { row: 5 + ROWS.length - 1, column: 6 },
    });
    expect(mockWorksheet.views).toEqual([
      { state: "frozen", ySplit: 4 },
    ]);
  });

  it("does not set auto-filter when there are no rows", async () => {
    await exportNoShowToExcel([], { t, date: "2026-07-10" });

    expect(mockWorksheet.autoFilter).toBeUndefined();
    // Header row is still written at row 4, but no data rows follow.
    expect(mockGetRow).toHaveBeenCalledWith(4);
    expect(mockGetRow).not.toHaveBeenCalledWith(5);
  });

  it("writes an .xlsx blob with a dated filename", async () => {
    await exportNoShowToExcel(ROWS, { t, date: "2026-07-10" });

    expect(mockWriteBuffer).toHaveBeenCalled();
    expect(mockSaveAs).toHaveBeenCalled();

    const [blob, filename] = mockSaveAs.mock.calls[0];
    expect(blob).toBeInstanceOf(Blob);
    expect(blob.type).toBe(
      "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    );
    expect(filename).toBe("vang-dau-ca-2026-07-10.xlsx");
  });
});
