export interface AppErrorResponse {
  [key: string]: unknown;
  error_code?: string;
  field?: string;
  message?: string;
}

import { maybeNull } from "@/types/api-overrides";

/** A single compliance violation returned by the backend on 409 Conflict. */
export interface ComplianceViolation {
  details: {
    employment_id: string;
    shift_ids: string[];
    conflicting_dates: string[];
    total_hours?: number;
    limit_hours?: number;
  };
  message: string;
  severity: "hard" | "soft";
  violation_type: "overlap" | "rest_period" | "weekly_hours" | "monthly_ot";
}

/** Structured retry payload for compliance violations. */
export interface ComplianceConflictRetry {
  requires_override: boolean;
  violations: ComplianceViolation[];
}

/** Narrow an unknown error body to check for a compliance-conflict retry payload.
 *  Handles both the raw backend shape ({ error: { retry } }) and the unwrapped
 *  shape ({ retry }) produced by ApiError.rejectResponse. */
export function parseComplianceConflict(body: unknown): ComplianceConflictRetry | undefined {
  if (typeof body !== "object" || body === null) return undefined;
  const b = body as { retry?: Partial<ComplianceConflictRetry>; error?: { retry?: Partial<ComplianceConflictRetry> } };
  // Try unwrapped shape first (produced by rejectResponse), then raw backend shape
  const retryCandidate = b.retry ?? b.error?.retry;
  const candidate = maybeNull(retryCandidate);
  if (typeof candidate !== "object" || candidate === null) return undefined;
  if (!Array.isArray(candidate.violations) || candidate.requires_override !== true) return undefined;
  return validateComplianceViolations(candidate.violations);
}

function validateComplianceViolations(raw: unknown[]): ComplianceConflictRetry | undefined {
  const violations: ComplianceViolation[] = [];
  for (const v of raw) {
    if (typeof v !== "object" || v === null) return undefined;
    const violation = v as Partial<ComplianceViolation>;
    if (typeof violation.violation_type !== "string" || typeof violation.message !== "string") return undefined;
    violations.push(violation as ComplianceViolation);
  }
  if (violations.length === 0) return undefined;
  return { violations, requires_override: true };
}

/** Structured retry hint returned on 409 Conflict for retired resources. */
export interface ConflictRetryHint {
  leave_type_id: string;
  name: string;
  reason: string;
}

/** Narrow an unknown error body to check for a conflict-retry payload.
 *  `errorBody` is always the unwrapped shape produced by ApiError.rejectResponse,
 *  where `retry` sits at the root (not nested under `error`). */
export function parseConflictRetry(body: unknown): ConflictRetryHint | undefined {
  if (typeof body !== "object" || body === null) return undefined;
  const b = body as { retry?: Partial<ConflictRetryHint> };
  const retVal: Partial<ConflictRetryHint> | null | undefined = b.retry;
  const hint = maybeNull(retVal);
  if (typeof hint !== "object" || hint === null) return undefined;
  if (hint.reason !== "retired_leave_type" || typeof hint.leave_type_id !== "string") return undefined;
  return {
    reason: hint.reason,
    leave_type_id: hint.leave_type_id,
    name: typeof hint.name === "string" ? hint.name : "",
  };
}

/**
 * Structured error class for API responses.
 * Provides properties for status codes, error codes, and the parsed response body.
 */
export class ApiError extends Error {
  constructor(
    public status: number,
    public statusText: string,
    public errorCode?: string,
    public errorBody?: AppErrorResponse,
  ) {
    super(`[${status}] ${errorCode ?? statusText}${errorBody?.message ? `: ${errorBody.message}` : ''}`);
    this.name = "ApiError";

    // Set prototype explicitly for built-in Error subclassing in TS/JS
    Object.setPrototypeOf(this, ApiError.prototype);
  }

  /**
   * Parse a non-OK fetch Response and throw an ApiError with the
   * server-provided error code and message (if any).
   *
   * Extracted so it can be shared between the openapi-fetch customFetch
   * wrapper and hand-written apiFetch call sites.
   */
  static async rejectResponse(res: Response): Promise<never> {
    let errorBody: AppErrorResponse | undefined;
    let errorText = "";
    try {
      errorText = await res.clone().text();
      if (errorText) {
        const raw = JSON.parse(errorText) as Record<string, unknown>;
        // Backend wraps errors as { error: { code, message } } — unwrap to
        // the flat AppErrorResponse shape the rest of the codebase expects.
        if (typeof raw.error === "object" && raw.error !== null) {
          const inner = raw.error as Record<string, unknown>;
          errorBody = {
            error_code: typeof inner.code === "string" ? inner.code : undefined,
            message: typeof inner.message === "string" ? inner.message : undefined,
            retry: inner.retry,
          };
        } else {
          errorBody = raw;
        }
      }
    } catch { /* JSON.parse failure on response body — fall through to generic error */
    }
    throw new ApiError(
      res.status,
      res.statusText,
      errorBody?.error_code,
      errorBody ?? (errorText ? { message: errorText } : undefined),
    );
  }

  get isNotFound() {
    return this.status === 404;
  }
  
  get isValidation() {
    return this.status === 400 || this.status === 422;
  }
  
  get isUnauthorized() {
    return this.status === 401;
  }
  
  get isConflict() {
    return this.status === 409;
  }
}

/**
 * Thrown when a 409 Conflict response carries a compliance violation retry
 * payload ({@link ComplianceConflictRetry}). Callers can inspect {@link retry}
 * or {@link violations} to surface the specific issues to the user before
 * retrying with allow_warnings + override_reason.
 */
export class ComplianceConflictError extends ApiError {
  public readonly retry: ComplianceConflictRetry;

  constructor(
    status: number,
    statusText: string,
    errorBody: AppErrorResponse | undefined,
    retry: ComplianceConflictRetry,
  ) {
    super(status, statusText, errorBody?.error_code, errorBody);
    this.name = "ComplianceConflictError";
    Object.setPrototypeOf(this, ComplianceConflictError.prototype);
    this.retry = retry;
  }

  get violations(): ComplianceViolation[] {
    return this.retry.violations;
  }
}
