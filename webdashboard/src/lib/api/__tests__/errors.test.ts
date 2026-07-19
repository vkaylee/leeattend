import { describe, it, expect } from "vitest";
import { parseConflictRetry, ConflictRetryHint } from "../errors";

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

  it("returns undefined when body has no error property", () => {
    expect(parseConflictRetry({ message: "Conflict" })).toBeUndefined();
  });

  it("returns undefined when error is not an object", () => {
    expect(parseConflictRetry({ error: "something" })).toBeUndefined();
  });

  it("returns undefined when error has no retry property", () => {
    expect(parseConflictRetry({ error: { message: "duplicate" } })).toBeUndefined();
  });

  it("returns undefined when retry is not an object", () => {
    expect(parseConflictRetry({ error: { retry: "later" } })).toBeUndefined();
  });

  it("returns undefined when retry reason is not retired_leave_type", () => {
    expect(
      parseConflictRetry({
        error: {
          retry: { reason: "other", leave_type_id: "abc-123", name: "Test" },
        },
      })
    ).toBeUndefined();
  });

  it("returns undefined when leave_type_id is missing", () => {
    expect(
      parseConflictRetry({
        error: { retry: { reason: "retired_leave_type", name: "Test" } },
      })
    ).toBeUndefined();
  });

  it("returns undefined when leave_type_id is not a string", () => {
    expect(
      parseConflictRetry({
        error: { retry: { reason: "retired_leave_type", leave_type_id: 123, name: "Test" } },
      })
    ).toBeUndefined();
  });

  it("parses a valid conflict retry payload with all fields", () => {
    const result = parseConflictRetry({
      error: {
        retry: {
          reason: "retired_leave_type",
          leave_type_id: "550e8400-e29b-41d4-a716-446655440000",
          name: "Annual Leave",
        },
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
      error: {
        retry: {
          reason: "retired_leave_type",
          leave_type_id: "abc-def",
        },
      },
    });
    expect(result).toEqual<ConflictRetryHint>({
      reason: "retired_leave_type",
      leave_type_id: "abc-def",
      name: "",
    });
  });

  it("handles a realistic 409 Conflict backend response", () => {
    // This matches what the backend sends when a retired leave-type code is reused.
    const body = JSON.parse(
      '{"error":{"retry":{"reason":"retired_leave_type","leave_type_id":"ltype-001","name":"Sick Leave"}}}'
    ) as unknown;
    const result = parseConflictRetry(body);
    expect(result).toEqual({
      reason: "retired_leave_type",
      leave_type_id: "ltype-001",
      name: "Sick Leave",
    });
  });
});
