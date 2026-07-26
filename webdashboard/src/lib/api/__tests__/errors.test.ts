import { describe, it, expect } from "vitest";

import {
  ApiError,
  parseConflictRetry,
  parseComplianceConflict,
  ComplianceConflictError,
  type ConflictRetryHint,
  type ComplianceConflictRetry,
  type ComplianceViolation,
} from "../errors";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Create a minimal mock Response with JSON body. */
function mockResponse(body: unknown, status = 400, statusText = "Bad Request"): Response {
  return new Response(JSON.stringify(body), { status, statusText });
}

/** Minimal valid compliance violation for parseComplianceConflict tests. */
function validViolation(overrides?: Partial<ComplianceViolation>): ComplianceViolation {
  return {
    details: {
      employment_id: "emp-1",
      shift_ids: ["shift-1"],
      conflicting_dates: ["2026-07-01"],
    },
    message: "Overlap detected",
    severity: "hard",
    violation_type: "overlap",
    ...overrides,
  };
}

// ---------------------------------------------------------------------------
// parseConflictRetry
// ---------------------------------------------------------------------------

describe("parseConflictRetry", () => {
  it("returns undefined for null input", () => {
    expect(parseConflictRetry(null)).toBeUndefined();
  });

  it("returns undefined for undefined input", () => {
    expect(parseConflictRetry(undefined)).toBeUndefined();
  });

  it("returns undefined for string input", () => {
    expect(parseConflictRetry("error")).toBeUndefined();
  });

  it("returns undefined for empty object", () => {
    expect(parseConflictRetry({})).toBeUndefined();
  });

  it("returns undefined when body has no retry property", () => {
    expect(parseConflictRetry({ message: "Conflict" })).toBeUndefined();
  });

  it("returns undefined when retry is not an object", () => {
    expect(parseConflictRetry({ retry: "later" })).toBeUndefined();
  });

  it("returns undefined when retry reason is not retired_leave_type", () => {
    expect(
      parseConflictRetry({
        retry: { reason: "other", leave_type_id: "abc-123", name: "Test" },
      })
    ).toBeUndefined();
  });

  it("returns undefined when leave_type_id is missing", () => {
    expect(
      parseConflictRetry({
        retry: { reason: "retired_leave_type", name: "Test" },
      })
    ).toBeUndefined();
  });

  it("returns undefined when leave_type_id is not a string", () => {
    expect(
      parseConflictRetry({
        retry: { reason: "retired_leave_type", leave_type_id: 123, name: "Test" },
      })
    ).toBeUndefined();
  });

  it("parses a valid conflict retry payload with all fields", () => {
    const result = parseConflictRetry({
      retry: {
        reason: "retired_leave_type",
        leave_type_id: "550e8400-e29b-41d4-a716-446655440000",
        name: "Annual Leave",
      },
    });
    expect(result).toEqual<ConflictRetryHint>({
      reason: "retired_leave_type",
      leave_type_id: "550e8400-e29b-41d4-a716-446655440000",
      name: "Annual Leave",
    });
  });

  it("parses a valid payload when name is missing (defaults to empty string)", () => {
    const result = parseConflictRetry({
      retry: {
        reason: "retired_leave_type",
        leave_type_id: "abc-def",
      },
    });
    expect(result).toEqual<ConflictRetryHint>({
      reason: "retired_leave_type",
      leave_type_id: "abc-def",
      name: "",
    });
  });

  it("handles the unwrapped error body produced by ApiError.rejectResponse", () => {
    const body = {
      error_code: "CONFLICT",
      message: "Leave type already exists",
      retry: { reason: "retired_leave_type", leave_type_id: "ltype-001", name: "Sick Leave" },
    } as unknown;
    const result = parseConflictRetry(body);
    expect(result).toEqual({
      reason: "retired_leave_type",
      leave_type_id: "ltype-001",
      name: "Sick Leave",
    });
  });
});

// ---------------------------------------------------------------------------
// ApiError.rejectResponse – the core fix
// ---------------------------------------------------------------------------

describe("ApiError.rejectResponse", () => {
  it("unwraps backend-style { error: { code, message } } into flat AppErrorResponse", async () => {
    const res = mockResponse(
      { error: { code: "VALIDATION_ERROR", message: "Email is required" } },
      422,
      "Unprocessable Entity",
    );

    let caught: ApiError | undefined;
    try {
      await ApiError.rejectResponse(res);
    } catch (e) {
      caught = e as ApiError;
    }

    expect(caught).toBeInstanceOf(ApiError);
    expect(caught!.status).toBe(422);
    expect(caught!.errorCode).toBe("VALIDATION_ERROR");
    expect(caught!.errorBody?.message).toBe("Email is required");
    expect(caught!.message).toContain("Email is required");
  });

  it("unwraps 500 backend error so message is shown, not swallowed", async () => {
    const res = mockResponse(
      { error: { code: "DATABASE_ERROR", message: "Connection pool exhausted" } },
      500,
      "Internal Server Error",
    );

    let caught: ApiError | undefined;
    try {
      await ApiError.rejectResponse(res);
    } catch (e) {
      caught = e as ApiError;
    }

    expect(caught!.status).toBe(500);
    expect(caught!.errorCode).toBe("DATABASE_ERROR");
    expect(caught!.errorBody?.message).toBe("Connection pool exhausted");
    expect(caught!.message).toContain("Connection pool exhausted");
  });

  it("preserves retry payload when backend sends { error: { code, message, retry } }", async () => {
    const retryPayload = { reason: "retired_leave_type", leave_type_id: "lt-1", name: "Sick" };
    const res = mockResponse(
      { error: { code: "CONFLICT", message: "Duplicate", retry: retryPayload } },
      409,
      "Conflict",
    );

    let caught: ApiError | undefined;
    try {
      await ApiError.rejectResponse(res);
    } catch (e) {
      caught = e as ApiError;
    }

    expect(caught!.errorBody?.retry).toEqual(retryPayload);
  });

  it("passes through flat JSON body unchanged (backward compat)", async () => {
    const flat = { message: "Something went wrong", error_code: "CUSTOM" };
    const res = mockResponse(flat, 400, "Bad Request");

    let caught: ApiError | undefined;
    try {
      await ApiError.rejectResponse(res);
    } catch (e) {
      caught = e as ApiError;
    }

    expect(caught!.errorBody?.message).toBe("Something went wrong");
    expect(caught!.errorCode).toBe("CUSTOM");
  });

  it("falls back to raw text when body is not JSON", async () => {
    const res = new Response("Plain text error", { status: 502, statusText: "Bad Gateway" });

    let caught: ApiError | undefined;
    try {
      await ApiError.rejectResponse(res);
    } catch (e) {
      caught = e as ApiError;
    }

    expect(caught!.errorBody?.message).toBe("Plain text error");
    expect(caught!.errorCode).toBeUndefined();
  });

  it("handles empty response body gracefully", async () => {
    const res = new Response("", { status: 500, statusText: "Internal Server Error" });

    let caught: ApiError | undefined;
    try {
      await ApiError.rejectResponse(res);
    } catch (e) {
      caught = e as ApiError;
    }

    expect(caught).toBeInstanceOf(ApiError);
    expect(caught!.status).toBe(500);
    expect(caught!.errorBody).toBeUndefined();
  });

  it("sets the Error.message to include status, error code, and server message", async () => {
    const res = mockResponse(
      { error: { code: "FORBIDDEN", message: "Insufficient permissions" } },
      403,
      "Forbidden",
    );

    let caught: ApiError | undefined;
    try {
      await ApiError.rejectResponse(res);
    } catch (e) {
      caught = e as ApiError;
    }

    expect(caught!.message).toBe("[403] FORBIDDEN: Insufficient permissions");
  });
});

// ---------------------------------------------------------------------------
// parseComplianceConflict – both raw & unwrapped shapes
// ---------------------------------------------------------------------------

describe("parseComplianceConflict", () => {
  it("returns undefined for null", () => {
    expect(parseComplianceConflict(null)).toBeUndefined();
  });

  it("returns undefined for non-object", () => {
    expect(parseComplianceConflict("nope")).toBeUndefined();
  });

  it("returns undefined for empty object", () => {
    expect(parseComplianceConflict({})).toBeUndefined();
  });

  it("returns undefined when retry is not an object", () => {
    expect(parseComplianceConflict({ retry: 123 })).toBeUndefined();
  });

  it("returns undefined when violations is not an array", () => {
    expect(parseComplianceConflict({ retry: { requires_override: true, violations: "bad" } })).toBeUndefined();
  });

  it("returns undefined when requires_override is not true", () => {
    expect(parseComplianceConflict({ retry: { requires_override: false, violations: [validViolation()] } })).toBeUndefined();
  });

  it("returns undefined when violations array is empty", () => {
    expect(parseComplianceConflict({ retry: { requires_override: true, violations: [] } })).toBeUndefined();
  });

  it("returns undefined when a violation is missing violation_type", () => {
    expect(
      parseComplianceConflict({
        retry: { requires_override: true, violations: [{ message: "bad" }] },
      })
    ).toBeUndefined();
  });

  // ── Unwrapped shape (as produced by rejectResponse) ──────────────

  it("parses unwrapped shape { retry: { ... } } from rejectResponse", () => {
    const v = validViolation();
    const result = parseComplianceConflict({
      error_code: "CONFLICT",
      message: "Compliance check failed",
      retry: { requires_override: true, violations: [v] },
    });
    expect(result).toEqual<ComplianceConflictRetry>({
      requires_override: true,
      violations: [v],
    });
  });

  // ── Raw backend shape (as used in shifts.ts before rejectResponse) ──

  it("parses raw backend shape { error: { retry: { ... } } }", () => {
    const v = validViolation({ violation_type: "rest_period" });
    const result = parseComplianceConflict({
      error: {
        code: "CONFLICT",
        message: "Rest period violated",
        retry: { requires_override: true, violations: [v] },
      },
    });
    expect(result).toEqual<ComplianceConflictRetry>({
      requires_override: true,
      violations: [v],
    });
  });

  it("returns undefined for raw shape when error.retry is missing", () => {
    expect(parseComplianceConflict({ error: { code: "CONFLICT" } })).toBeUndefined();
  });
});

// ---------------------------------------------------------------------------
// ComplianceConflictError
// ---------------------------------------------------------------------------

describe("ComplianceConflictError", () => {
  it("is an instance of ApiError", () => {
    const v = validViolation();
    const err = new ComplianceConflictError(409, "Conflict", { message: "test" }, {
      requires_override: true,
      violations: [v],
    });
    expect(err).toBeInstanceOf(ApiError);
    expect(err).toBeInstanceOf(ComplianceConflictError);
  });

  it("exposes violations via getter", () => {
    const v1 = validViolation({ violation_type: "overlap" });
    const v2 = validViolation({ violation_type: "weekly_hours" });
    const err = new ComplianceConflictError(409, "Conflict", { message: "test" }, {
      requires_override: true,
      violations: [v1, v2],
    });
    expect(err.violations).toEqual([v1, v2]);
    expect(err.retry.requires_override).toBe(true);
  });

  it("has correct name property", () => {
    const err = new ComplianceConflictError(409, "Conflict", undefined, {
      requires_override: true,
      violations: [validViolation()],
    });
    expect(err.name).toBe("ComplianceConflictError");
  });
});
