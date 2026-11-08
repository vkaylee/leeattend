import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock the fetch chokepoint. Every client.METHOD in documents.ts routes through
// customFetch → apiFetch, so mocking apiFetch intercepts every document call.
vi.mock("../helpers", () => ({
  apiFetch: vi.fn(),
}));

import {
  fetchFolders,
  createFolder,
  renameFolder,
  deleteFolder,
  distributeDocument,
  fetchPendingAcknowledgements,
  acknowledgeDocument,
  previewTemplate,
} from "../documents";
import { apiFetch } from "../helpers";

// Bare-payload response (see payroll.test.ts for why).
function okRes(payload: unknown) {
  return {
    ok: true,
    status: 200,
    statusText: "OK",
    headers: new Headers({ "content-type": "application/json" }),
    json: () => Promise.resolve(payload),
    text: () => Promise.resolve(JSON.stringify(payload)),
    clone() {
      return this;
    },
  } as unknown as Response;
}

function errRes(status: number, statusText: string, body: unknown) {
  return {
    ok: false,
    status,
    statusText,
    headers: new Headers({ "content-type": "application/json" }),
    json: () => Promise.resolve(body),
    text: () => Promise.resolve(JSON.stringify(body)),
    clone() {
      return this;
    },
  } as unknown as Response;
}

describe("documents API", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetchFolders: happy path returns folders from the flat endpoint", async () => {
    const mock = [{ id: "f-1", name: "HR" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchFolders();
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toBe("/v1/documents/folders");
  });

  it("fetchFolders: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", { message: "boom" }));
    await expect(fetchFolders()).rejects.toThrow();
  });

  it("createFolder: POST forwards name + parent_folder_id", async () => {
    const mock = { id: "f-new", name: "Finance", parent_folder_id: null };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createFolder("Finance");
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toBe("/v1/documents/folders");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("POST");
    const body = JSON.parse(opts.body as string);
    expect(body.name).toBe("Finance");
    expect(body.parent_folder_id).toBeNull();
  });

  it("createFolder: forwards parentFolderId when provided", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "f-child" }));
    await createFolder("Child", "f-parent");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(JSON.parse(opts.body as string).parent_folder_id).toBe("f-parent");
  });

  it("createFolder: ApiError on 403", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(403, "Forbidden", { error_code: "NO_ACCESS" }));
    await expect(createFolder("X")).rejects.toThrow();
  });

  it("renameFolder: PUT forwards name", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "f-1", name: "NewName" }));
    const result = await renameFolder("f-1", "NewName");
    expect(result).toEqual({ id: "f-1", name: "NewName" });
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("PUT");
    expect(JSON.parse(opts.body as string).name).toBe("NewName");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toBe("/v1/documents/folders/f-1");
  });

  it("deleteFolder: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ status: "success" }));
    await expect(deleteFolder("f-1")).resolves.toBeUndefined();
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("DELETE");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toBe("/v1/documents/folders/f-1");
  });

  it("distributeDocument: POST forwards document_id + target_employee_ids", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ status: "success", count: 2 }));
    const result = await distributeDocument("doc-1", ["e1", "e2"]);
    expect(result).toEqual({ status: "success", count: 2 });
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toBe("/v1/documents/distributions");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    const body = JSON.parse(opts.body as string);
    expect(body.document_id).toBe("doc-1");
    expect(body.target_employee_ids).toEqual(["e1", "e2"]);
  });

  it("fetchPendingAcknowledgements: GET", async () => {
    const mock = [{ id: "ack-1", document_id: "doc-1", employee_id: "emp-1", status: "pending" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchPendingAcknowledgements();
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toBe("/v1/ess/documents/pending");
  });

  it("fetchPendingAcknowledgements: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(fetchPendingAcknowledgements()).rejects.toThrow();
  });

  it("acknowledgeDocument: POST acknowledge", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ status: "acknowledged" }));
    const result = await acknowledgeDocument("ack-1");
    expect(result).toEqual({ status: "acknowledged" });
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toBe("/v1/ess/documents/ack-1/acknowledge");
  });

  it("previewTemplate: POST forwards template + employee + optional variables", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ filled_content: "Hello Bob" }));
    const result = await previewTemplate("Hello {{name}}", "emp-1", { name: "Bob" });
    expect(result).toEqual({ filled_content: "Hello Bob" });
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toBe("/v1/documents/templates/preview");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    const body = JSON.parse(opts.body as string);
    expect(body.template_content).toBe("Hello {{name}}");
    expect(body.employee_id).toBe("emp-1");
    expect(body.manual_variables).toEqual({ name: "Bob" });
  });

  it("previewTemplate: manual_variables defaults to null when omitted", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ filled_content: "" }));
    await previewTemplate("Hi", "emp-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(JSON.parse(opts.body as string).manual_variables).toBeNull();
  });
});
