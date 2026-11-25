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
  fetchSalaryComponents,
  createSalaryComponent,
  fetchIncentivePolicies,
  createIncentivePolicy,
  cloneIncentivePolicy,
  deleteIncentivePolicy,
  fetchEmployeeSalaryComponents,
  assignEmployeeSalaryComponent,
  calculate_payroll,
  fetchEmployeePayslips,
  fetchOvertimePolicies,
  fetchOvertimePolicy,
  createOvertimePolicy,
  createOvertimePolicyVersion,
  publishOvertimePolicyVersion,
  calculateBatchPayslips,
  fetchPayslips,
  fetchPayrollRules,
  createPayrollRule,
  fetchPayrollRuleTiers,
  createPayrollRuleTier,
  fetchPayrollRuleAssignments,
  createPayrollRuleAssignment,
  deletePayrollRuleAssignment,
  fetchWorkPolicyMemberships,
  resolveWorkPolicyByRelationship,
  createWorkPolicyVersion,
  patchWorkPolicyVersion,
  publishWorkPolicyVersion,
  transferWorkPolicyMembership,
  simulateWorkPolicyVersionBatch,
  fetchWorkPolicyReferenceOptions,
  fetchPayrollImpact,
  fetchPayrollImpactBatch,
  simulatePayroll,
  simulatePayrollWithPolicy,
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
  it("fetches policy memberships", async () => {
    const memberships = [{ id: "m-1", employment_relationship_id: "rel-1", group_id: "g-1", effective_from: "2026-01-01" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(memberships));
    await expect(fetchWorkPolicyMemberships("g-1")).resolves.toEqual(memberships);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/work-policy-groups/g-1/memberships");
  });

  it("resolves policy by relationship at effective date", async () => {
    const policy = { version_id: "v-1", group_id: "g-1", effective_from: "2026-01-01", pay_method: "hourly" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(policy));
    await expect(resolveWorkPolicyByRelationship("rel-1", "2026-06-01")).resolves.toEqual(policy);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/employment-relationships/rel-1/work-policy");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("as_of=2026-06-01");
  });

  it("creates, patches, publishes, transfers, and batch-simulates work policy versions", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "v-2", group_id: "g-1", status: "draft", row_version: 1 }));
    await createWorkPolicyVersion("g-1", { clone_from_version_id: "v-1" });
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/work-policy-groups/g-1/versions");

    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "v-2", group_id: "g-1", status: "draft", row_version: 2 }));
    await patchWorkPolicyVersion("g-1", "v-2", { expected_row_version: 1 } as never);
    expect((vi.mocked(apiFetch).mock.calls[1][1] as RequestInit).method).toBe("PATCH");

    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "v-2", group_id: "g-1", status: "published", row_version: 3 }));
    await publishWorkPolicyVersion("g-1", "v-2", { expected_row_version: 2, auto_close_previous: true }, "idem-1");
    expect(vi.mocked(apiFetch).mock.calls[2][0]).toContain("/work-policy-groups/g-1/versions/v-2/publish");

    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ new_membership: { id: "m-2" } }));
    await transferWorkPolicyMembership({ employment_relationship_id: "rel-1", target_group_id: "g-2", effective_date: "2026-05-01" });
    expect(vi.mocked(apiFetch).mock.calls[3][0]).toContain("/work-policy-memberships/transfer");

    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ total: 1, computed: 1, affected: 1, uncomputable: 0, items: [] }));
    await simulateWorkPolicyVersionBatch("g-1", "v-2", { year: 2026, month: 3, employment_relationship_ids: ["rel-1"] });
    expect(vi.mocked(apiFetch).mock.calls[4][0]).toContain("/work-policy-groups/g-1/versions/v-2/simulate");
  });
  it("fetches payroll impact with stale metadata", async () => {
    const impact = { employment_id: "e-1", period_year: 2026, period_month: 6, before_run_id: "r-1", after_run_id: "r-2", computed: true, stale: true, stale_reasons: ["policy_version_changed"], base_pay_delta: "200", overtime_pay_delta: "50", deductions_delta: "10", net_pay_delta: "240", standard_units_delta: 2, overtime_units_delta: 1 };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(impact));
    await expect(fetchPayrollImpact("e-1", 2026, 6, "r-1", "r-2")).resolves.toEqual(impact);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/employments/e-1/payroll/impact");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("before_run_id=r-1");
  });

  it("fetches bounded payroll impact batches", async () => {
    const result = { total: 1, computed: 1, affected: 1, uncomputable: 0, items: [] };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(result));
    await expect(fetchPayrollImpactBatch({ employment_ids: ["e-1"], year: 2026, month: 6, before_run_id: "r-1", after_run_id: "r-2" })).resolves.toEqual(result);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/payroll/impact/batch");
    expect((vi.mocked(apiFetch).mock.calls[0][1] as RequestInit).method).toBe("POST");
  });

  it("simulates payroll with current policy", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "p-1", base_pay_amount: "1000", net_pay_amount: "900" }));
    await expect(simulatePayroll("employee-1", 2026, 6)).resolves.toMatchObject({ id: "p-1" });
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/employees/employee-1/simulate-payroll");
    expect(call[0]).not.toContain("policy_version_id");
    expect((call[1] as RequestInit).method).toBe("POST");
  });

  it("simulates payroll with selected policy version", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "p-1", base_pay_amount: "1000", net_pay_amount: "900" }));
    await expect(simulatePayrollWithPolicy("employee-1", 2026, 6, "policy-1")).resolves.toMatchObject({ id: "p-1" });
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/employees/employee-1/simulate-payroll");
    expect(call[0]).toContain("policy_version_id=policy-1");
    expect((call[1] as RequestInit).method).toBe("POST");
  });

  it("fetches work policy v2 reference options in parallel", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes([{ id: "pattern-1", name: "Weekly" }]));
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ data: [{ id: "overtime-1", name: "Standard OT", latest_published_version_id: "ot-v1", latest_published_version_no: 1 }] }));
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes([]));
    const result = await fetchWorkPolicyReferenceOptions();
    expect(result.overtimeVersions).toEqual([{ id: "ot-v1", label: "Standard OT (v1)" }]);
    expect(result.patterns).toEqual([{ id: "pattern-1", label: "Weekly" }]);
  });

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetchSalaryComponents: happy path", async () => {
    const mock = [{ id: "sc-1", name: "Basic", code: "BASIC" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchSalaryComponents();
    expect(result).toEqual(mock);
  });

  it("createSalaryComponent: POST forwards body, returns created record", async () => {
    const body = { name: "HRA", code: "HRA" };
    const created = { id: "sc-new", ...body };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(created));
    const result = await createSalaryComponent(body);
    expect(result).toEqual(created);
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/salary-components");
    expect(call[1] && (call[1] as RequestInit).method).toBe("POST");
    expect(JSON.parse((call[1] as RequestInit).body as string)).toEqual(body);
  });

  it("createSalaryComponent: ApiError on 422 validation", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(422, "Unprocessable", { error_code: "VALIDATION_FAILED" }));
    await expect(createSalaryComponent({} as never)).rejects.toThrow();
  });

  it("fetchIncentivePolicies: happy path", async () => {
    const mock = [{ id: "ip-1", name: "Q1", payout_type: "fixed" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchIncentivePolicies();
    expect(result).toEqual(mock);
  });

  it("createIncentivePolicy: POST forwards body", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await createIncentivePolicy({
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
    await cloneIncentivePolicy("policy-1");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/payroll/incentive-policies/policy-1/clone");
  });

  it("deleteIncentivePolicy: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deleteIncentivePolicy("policy-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("DELETE");
  });

  it("fetchEmployeeSalaryComponents: happy path", async () => {
    const mock = [{ id: "esc-1", amount: "100" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchEmployeeSalaryComponents("emp-1");
    expect(result).toEqual(mock);
  });

  it("assignEmployeeSalaryComponent: POST returns record", async () => {
    const mock = { id: "esc-new", amount: "500" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await assignEmployeeSalaryComponent("emp-1", {
      amount: "500",
      component_definition_id: "cd-1",
      effective_date: "2026-01-01",
    });
    expect(result).toEqual(mock);
  });

  it("calculate_payroll: POST with query year/month", async () => {
    const mock = { id: "ps-1", gross: 1000, net: 900 };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await calculate_payroll("emp-1", 2026, 3);
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/employees/emp-1/calculate-payroll");
  });

  it("calculate_payroll: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(calculate_payroll("emp-1", 2026, 3)).rejects.toThrow();
  });

  it("fetchEmployeePayslips: happy path", async () => {
    const mock = [{ id: "p-1", net: 900 }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchEmployeePayslips("emp-1");
    expect(result).toEqual(mock);
  });

  it("fetchOvertimePolicies: returns data array", async () => {
    const mock = [{ id: "op-1", name: "OT-Weekday" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ data: mock }));
    const result = await fetchOvertimePolicies();
    expect(result).toEqual(mock);
  });

  it("fetchOvertimePolicy: returns policy + versions detail", async () => {
    const detail = { policy: { id: "op-1", name: "OT" }, versions: [{ id: "ov-1", version_no: 1, status: "published" }] };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(detail));
    const result = await fetchOvertimePolicy("op-1");
    expect(result).toEqual(detail);
  });

  it("createOvertimePolicy + version + publish", async () => {
    const detail = { policy: { id: "op-new", name: "New" }, versions: [{ id: "ov-1", version_no: 1, status: "draft", row_version: 1 }] };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(detail));
    await expect(createOvertimePolicy({ name: "New", rules: [] })).resolves.toEqual(detail);

    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "ov-2", version_no: 2, status: "draft", row_version: 1 }));
    await createOvertimePolicyVersion("op-new", { rules: [{ priority: 10, day_type: "weekday", days_of_week: 127, multiplier: "1.5" }] });
    expect(vi.mocked(apiFetch).mock.calls[1][0]).toContain("/overtime/policies/op-new/versions");

    vi.mocked(apiFetch).mockResolvedValueOnce(okRes({ id: "ov-2", status: "published", row_version: 2 }));
    await publishOvertimePolicyVersion("op-new", "ov-2", 1);
    expect(vi.mocked(apiFetch).mock.calls[2][0]).toContain("/overtime/policies/op-new/versions/ov-2/publish");
  });

  it("calculateBatchPayslips: POST body with employment_ids + year/month", async () => {
    const mock = { results: [{ employee_id: "e1", net: 100 }] };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await calculateBatchPayslips({ employment_ids: ["e1"], year: 2026, month: 3 });
    expect(result).toEqual(mock);
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    const body = JSON.parse(opts.body as string);
    expect(body.employment_ids).toEqual(["e1"]);
    expect(body.year).toBe(2026);
    expect(body.month).toBe(3);
  });

  it("calculateBatchPayslips: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(calculateBatchPayslips({ employment_ids: ["e1"], year: 2026, month: 3 })).rejects.toThrow();
  });

  it("fetchPayslips: GET with query year/month", async () => {
    const mock = [{ id: "p-1", net: 900 }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchPayslips(2026, 3);
    expect(result).toEqual(mock);
  });

  it("fetchPayrollRules: GET list", async () => {
    const mock = [{ id: "pr-1", name: "Tax" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchPayrollRules();
    expect(result).toEqual(mock);
  });

  it("createPayrollRule: POST returns created", async () => {
    const mock = { id: "pr-new", name: "Tax2" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createPayrollRule({ name: "Tax2" } as never);
    expect(result).toEqual(mock);
  });

  it("fetchPayrollRuleTiers: GET list", async () => {
    const mock = [{ id: "t-1", tier_order: 1 }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchPayrollRuleTiers("pr-1");
    expect(result).toEqual(mock);
  });

  it("createPayrollRuleTier: POST returns tier", async () => {
    const mock = { id: "t-new", tier_order: 2 };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createPayrollRuleTier("pr-1", {
      min_amount: "0",
      rate: "5",
      tier_order: 2,
    });
    expect(result).toEqual(mock);
  });

  it("fetchPayrollRuleAssignments: GET list", async () => {
    const mock = [{ id: "a-1", assignee_type: "employee" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchPayrollRuleAssignments("pr-1");
    expect(result).toEqual(mock);
  });

  it("createPayrollRuleAssignment: POST returns assignment", async () => {
    const mock = { id: "a-new", assignee_type: "employee" };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createPayrollRuleAssignment("pr-1", {
      assignee_type: "employee",
      effective_from: "2026-01-01",
    } as never);
    expect(result).toEqual(mock);
  });

  it("deletePayrollRuleAssignment: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deletePayrollRuleAssignment("pr-1", "a-1");
    const opts = vi.mocked(apiFetch).mock.calls[0][1] as RequestInit;
    expect(opts.method).toBe("DELETE");
  });
});
