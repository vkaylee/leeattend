import { describe, it, expect, vi, beforeEach, beforeAll } from "vitest";

import { downloadBlob } from "../download-helper";

// downloadBlob uses window.URL.createObjectURL/revokeObjectURL and
// document.createElement/body.appendChild/body.removeChild.  We stub
// these DOM APIs because happy-dom implements them incompletely.
beforeAll(() => {
  URL.createObjectURL = () => "blob:mock-url-12345";
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  URL.revokeObjectURL = () => {};
  if (!document.body) {
    document.body = document.createElement("body");
  }
});

describe("downloadBlob", () => {
  let anchorSpy: { click: () => void; href: string; download: string };

  beforeEach(() => {
    vi.clearAllMocks();

    anchorSpy = { click: vi.fn(), href: "", download: "" };

    vi.spyOn(document, "createElement").mockReturnValue(
      anchorSpy as unknown as HTMLAnchorElement,
    );
    vi.spyOn(document.body, "appendChild").mockImplementation(
      (node) => node,
    );
    vi.spyOn(document.body, "removeChild").mockImplementation(
      (node) => node,
    );
    vi.spyOn(window.URL, "createObjectURL").mockReturnValue("blob:mock-url-12345");
    vi.spyOn(window.URL, "revokeObjectURL");
  });

  it("creates an object URL from the blob", () => {
    const blob = new Blob(["test content"]);
    downloadBlob(blob, "report.xlsx");

    expect(window.URL.createObjectURL).toHaveBeenCalledWith(blob);
  });

  it("sets the anchor href to the object URL", () => {
    const blob = new Blob(["data"]);
    downloadBlob(blob, "export.csv");

    expect(anchorSpy.href).toBe("blob:mock-url-12345");
  });

  it("sets the anchor download attribute to the given filename", () => {
    const blob = new Blob(["data"]);
    downloadBlob(blob, "my-template.xlsx");

    expect(anchorSpy.download).toBe("my-template.xlsx");
  });

  it("appends the anchor to the document body", () => {
    const blob = new Blob(["data"]);
    downloadBlob(blob, "file.pdf");

    expect(document.body.appendChild).toHaveBeenCalledTimes(1);
    const appended = (document.body.appendChild as ReturnType<typeof vi.fn>).mock
      .calls[0][0];
    expect(appended).toBe(anchorSpy);
  });

  it("clicks the anchor to trigger the download", () => {
    const blob = new Blob(["data"]);
    downloadBlob(blob, "file.pdf");

    expect(anchorSpy.click).toHaveBeenCalledTimes(1);
  });

  it("removes the anchor from the document body after clicking", () => {
    const blob = new Blob(["data"]);
    downloadBlob(blob, "file.pdf");

    expect(document.body.removeChild).toHaveBeenCalledWith(anchorSpy);
    // appendChild must be called before removeChild
    const appendOrder = (
      document.body.appendChild as ReturnType<typeof vi.fn>
    ).mock.invocationCallOrder[0];
    const removeOrder = (
      document.body.removeChild as ReturnType<typeof vi.fn>
    ).mock.invocationCallOrder[0];
    expect(appendOrder).toBeLessThan(removeOrder);
  });

  it("revokes the object URL after download", () => {
    const blob = new Blob(["data"]);
    downloadBlob(blob, "file.pdf");

    expect(window.URL.revokeObjectURL).toHaveBeenCalledWith(
      "blob:mock-url-12345",
    );
  });

  it("handles filenames with special characters", () => {
    const blob = new Blob(["data"]);
    downloadBlob(blob, "leave_import_errors_job-abc-123.xlsx");

    expect(anchorSpy.download).toBe(
      "leave_import_errors_job-abc-123.xlsx",
    );
  });
});
