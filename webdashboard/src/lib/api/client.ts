import { ApiError, AppErrorResponse } from "./errors";
import { apiFetch } from "./helpers";

async function handleResponse<T>(res: Response): Promise<{ data: T }> {
  if (!res.ok) {
    let errorBody: AppErrorResponse | undefined;
    let errorText = "";
    
    try {
      errorText = await res.text();
      if (errorText) {
        errorBody = JSON.parse(errorText) as AppErrorResponse;
      }
    } catch {
      // Ignore parsing errors, keep raw text or undefined body
    }

    throw new ApiError(
      res.status,
      res.statusText,
      errorBody?.error_code,
      errorBody || (errorText ? { message: errorText } : undefined)
    );
  }

  const text = await res.text();
  return { data: (text ? JSON.parse(text) : null) as T };
}

/**
 * A typed client wrapper around `apiFetch` that handles JSON parsing
 * and throws structured `ApiError` instances on failure.
 */
export const apiClient = {
  get: async <T = any>(url: string): Promise<{ data: T }> => {
    const res = await apiFetch(url);
    return handleResponse<T>(res);
  },
  
  post: async <T = any>(url: string, body?: any): Promise<{ data: T }> => {
    const res = await apiFetch(url, {
      method: "POST",
      body: body ? JSON.stringify(body) : undefined,
    });
    return handleResponse<T>(res);
  },
  
  put: async <T = any>(url: string, body?: any): Promise<{ data: T }> => {
    const res = await apiFetch(url, {
      method: "PUT",
      body: body ? JSON.stringify(body) : undefined,
    });
    return handleResponse<T>(res);
  },
  
  delete: async <T = any>(url: string): Promise<{ data: T }> => {
    const res = await apiFetch(url, { method: "DELETE" });
    return handleResponse<T>(res);
  },
  
  patch: async <T = any>(url: string, body?: any): Promise<{ data: T }> => {
    const res = await apiFetch(url, {
      method: "PATCH",
      body: body ? JSON.stringify(body) : undefined,
    });
    return handleResponse<T>(res);
  }
};
