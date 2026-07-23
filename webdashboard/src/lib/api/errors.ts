export interface AppErrorResponse {
  [key: string]: unknown;
  error_code?: string;
  field?: string;
  message?: string;
}

/** Structured retry hint returned on 409 Conflict for retired resources. */
export interface ConflictRetryHint {
  leave_type_id: string;
  name: string;
  reason: string;
}

/** Narrow an unknown error body to check for a conflict-retry payload. */
export function parseConflictRetry(body: unknown): ConflictRetryHint | undefined {
  // Step through the nested error shape with runtime type checks at each level.
  // The API returns { error: { retry: { reason, leave_type_id, name } } } on
  // 409 Conflict when a retired resource can be restored.
  if (typeof body !== "object" || body === null) return undefined;
  const b = body as { error?: { retry?: Partial<ConflictRetryHint> } };
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- defensive runtime narrowing on cast object
  if (typeof b.error !== "object" || b.error === null) return undefined;
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- defensive runtime narrowing on cast object
  if (typeof b.error.retry !== "object" || b.error.retry === null) return undefined;
  if (b.error.retry.reason !== "retired_leave_type" || typeof b.error.retry.leave_type_id !== "string") return undefined;
  return {
    reason: b.error.retry.reason,
    leave_type_id: b.error.retry.leave_type_id,
    name: typeof b.error.retry.name === "string" ? b.error.retry.name : "",
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
        errorBody = JSON.parse(errorText) as AppErrorResponse;
      }
    } catch {
      // Ignore parse failures — fall through to the generic error below.
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
