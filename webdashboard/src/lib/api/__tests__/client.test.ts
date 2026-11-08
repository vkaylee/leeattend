import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { buildUrl, client } from "../client";
import { ApiError } from "../errors";
import { apiFetch } from "../helpers";

vi.mock("../helpers", () => ({ apiFetch: vi.fn() }));

describe("API client", () => {
  beforeEach(() => vi.clearAllMocks());
  afterEach(() => vi.restoreAllMocks());

  it("builds flat paths with encoded entity IDs and array query values", () => {
    expect(buildUrl("/v1/employees/{id}", {
      path: { id: "employee/1", ws_id: "ignored" },
      query: { ids: ["a", null, "b"], page: 2, status: "all", absent: undefined },
    })).toBe("/v1/employees/employee%2F1?ids=a&ids=b&page=2");
  });

  it("rejects missing path parameters instead of issuing malformed URLs", () => {
    expect(() => buildUrl("/v1/employees/{id}")).toThrow("Missing path parameter: id");
  });

  it("preserves JSON, FormData, and Blob request bodies", async () => {
    vi.mocked(apiFetch).mockResolvedValue(new Response(JSON.stringify({ id: "e1" }), {
      headers: { "content-type": "application/json" },
    }));
    await client.POST<{ id: string }>("/v1/employees", { body: { name: "A" } });
    expect(vi.mocked(apiFetch).mock.calls[0][1]).toMatchObject({
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name: "A" }),
    });

    const form = new FormData();
    form.append("file", new Blob(["data"]), "file.csv");
    vi.mocked(apiFetch).mockResolvedValueOnce(new Response("uploaded"));
    await client.POST<string>("/v1/files/upload", { body: form });
    expect(vi.mocked(apiFetch).mock.calls[1][1]?.body).toBe(form);
  });

  it("throws ApiError for failed responses and returns undefined for DELETE 204", async () => {
    const errorBody = { error_code: "VALIDATION_FAILED", message: "Invalid parameters" };
    vi.mocked(apiFetch).mockResolvedValueOnce(new Response(JSON.stringify(errorBody), {
      status: 400,
      statusText: "Bad Request",
      headers: { "content-type": "application/json" },
    }));
    await expect(client.GET("/v1/test")).rejects.toBeInstanceOf(ApiError);

    vi.mocked(apiFetch).mockResolvedValueOnce(new Response(null, { status: 204 }));
    const result = await client.DELETE<void>("/v1/employees/e1");
    expect(result.data).toBeUndefined();
  });
});
