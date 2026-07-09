import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock the dependencies BEFORE importing the file under test.
// apiFetch is the single chokepoint: customFetch (used by every typed
// client.METHOD call) routes through it, so mocking here intercepts every
// GET/POST/PUT/DELETE/PATCH in payroll.ts without touching the network.
vi.mock("../helpers", () => ({
  apiFetch: vi.fn(),
}));

import { apiFetch } from "../helpers";
import {
  fetchPayrollSettings,
  updatePayrollSettings,
  fetchPayrollSettingsHistory,
  fetchSalaryComponents,
  createSalaryComponent,
  fetchIncentivePolicies,
  createIncentivePolicy,
  cloneIncentivePolicy,
  deleteIncentivePolicy,
  createSalaryBaseGroup,
  fetchSalaryBaseGroups,
  assignComponentToSalaryBase,
  unassignComponentFromSalaryBase,
  fetchEmployeeSalaryComponents,
  assignEmployeeSalaryComponent,
  calculate_payroll,
  fetchEmployeePayslips,
  fetchEmployeePayrollConfigs,
  fetchOvertimePolicies,
  fetchOvertimePolicy,
  createOvertimePolicy,
  addOvertimeRule,
  setEmployeePayrollConfig,
  fetchDepartmentPayrollConfigs,
  setDepartmentPayrollConfig,
  calculateBatchPayslips,
  fetchWorkspacePayslips,
  assignOvertimePolicy,
  fetchPayrollRules,
  createPayrollRule,
  fetchPayrollRuleTiers,
  createPayrollRuleTier,
  fetchPayrollRuleAssignments,
  createPayrollRuleAssignment,
  deletePayrollRuleAssignment,
} from "../payroll";

// --- response helpers -------------------------------------------------------
// openapi-fetch calls res.json() and resolves the promise with the parsed body
// directly as `data`. Source functions destructure `{ data: resData }` from the
// client call result, so our fake Response.json() must return the bare wire
// payload (NOT wrapped in { data }). Example: json() returns the array/object
// the server sent, and openapi-fetch hands that straight to the caller as data.
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

describe("payroll API", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetchPayrollSettings: happy path returns settings", async () => {
    const mock = { id: "ps-1", payroll_cycle: "monthly" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));

    const result = await fetchPayrollSettings("ws-1");
    expect(result).toEqual(mock);
    expect(apiFetch).toHaveBeenCalledTimes(1);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/workspaces/ws-1/payroll-settings");
  });

  it("fetchPayrollSettings: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Internal Server Error", { message: "boom" }));
    await expect(fetchPayrollSettings("ws-1")).rejects.toThrow();
  });

  it("fetchSalaryComponents: happy path", async () => {
    const mock = [{ id: "sc-1", name: "Basic", code: "BASIC" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchSalaryComponents("ws-1");
    expect(result).toEqual(mock);
  });

  it("createSalaryComponent: POST forwards body, returns created record", async () => {
    const body = { name: "HRA", code: "HRA", component_type: "earning", calculation_type: "fixed" };
    const created = { id: "sc-new", ...body };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(created));
    const result = await createSalaryComponent("ws-1", body as never);
    expect(result).toEqual(created);
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/workspaces/ws-1/salary-components");
    expect(call[1] && (call[1] as RequestInit).method).toBe("POST");
  });

  it("createSalaryComponent: ApiError on 422 validation", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(422, "Unprocessable", { error_code: "VALIDATION_FAILED" }));
    await expect(createSalaryComponent("ws-1", {} as never)).rejects.toThrow();
  });

  it("fetchIncentivePolicies: happy path", async () => {
    const mock = [{ id: "ip-1", name: "Q1", payout_type: "fixed" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchIncentivePolicies("ws-1");
    expect(result).toEqual(mock);
  });

  it("createIncentivePolicy: POST forwards body", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await createIncentivePolicy("ws-1", {
      base_amount: 100,
      effective_from: "2026-01-01",
      name: "Bonus",
      payout_type: "fixed",
      tiers: [],
    });
    expect(apiFetch).toHaveBeenCalledTimes(1);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("POST");
    expect(JSON.parse(opts.body as string).name).toBe("Bonus");
  });

  it("cloneIncentivePolicy: POST to clone path", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await cloneIncentivePolicy("ws-1", "policy-1");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/payroll/incentive-policies/policy-1/clone");
  });

  it("deleteIncentivePolicy: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deleteIncentivePolicy("ws-1", "policy-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("DELETE");
  });

  it("createSalaryBaseGroup: returns created group", async () => {
    const mock = { id: "bg-1", name: "MGMT", slug: "mgmt" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createSalaryBaseGroup("ws-1", { name: "MGMT", slug: "mgmt" });
    expect(result).toEqual(mock);
  });

  it("createSalaryBaseGroup: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(createSalaryBaseGroup("ws-1", { name: "X", slug: "x" })).rejects.toThrow();
  });

  it("fetchSalaryBaseGroups: happy path", async () => {
    const mock = { groups: [{ id: "bg-1" }], members: [] };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchSalaryBaseGroups("ws-1");
    expect(result).toEqual(mock);
  });

  it("assignComponentToSalaryBase: POST body forwards component id", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await assignComponentToSalaryBase("ws-1", "base-1", "comp-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(JSON.parse(opts.body as string).component_definition_id).toBe("comp-1");
  });

  it("unassignComponentFromSalaryBase: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await unassignComponentFromSalaryBase("ws-1", "base-1", "comp-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("DELETE");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("base-1/members/comp-1");
  });

  it("fetchEmployeeSalaryComponents: happy path", async () => {
    const mock = [{ id: "esc-1", amount: "100" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchEmployeeSalaryComponents("ws-1", "emp-1");
    expect(result).toEqual(mock);
  });

  it("assignEmployeeSalaryComponent: POST returns record", async () => {
    const mock = { id: "esc-new", amount: "500" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await assignEmployeeSalaryComponent("ws-1", "emp-1", {
      amount: "500",
      component_definition_id: "cd-1",
      effective_date: "2026-01-01",
    });
    expect(result).toEqual(mock);
  });

  it("calculate_payroll: POST with query year/month", async () => {
    const mock = { id: "ps-1", gross: 1000, net: 900 };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await calculate_payroll("ws-1", "emp-1", 2026, 3);
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/employees/emp-1/calculate-payroll");
  });

  it("calculate_payroll: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(calculate_payroll("ws-1", "emp-1", 2026, 3)).rejects.toThrow();
  });

  it("fetchEmployeePayslips: happy path", async () => {
    const mock = [{ id: "p-1", net: 900 }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchEmployeePayslips("ws-1", "emp-1");
    expect(result).toEqual(mock);
  });

  it("fetchEmployeePayrollConfigs: happy path", async () => {
    const mock = [{ id: "cfg-1", hours_per_day: "8" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchEmployeePayrollConfigs("ws-1", "emp-1");
    expect(result).toEqual(mock);
  });

  it("fetchOvertimePolicies: returns data array", async () => {
    const mock = [{ id: "op-1", name: "OT-Weekday" }];
    // fetchOvertimePolicies accesses resData.data, so payload needs a data key.
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ data: mock }));
    const result = await fetchOvertimePolicies("ws-1");
    expect(result).toEqual(mock);
  });

  it("fetchOvertimePolicy: returns [policy, rules] tuple", async () => {
    const policy = { id: "op-1" };
    const rules = [{ id: "or-1" }];
    // The implementation casts resData `as unknown as [policy, rules]`.
    // We return an actual array of length 2 so both the Array.isArray cast
    // and the positional tuple semantics are exercised.
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes([policy, rules]));
    const result = await fetchOvertimePolicy("ws-1", "op-1");
    expect(Array.isArray(result)).toBe(true);
    expect((result as unknown[]).length).toBe(2);
  });

  it("createOvertimePolicy: POST returns policy", async () => {
    const mock = { id: "op-new", name: "New" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createOvertimePolicy("ws-1", { name: "New" } as never);
    expect(result).toEqual(mock);
  });

  it("addOvertimeRule: POST returns rule", async () => {
    const mock = { id: "or-new", multiplier: "1.5" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await addOvertimeRule("ws-1", "op-1", {
      days_of_week: 1,
      is_holiday: false,
      multiplier: "1.5",
      priority: 1,
    });
    expect(result).toEqual(mock);
  });

  it("setEmployeePayrollConfig: POST", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ ok: true }));
    const result = await setEmployeePayrollConfig("ws-1", "emp-1", {
      effective_date: "2026-01-01",
      hours_per_day: "8",
      std_working_days: "22",
    });
    expect(result).toEqual({ ok: true });
  });

  it("fetchDepartmentPayrollConfigs: GET", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ cfg: true }));
    const result = await fetchDepartmentPayrollConfigs("ws-1", "dept-1");
    expect(result).toEqual({ cfg: true });
  });

  it("setDepartmentPayrollConfig: POST", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ ok: true }));
    const result = await setDepartmentPayrollConfig("ws-1", "dept-1", {
      effective_date: "2026-01-01",
      hours_per_day: "8",
      std_working_days: "22",
    });
    expect(result).toEqual({ ok: true });
  });

  it("calculateBatchPayslips: POST body with employment_ids + year/month", async () => {
    const mock = { results: [{ employee_id: "e1", net: 100 }] };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await calculateBatchPayslips("ws-1", { employment_ids: ["e1"], year: 2026, month: 3 });
    expect(result).toEqual(mock);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    const body = JSON.parse(opts.body as string);
    expect(body.employment_ids).toEqual(["e1"]);
    expect(body.year).toBe(2026);
    expect(body.month).toBe(3);
  });

  it("calculateBatchPayslips: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(calculateBatchPayslips("ws-1", { employment_ids: ["e1"], year: 2026, month: 3 })).rejects.toThrow();
  });

  it("fetchWorkspacePayslips: GET with query year/month", async () => {
    const mock = [{ id: "p-1", net: 900 }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchWorkspacePayslips("ws-1", 2026, 3);
    expect(result).toEqual(mock);
  });

  it("assignOvertimePolicy: POST forwards body", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await assignOvertimePolicy("ws-1", {
      assignee_type: "employee",
      effective_from: "2026-01-01",
      policy_id: "op-1",
    });
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    const body = JSON.parse(opts.body as string);
    expect(body.assignee_type).toBe("employee");
  });

  it("fetchPayrollRules: GET list", async () => {
    const mock = [{ id: "pr-1", name: "Tax" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchPayrollRules("ws-1");
    expect(result).toEqual(mock);
  });

  it("createPayrollRule: POST returns created", async () => {
    const mock = { id: "pr-new", name: "Tax2" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createPayrollRule("ws-1", { name: "Tax2" } as never);
    expect(result).toEqual(mock);
  });

  it("fetchPayrollRuleTiers: GET list", async () => {
    const mock = [{ id: "t-1", tier_order: 1 }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchPayrollRuleTiers("ws-1", "pr-1");
    expect(result).toEqual(mock);
  });

  it("createPayrollRuleTier: POST returns tier", async () => {
    const mock = { id: "t-new", tier_order: 2 };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createPayrollRuleTier("ws-1", "pr-1", {
      min_amount: "0",
      rate: "5",
      tier_order: 2,
    });
    expect(result).toEqual(mock);
  });

  it("fetchPayrollRuleAssignments: GET list", async () => {
    const mock = [{ id: "a-1", assignee_type: "employee" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchPayrollRuleAssignments("ws-1", "pr-1");
    expect(result).toEqual(mock);
  });

  // --- updatePayrollSettings + fetchPayrollSettingsHistory ------------------
  // These two live at lines 91-104 (the previous gap in coverage). Both are
  // simple GET/PATCH wrappers, tested here with param forwarding + error path.
  it("updatePayrollSettings: PATCH forwards body", async () => {
    const mock = { id: "ps-1", payroll_cycle: "bi-weekly" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await updatePayrollSettings("ws-1", { payroll_cycle: "bi-weekly" } as never);
    expect(result).toEqual(mock);
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/workspaces/ws-1/payroll-settings");
    expect((call[1] as RequestInit).method).toBe("PATCH");
    expect(JSON.parse((call[1] as RequestInit).body as string).payroll_cycle).toBe("bi-weekly");
  });

  it("updatePayrollSettings: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(updatePayrollSettings("ws-1", {} as never)).rejects.toThrow();
  });

  it("fetchPayrollSettingsHistory: GET history list", async () => {
    const mock = [
      { id: "ps-h1", payroll_cycle: "monthly", created_at: "2026-01-01" },
      { id: "ps-h2", payroll_cycle: "bi-weekly", created_at: "2026-02-01" },
    ];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchPayrollSettingsHistory("ws-1");
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/payroll-settings/history");
  });

  it("fetchPayrollSettingsHistory: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(fetchPayrollSettingsHistory("ws-1")).rejects.toThrow();
  });

  it("createPayrollRuleAssignment: POST returns assignment", async () => {
    const mock = { id: "a-new", assignee_type: "employee" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createPayrollRuleAssignment("ws-1", "pr-1", {
      assignee_type: "employee",
      effective_from: "2026-01-01",
    } as never);
    expect(result).toEqual(mock);
  });

  it("deletePayrollRuleAssignment: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deletePayrollRuleAssignment("ws-1", "pr-1", "a-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("DELETE");
  });
});
