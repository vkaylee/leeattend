import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock the fetch chokepoint. Every client.METHOD in documents.ts routes through
// customFetch → apiFetch, so mocking apiFetch intercepts every document call.
vi.mock("../helpers", () => ({
  apiFetch: vi.fn(),
}));

import { apiFetch } from "../helpers";
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

  it("fetchFolders: happy path returns folders", async () => {
    const mock = [{ id: "f-1", name: "HR" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchFolders("ws-1");
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/documents/folders");
  });

  it("fetchFolders: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", { message: "boom" }));
    await expect(fetchFolders("ws-1")).rejects.toThrow();
  });

  it("createFolder: POST forwards name + parent_folder_id", async () => {
    const mock = { id: "f-new", name: "Finance", parent_folder_id: null };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createFolder("ws-1", "Finance");
    expect(result).toEqual(mock);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("POST");
    const body = JSON.parse(opts.body as string);
    expect(body.name).toBe("Finance");
    expect(body.parent_folder_id).toBeNull();
  });

  it("createFolder: forwards parentFolderId when provided", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "f-child" }));
    await createFolder("ws-1", "Child", "f-parent");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(JSON.parse(opts.body as string).parent_folder_id).toBe("f-parent");
  });

  it("createFolder: ApiError on 403", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(403, "Forbidden", { error_code: "NO_ACCESS" }));
    await expect(createFolder("ws-1", "X")).rejects.toThrow();
  });

  it("renameFolder: PUT forwards name", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "f-1", name: "NewName" }));
    const result = await renameFolder("ws-1", "f-1", "NewName");
    expect(result).toEqual({ id: "f-1", name: "NewName" });
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("PUT");
    expect(JSON.parse(opts.body as string).name).toBe("NewName");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/documents/folders/f-1");
  });

  it("deleteFolder: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ ok: true }));
    const result = await deleteFolder("ws-1", "f-1");
    expect(result).toEqual({ ok: true });
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("DELETE");
  });

  it("distributeDocument: POST forwards document_id + target_employee_ids", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "dist-1" }));
    const result = await distributeDocument("ws-1", "doc-1", ["e1", "e2"]);
    expect(result).toEqual({ id: "dist-1" });
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    const body = JSON.parse(opts.body as string);
    expect(body.document_id).toBe("doc-1");
    expect(body.target_employee_ids).toEqual(["e1", "e2"]);
  });

  it("fetchPendingAcknowledgements: GET", async () => {
    const mock = [{ id: "ack-1", doc: "Offer" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchPendingAcknowledgements();
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/ess/documents/pending");
  });

  it("fetchPendingAcknowledgements: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(fetchPendingAcknowledgements()).rejects.toThrow();
  });

  it("acknowledgeDocument: POST acknowledge", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "ack-1", status: "ack" }));
    const result = await acknowledgeDocument("ack-1");
    expect(result).toEqual({ id: "ack-1", status: "ack" });
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/ess/documents/ack-1/acknowledge");
  });

  it("previewTemplate: POST forwards template + employee + optional variables", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ rendered: "Hello Bob" }));
    const result = await previewTemplate("ws-1", "Hello {{name}}", "emp-1", { name: "Bob" });
    expect(result).toEqual({ rendered: "Hello Bob" });
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    const body = JSON.parse(opts.body as string);
    expect(body.template_content).toBe("Hello {{name}}");
    expect(body.employee_id).toBe("emp-1");
    expect(body.manual_variables).toEqual({ name: "Bob" });
  });

  it("previewTemplate: manual_variables defaults to null when omitted", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ rendered: "" }));
    await previewTemplate("ws-1", "Hi", "emp-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(JSON.parse(opts.body as string).manual_variables).toBeNull();
  });
});
