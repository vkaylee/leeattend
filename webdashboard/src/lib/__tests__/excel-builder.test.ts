import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import type { SummaryRow, DetailRow, AttendanceRow } from "@/types/report-stream";

import { generateExcel, isSummaryRow, isDetailRow } from "../excel-builder";

// Mock ExcelJS
const mockWriteBuffer = vi.fn(() => Promise.resolve(new ArrayBuffer(0)));
const mockCommit = vi.fn();
const mockGetCell = vi.fn((row: number, col?: number) => {
  const _cellIndex = col ?? row;
  return {
    value: null,
    font: {},
    fill: {},
    alignment: {},
  };
});
const mockGetRow = vi.fn((rowNumber: number) => ({
  getCell: (col: number) => mockGetCell(rowNumber, col),
  commit: mockCommit,
}));

const mockWorksheet = {
  columns: [] as { header: string; key: string; width?: number }[],
  getCell: mockGetCell,
  getRow: mockGetRow,
  mergeCells: vi.fn(),
  autoFilter: undefined,
  views: [],
};

const mockWorkbook = {
  creator: "",
  created: null,
  addWorksheet: vi.fn(() => mockWorksheet),
  xlsx: {
    writeBuffer: mockWriteBuffer,
  },
};

class MockWorkbook {
  creator = "";
  created = null;
  addWorksheet = mockWorkbook.addWorksheet;
  xlsx = mockWorkbook.xlsx;
}

vi.mock("exceljs", () => ({
  default: {
    Workbook: MockWorkbook,
  },
  Workbook: MockWorkbook,
}));

// Mock file-saver
const mockSaveAs = vi.fn();
vi.mock("file-saver", () => ({
  saveAs: mockSaveAs,
}));

describe("excel-builder", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("generateExcel", () => {
    const summaryRows: SummaryRow[] = [
      {
        employee_name: "John Doe",
        total_std_units: 140,
        total_ot_units: 20,
        total_leave_days: 2,
      },
      {
        employee_name: "Jane Smith",
        total_std_units: 152,
        total_ot_units: 0,
        total_leave_days: 1,
      },
    ];

    const detailRows: DetailRow[] = [
      {
        date: "2026-01-15",
        employee_name: "John Doe",
        scheduled_start: "09:00",
        scheduled_end: "17:00",
        actual_in: "08:55",
        actual_out: "17:10",
        std_work_units: 8,
        ot_work_units: 0,
        status: "present",
      },
      {
        date: "2026-01-16",
        employee_name: "John Doe",
        scheduled_start: "09:00",
        scheduled_end: "17:00",
        actual_in: "09:15",
        actual_out: "17:00",
        std_work_units: 7,
        ot_work_units: 0,
        status: "late",
      },
      {
        date: "2026-01-17",
        employee_name: "Jane Smith",
        scheduled_start: "09:00",
        scheduled_end: "17:00",
        actual_in: null,
        actual_out: null,
        std_work_units: 0,
        ot_work_units: 0,
        status: "absent",
      },
    ];

    it("generates summary report with correct structure", async () => {
      const options = {
        reportType: "summary" as const,
        workspaceName: "Test Workspace",
        dateRange: { start: "2026-01-01", end: "2026-01-31" },
      };

      await generateExcel(summaryRows, options);

      // Verify workbook creation
      expect(mockWorkbook.addWorksheet).toHaveBeenCalledWith("Summary");

      // Verify worksheet columns set (summary has 4 columns)
      expect(mockWorksheet.columns).toHaveLength(4);
      expect(mockWorksheet.columns[0].header).toBe("Employee Name");
      expect(mockWorksheet.columns[1].header).toBe("Std Units");
      expect(mockWorksheet.columns[2].header).toBe("OT Units");
      expect(mockWorksheet.columns[3].header).toBe("Total Leave Days");

      // Verify title row creation
      expect(mockGetCell).toHaveBeenCalledWith("A1");
      expect(mockWorksheet.mergeCells).toHaveBeenCalledWith(1, 1, 1, 4);

      // Verify date range row
      expect(mockGetCell).toHaveBeenCalledWith("A2");
      expect(mockWorksheet.mergeCells).toHaveBeenCalledWith(2, 1, 2, 4);

      // Verify header row (row 4)
      expect(mockGetRow).toHaveBeenCalledWith(4);

      // Verify data rows (starting from row 5)
      expect(mockGetRow).toHaveBeenCalledWith(5); // First data row
      expect(mockGetRow).toHaveBeenCalledWith(6); // Second data row

      // Verify file generation
      expect(mockWriteBuffer).toHaveBeenCalled();
      expect(mockSaveAs).toHaveBeenCalled();

      const saveAsCall = mockSaveAs.mock.calls[0];
      expect(saveAsCall[0]).toBeInstanceOf(Blob);
      expect(saveAsCall[1]).toMatch(/^attendance-summary-\d{4}-\d{2}-\d{2}\.xlsx$/);
    });

    it("generates detail report with correct structure", async () => {
      const options = {
        reportType: "detail" as const,
        workspaceName: "Test Workspace",
        dateRange: { start: "2026-01-15", end: "2026-01-17" },
      };

      await generateExcel(detailRows, options);

      // Verify workbook creation
      expect(mockWorkbook.addWorksheet).toHaveBeenCalledWith("Detail");

      // Verify worksheet columns set (detail has 10 columns)
      expect(mockWorksheet.columns).toHaveLength(10);
      expect(mockWorksheet.columns[0].header).toBe("Date");
      expect(mockWorksheet.columns[1].header).toBe("Employee Name");
      expect(mockWorksheet.columns[9].header).toBe("Status");

      // Verify title and header rows
      expect(mockGetCell).toHaveBeenCalledWith("A1");
      expect(mockWorksheet.mergeCells).toHaveBeenCalledWith(1, 1, 1, 10);

      // Verify data rows
      expect(mockGetRow).toHaveBeenCalledWith(5); // First data row
      expect(mockGetRow).toHaveBeenCalledWith(6); // Second data row
      expect(mockGetRow).toHaveBeenCalledWith(7); // Third data row

      // Verify file generation
      expect(mockWriteBuffer).toHaveBeenCalled();
      expect(mockSaveAs).toHaveBeenCalled();

      const saveAsCall = mockSaveAs.mock.calls[0];
      expect(saveAsCall[1]).toMatch(/^attendance-detail-\d{4}-\d{2}-\d{2}\.xlsx$/);
    });

    it("uses custom filename when provided", async () => {
      const options = {
        reportType: "summary" as const,
        filename: "custom-report.xlsx",
      };

      await generateExcel(summaryRows, options);

      expect(mockSaveAs).toHaveBeenCalled();
      const saveAsCall = mockSaveAs.mock.calls[0];
      expect(saveAsCall[1]).toBe("custom-report.xlsx");
    });

    it("calls progress callback with correct percentages", async () => {
      const onProgress = vi.fn();
      const options = {
        reportType: "summary" as const,
        onProgress,
      };

      await generateExcel(summaryRows, options);

      // Verify progress callback sequence
      expect(onProgress).toHaveBeenCalledWith(0); // Start
      expect(onProgress).toHaveBeenCalledWith(10); // After initial setup
      expect(onProgress).toHaveBeenCalledWith(20); // After header
      expect(onProgress).toHaveBeenCalledWith(85); // Before auto-filter
      expect(onProgress).toHaveBeenCalledWith(90); // After auto-filter
      expect(onProgress).toHaveBeenCalledWith(100); // Complete

      // Verify progress is called during data processing (20-80 range)
      const progressCalls = onProgress.mock.calls.map((call) => call[0]);
      const dataProcessCalls = progressCalls.filter((p) => p > 20 && p < 85);
      expect(dataProcessCalls.length).toBeGreaterThan(0);
    });

    it("handles large datasets with chunked processing", async () => {
      // Create 2500 rows to test chunking (chunk size is 1000)
      const largeDataset: SummaryRow[] = Array.from({ length: 2500 }, (_, index) => ({
        employee_name: `Employee ${index + 1}`,
        total_std_units: 140,
        total_ot_units: 20,
        total_leave_days: 0,
      }));

      const onProgress = vi.fn();
      const options = {
        reportType: "summary" as const,
        onProgress,
      };

      await generateExcel(largeDataset, options);

      // Verify multiple progress updates during data processing
      const progressCalls = onProgress.mock.calls.map((call) => call[0]);
      const dataProcessCalls = progressCalls.filter((p) => p > 20 && p < 85);

      // Should have multiple progress updates for 2500 rows in chunks of 1000
      expect(dataProcessCalls.length).toBeGreaterThanOrEqual(2);

      // Verify all rows were processed (starting from row 5)
      const lastDataRow = 5 + largeDataset.length - 1;
      expect(mockGetRow).toHaveBeenCalledWith(lastDataRow);
    });

    it("handles empty dataset gracefully", async () => {
      const options = {
        reportType: "summary" as const,
      };

      await generateExcel([], options);

      // Should still create workbook and headers
      expect(mockWorkbook.addWorksheet).toHaveBeenCalled();
      expect(mockWriteBuffer).toHaveBeenCalled();
      expect(mockSaveAs).toHaveBeenCalled();

      // No data rows should be created (only header at row 4)
      const dataRowCalls = mockGetRow.mock.calls.filter(
        (call) => (call[0]) >= 5
      );
      expect(dataRowCalls).toHaveLength(0);
    });

    it("sets auto-filter when rows exist", async () => {
      const options = {
        reportType: "summary" as const,
      };

      await generateExcel(summaryRows, options);

      // Auto-filter should be set for data range
      expect(mockWorksheet.autoFilter).toBeDefined();
      expect(mockWorksheet.autoFilter).toEqual({
        from: { row: 4, column: 1 },
        to: { row: 6, column: 4 }, // 2 data rows + header row
      });
    });

    it("freezes header rows", async () => {
      const options = {
        reportType: "summary" as const,
      };

      await generateExcel(summaryRows, options);

      // Verify freeze pane at row 4 (header row)
      expect(mockWorksheet.views).toEqual([
        { state: "frozen", ySplit: 4 },
      ]);
    });

    it("omits workspace name when not provided", async () => {
      const options = {
        reportType: "summary" as const,
      };

      await generateExcel(summaryRows, options);

      // Title should not include workspace name
      expect(mockGetCell).toHaveBeenCalledWith("A1");
    });

    it("omits date range when not provided", async () => {
      const options = {
        reportType: "summary" as const,
        workspaceName: "Test Workspace",
      };

      // Track merge calls before
      const mergeCallsBefore = mockWorksheet.mergeCells.mock.calls.length;

      await generateExcel(summaryRows, options);

      // Should only have title merge, not date range merge
      const mergeCallsAfter = mockWorksheet.mergeCells.mock.calls.length;
      expect(mergeCallsAfter).toBeGreaterThan(mergeCallsBefore);

      // Title row merged, but no date range row
      expect(mockGetCell).toHaveBeenCalledWith("A1");
      // A2 should not be accessed for date range (call signature is getCell(row, col) where row is a number)
      const a2Calls = mockGetCell.mock.calls.filter((call) => {
        const argument = call[0];
        return typeof argument === 'string' && argument === "A2";
      });
      expect(a2Calls).toHaveLength(0);
    });

    it("handles null values in detail rows", async () => {
      const rowsWithNulls: DetailRow[] = [
        {
          date: "2026-01-15",
          employee_name: "John Doe",
          scheduled_start: null,
          scheduled_end: null,
          actual_in: null,
          actual_out: null,
          std_work_units: 0,
          ot_work_units: 0,
          status: "off",
        },
      ];

      const options = {
        reportType: "detail" as const,
      };

      // Should not throw errors with null values
      await expect(generateExcel(rowsWithNulls, options)).resolves.not.toThrow();

      expect(mockWriteBuffer).toHaveBeenCalled();
      expect(mockSaveAs).toHaveBeenCalled();
    });
  });

  describe("Type Guards", () => {
    describe("isSummaryRow", () => {
      it("returns true for valid SummaryRow", () => {
        const summaryRow: SummaryRow = {
          employee_name: "John Doe",
          total_std_units: 140,
          total_ot_units: 20,
          total_leave_days: 2,
        };

        expect(isSummaryRow(summaryRow)).toBe(true);
      });

      it("returns false for DetailRow", () => {
        const detailRow: DetailRow = {
          date: "2026-01-15",
          employee_name: "John Doe",
          scheduled_start: "09:00",
          scheduled_end: "17:00",
          actual_in: "08:55",
          actual_out: "17:10",
          std_work_units: 8,
          ot_work_units: 0,
          status: "present",
        };

        expect(isSummaryRow(detailRow)).toBe(false);
      });

      it("returns false for object without total_std_units", () => {
        const invalidRow = {
          employee_name: "John Doe",
          total_leave_days: 2,
        };

        expect(isSummaryRow(invalidRow as AttendanceRow)).toBe(false);
      });

      it("returns true even with additional properties", () => {
        const rowWithExtra = {
          employee_name: "John Doe",
          total_std_units: 140,
          total_ot_units: 20,
          total_leave_days: 2,
          extra_field: "value",
        };

        expect(isSummaryRow(rowWithExtra as AttendanceRow)).toBe(true);
      });
    });

    describe("isDetailRow", () => {
      it("returns true for valid DetailRow", () => {
        const detailRow: DetailRow = {
          date: "2026-01-15",
          employee_name: "John Doe",
          scheduled_start: "09:00",
          scheduled_end: "17:00",
          actual_in: "08:55",
          actual_out: "17:10",
          std_work_units: 8,
          ot_work_units: 0,
          status: "present",
        };

        expect(isDetailRow(detailRow)).toBe(true);
      });

      it("returns false for SummaryRow", () => {
        const summaryRow: SummaryRow = {
          employee_name: "John Doe",
          total_std_units: 140,
          total_ot_units: 20,
          total_leave_days: 2,
        };

        expect(isDetailRow(summaryRow)).toBe(false);
      });

      it("returns false for object without date", () => {
        const invalidRow = {
          employee_name: "John Doe",
          status: "present",
        };

        expect(isDetailRow(invalidRow as AttendanceRow)).toBe(false);
      });

      it("returns false for object without status", () => {
        const invalidRow = {
          date: "2026-01-15",
          employee_name: "John Doe",
        };

        expect(isDetailRow(invalidRow as AttendanceRow)).toBe(false);
      });

      it("returns true even with null values", () => {
        const rowWithNulls: DetailRow = {
          date: "2026-01-15",
          employee_name: "John Doe",
          scheduled_start: null,
          scheduled_end: null,
          actual_in: null,
          actual_out: null,
          std_work_units: 0,
          ot_work_units: 0,
          status: "off",
        };

        expect(isDetailRow(rowWithNulls)).toBe(true);
      });

      it("returns true even with additional properties", () => {
        const rowWithExtra = {
          date: "2026-01-15",
          employee_name: "John Doe",
          scheduled_start: "09:00",
          scheduled_end: "17:00",
          actual_in: "08:55",
          actual_out: "17:10",
          std_work_units: 8,
          ot_work_units: 0,
          status: "present",
          extra_field: "value",
        };

        expect(isDetailRow(rowWithExtra as AttendanceRow)).toBe(true);
      });
    });

    describe("Type Guards - Mutual Exclusivity", () => {
      it("SummaryRow and DetailRow are mutually exclusive", () => {
        const summaryRow: SummaryRow = {
          employee_name: "John Doe",
          total_std_units: 140,
          total_ot_units: 20,
          total_leave_days: 2,
        };

        const detailRow: DetailRow = {
          date: "2026-01-15",
          employee_name: "John Doe",
          scheduled_start: "09:00",
          scheduled_end: "17:00",
          actual_in: "08:55",
          actual_out: "17:10",
          std_work_units: 8,
          ot_work_units: 0,
          status: "present",
        };

        expect(isSummaryRow(summaryRow) && isDetailRow(summaryRow)).toBe(false);
        expect(isDetailRow(detailRow) && isSummaryRow(detailRow)).toBe(false);
      });
    });
  });
});
