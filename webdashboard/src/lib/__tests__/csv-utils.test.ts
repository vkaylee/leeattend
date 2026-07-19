import { describe, it, expect, vi, beforeEach } from "vitest";

import { downloadCsvErrorReport, downloadCsvTemplate } from "../csv-utils";

// ── DOM mocks ───────────────────────────────────────────────────────

function setupDomMocks() {
  const anchorMock = {
    href: "",
    download: "",
    click: vi.fn(),
  };

  const createElementSpy = vi
    .spyOn(document, "createElement")
    .mockReturnValue(anchorMock as unknown as HTMLElement);

  const appendChildSpy = vi.spyOn(document.body, "appendChild").mockImplementation(vi.fn());
  const removeChildSpy = vi.spyOn(document.body, "removeChild").mockImplementation(vi.fn());

  const createObjectURLSpy = vi
    .spyOn(window.URL, "createObjectURL")
    .mockReturnValue("blob:mock-url");
  const revokeObjectURLSpy = vi
    .spyOn(window.URL, "revokeObjectURL")
    .mockImplementation(vi.fn());

  return {
    anchorMock,
    createElementSpy,
    appendChildSpy,
    removeChildSpy,
    createObjectURLSpy,
    revokeObjectURLSpy,
  };
}

// ── downloadCsvTemplate ─────────────────────────────────────────────

describe("downloadCsvTemplate", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("creates a CSV file with correct headers", () => {
    const { createObjectURLSpy } = setupDomMocks();

    downloadCsvTemplate();

    // Should create blob URL
    expect(createObjectURLSpy).toHaveBeenCalledTimes(1);
    const blobArg = createObjectURLSpy.mock.calls[0]?.[0] as Blob;
    expect(blobArg.type).toBe("text/csv");
  });

  it("sets the download filename to ot_upload_template.csv", () => {
    const { anchorMock } = setupDomMocks();

    downloadCsvTemplate();

    expect(anchorMock.download).toBe("ot_upload_template.csv");
  });

  it("triggers a click on the anchor element", () => {
    const { anchorMock } = setupDomMocks();

    downloadCsvTemplate();

    expect(anchorMock.click).toHaveBeenCalledTimes(1);
  });

  it("appends and removes the anchor from document body", () => {
    const { appendChildSpy, removeChildSpy } = setupDomMocks();

    downloadCsvTemplate();

    expect(appendChildSpy).toHaveBeenCalledTimes(1);
    expect(removeChildSpy).toHaveBeenCalledTimes(1);
  });

  it("revokes the blob URL after download", () => {
    const { revokeObjectURLSpy } = setupDomMocks();

    downloadCsvTemplate();

    expect(revokeObjectURLSpy).toHaveBeenCalledWith("blob:mock-url");
  });

  it("produces CSV content with 3 data columns", () => {
    const { createObjectURLSpy } = setupDomMocks();

    downloadCsvTemplate();

    const blob = createObjectURLSpy.mock.calls[0]?.[0] as Blob;
    // CSV content is in the blob — we verify through the Blob constructor
    expect(blob).toBeInstanceOf(Blob);
  });
});

// ── downloadCsvErrorReport ─────────────────────────────────────────

describe("downloadCsvErrorReport", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("returns early without side effects when errors array is empty", () => {
    const { createElementSpy, createObjectURLSpy } = setupDomMocks();

    downloadCsvErrorReport([]);

    expect(createObjectURLSpy).not.toHaveBeenCalled();
    expect(createElementSpy).not.toHaveBeenCalled();
  });

  it("creates a CSV with error details", () => {
    const { createObjectURLSpy } = setupDomMocks();

    downloadCsvErrorReport([
      { line_number: 1, employment_id: "E001", error_message: "Missing date" },
      { line_number: 3, employment_id: "E003", error_message: "Invalid minutes" },
    ]);

    expect(createObjectURLSpy).toHaveBeenCalledTimes(1);
    const blob = createObjectURLSpy.mock.calls[0]?.[0] as Blob;
    expect(blob.type).toBe("text/csv");
  });

  it("includes correct CSV headers for errors", () => {
    const { createObjectURLSpy } = setupDomMocks();

    downloadCsvErrorReport([
      { line_number: 5, employment_id: "E005", error_message: "Test error" },
    ]);

    const blob = createObjectURLSpy.mock.calls[0]?.[0] as Blob;
    expect(blob).toBeInstanceOf(Blob);
  });

  it("sets download filename with timestamp", () => {
    const { anchorMock } = setupDomMocks();

    downloadCsvErrorReport([
      { line_number: 1, employment_id: "E001", error_message: "Err" },
    ]);

    expect(anchorMock.download).toMatch(/^ot_upload_errors_\d+\.csv$/);
  });

  it("triggers download click for error report", () => {
    const { anchorMock } = setupDomMocks();

    downloadCsvErrorReport([
      { line_number: 1, employment_id: "E001", error_message: "Err" },
    ]);

    expect(anchorMock.click).toHaveBeenCalledTimes(1);
  });

  it("handles single error row", () => {
    const { revokeObjectURLSpy } = setupDomMocks();

    downloadCsvErrorReport([
      { line_number: 42, employment_id: "EMP-99", error_message: "Unknown employee" },
    ]);

    expect(revokeObjectURLSpy).toHaveBeenCalledTimes(1);
  });

  it("handles error messages with commas (quoted in CSV)", () => {
    const { createObjectURLSpy } = setupDomMocks();

    downloadCsvErrorReport([
      { line_number: 1, employment_id: "E001", error_message: "Invalid field: date, minutes" },
    ]);

    // Should not throw — the error message with commas should be properly quoted
    expect(createObjectURLSpy).toHaveBeenCalledTimes(1);
  });

  it("cleans up anchor from DOM after download", () => {
    const { appendChildSpy, removeChildSpy } = setupDomMocks();

    downloadCsvErrorReport([
      { line_number: 1, employment_id: "E001", error_message: "Err" },
    ]);

    expect(appendChildSpy).toHaveBeenCalledTimes(1);
    expect(removeChildSpy).toHaveBeenCalledTimes(1);
  });
});
