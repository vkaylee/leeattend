import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("../helpers", () => ({
  apiFetch: vi.fn(),
}));

import {
  createBehalfAssignmentApi,
  type BehalfAssignmentConfig,
  type BehalfAssignmentApi,
} from "../behalf-assignment-factory";
import { apiFetch } from "../helpers";

// ── Test helpers ──────────────────────────────────────────────────────────

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

function errRes(status: number, body: unknown) {
  return {
    ok: false,
    status,
    statusText: String(status),
    headers: new Headers({ "content-type": "application/json" }),
    json: () => Promise.resolve(body),
    text: () => Promise.resolve(JSON.stringify(body)),
    clone() {
      return this;
    },
  } as unknown as Response;
}

// ── Test type (mirrors OtClaimBehalfAssignment shape) ────────────────────

interface TestAssignment {
  assignee_user_id: string;
  assignment_type: string;
  created_at: string;
  id: string;
  target_department_id?: string | null;
  target_employee_id?: string | null;
  updated_at: string;
}

const BASE_CONFIG: BehalfAssignmentConfig = {
  basePath: "/v1/test/behalf-assignments",
};

function makeApi<T = TestAssignment>(
  overrides?: Partial<BehalfAssignmentConfig>,
): BehalfAssignmentApi<T> {
  return createBehalfAssignmentApi<T>({ ...BASE_CONFIG, ...overrides });
}

// ── Tests ────────────────────────────────────────────────────────────────

describe("createBehalfAssignmentApi", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(apiFetch).mockReset();
  });

  // ── list ───────────────────────────────────────────────────────────────

  describe("list", () => {
    it("sends GET to the collection URL", async () => {
      const api = makeApi();
      vi.mocked(apiFetch).mockResolvedValueOnce(okRes([]));

      await api.list();

      const [url, opts] = vi.mocked(apiFetch).mock.calls[0];
      expect(url).toBe("/v1/test/behalf-assignments");
      expect(opts).toBeUndefined(); // GET uses default options
    });

    it("returns the parsed JSON array", async () => {
      const api = makeApi();
      const mock: TestAssignment[] = [
        {
          id: "ba-1",
          assignee_user_id: "u1",
          assignment_type: "ot_claim",
          created_at: "2026-01-01T00:00:00Z",
          updated_at: "2026-01-01T00:00:00Z",
        },
        {
          id: "ba-2",
          assignee_user_id: "u2",
          assignment_type: "ot_claim",
          created_at: "2026-01-02T00:00:00Z",
          updated_at: "2026-01-02T00:00:00Z",
        },
      ];
      vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));

      const result = await api.list();
      expect(result).toEqual(mock);
    });

    it("throws ApiError when response is not ok", async () => {
      const api = makeApi();
      vi.mocked(apiFetch).mockResolvedValueOnce(
        errRes(500, { error: "Internal Server Error" }),
      );

      await expect(api.list()).rejects.toThrow("[500]");
    });
  });

  // ── list with dataWrapper ──────────────────────────────────────────────

  describe("list with dataWrapper", () => {
    it("unwraps { data: [...] } when dataWrapper is true", async () => {
      const api = makeApi({ dataWrapper: true });
      const inner = [{ id: "ba-1", assignee_user_id: "u1" }];
      vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ data: inner }));

      const result = await api.list();
      expect(result).toEqual(inner);
    });

    it("returns empty array when wrapped data is empty", async () => {
      const api = makeApi({ dataWrapper: true });
      vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ data: [] }));

      const result = await api.list();
      expect(result).toEqual([]);
    });
  });

  // ── create ─────────────────────────────────────────────────────────────

  describe("create", () => {
    it("sends POST to the collection URL with JSON body", async () => {
      const api = makeApi();
      const mock: TestAssignment = {
        id: "ba-new",
        assignee_user_id: "u1",
        assignment_type: "ot_claim",
        created_at: "2026-06-01T00:00:00Z",
        updated_at: "2026-06-01T00:00:00Z",
      };
      vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));

      const result = await api.create({
        assignee_user_id: "u1",
      });

      const [url, opts] = vi.mocked(apiFetch).mock.calls[0];
      expect(url).toBe("/v1/test/behalf-assignments");
      expect((opts as RequestInit).method).toBe("POST");
      expect(
        JSON.parse((opts as RequestInit).body as string),
      ).toEqual({ assignee_user_id: "u1" });
      expect(result).toEqual(mock);
    });

    it("includes optional fields in the POST body", async () => {
      const api = makeApi();
      const mock: TestAssignment = {
        id: "ba-new",
        assignee_user_id: "u1",
        assignment_type: "ot_claim",
        target_department_id: "dept-1",
        target_employee_id: "emp-1",
        created_at: "2026-06-01T00:00:00Z",
        updated_at: "2026-06-01T00:00:00Z",
      };
      vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));

      await api.create({
        assignee_user_id: "u1",
        target_department_id: "dept-1",
        target_employee_id: "emp-1",
      });

      const body = JSON.parse(
        (vi.mocked(apiFetch).mock.calls[0][1] as RequestInit).body as string,
      );
      expect(body.assignee_user_id).toBe("u1");
      expect(body.target_department_id).toBe("dept-1");
      expect(body.target_employee_id).toBe("emp-1");
    });

    it("throws ApiError when create fails", async () => {
      const api = makeApi();
      vi.mocked(apiFetch).mockResolvedValueOnce(
        errRes(409, { error: "Duplicate assignment" }),
      );

      await expect(
        api.create({ assignee_user_id: "u1" }),
      ).rejects.toThrow("[409]");
    });
  });

  // ── delete ─────────────────────────────────────────────────────────────

  describe("delete", () => {
    it("sends DELETE to the item URL (id substituted)", async () => {
      const api = makeApi();
      vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));

      await api.delete("ba-99");

      const [url, opts] = vi.mocked(apiFetch).mock.calls[0];
      expect(url).toBe(
        "/v1/test/behalf-assignments/ba-99",
      );
      expect((opts as RequestInit).method).toBe("DELETE");
    });

    it("resolves void on success", async () => {
      const api = makeApi();
      vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));

      await expect(api.delete("ba-1")).resolves.toBeUndefined();
    });

    it("throws ApiError when delete fails", async () => {
      const api = makeApi();
      vi.mocked(apiFetch).mockResolvedValueOnce(
        errRes(404, { error: "Not found" }),
      );

      await expect(api.delete("ba-missing")).rejects.toThrow(
        "[404]",
      );
    });
  });

  // ── Integration: multiple instances with different configs ─────────────

  describe("multiple instances", () => {
    it("routes to different base paths independently", async () => {
      const otApi = makeApi({
        basePath: "/v1/attendance/claim-ot/behalf-assignments",
      });
      const adjApi = makeApi({
        basePath: "/v1/attendance/adjustments/behalf-assignments",
      });

      vi.mocked(apiFetch)
        .mockResolvedValueOnce(okRes([{ id: "ot-1" }]))
        .mockResolvedValueOnce(okRes([{ id: "adj-1" }]));

      const otResult = await otApi.list();
      const adjResult = await adjApi.list();

      expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/claim-ot/");
      expect(vi.mocked(apiFetch).mock.calls[1][0]).toContain("/adjustments/");
      expect(otResult).toEqual([{ id: "ot-1" }]);
      expect(adjResult).toEqual([{ id: "adj-1" }]);
    });
  });

  // ── Default type parameter ─────────────────────────────────────────────

  describe("default type parameter", () => {
    it("creates an API without specifying a type argument", async () => {
      // Using the default BehalfAssignment type (from generated schema).
      // We pass no type arg — just verify construction + basic call.
      const api = createBehalfAssignmentApi(BASE_CONFIG);
      vi.mocked(apiFetch).mockResolvedValueOnce(okRes([]));

      const result = await api.list();
      expect(result).toEqual([]);
    });
  });
});
