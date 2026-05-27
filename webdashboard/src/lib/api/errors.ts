export interface AppErrorResponse {
  [key: string]: unknown;
  error_code?: string;
  field?: string;
  message?: string;
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
    super(`[${status}] ${errorCode || statusText}${errorBody?.message ? `: ${errorBody.message}` : ''}`);
    this.name = "ApiError";

    // Set prototype explicitly for built-in Error subclassing in TS/JS
    Object.setPrototypeOf(this, ApiError.prototype);
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
