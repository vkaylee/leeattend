import createClient from "openapi-fetch";

import type { paths } from "@/types/api-generated";

import { API_BASE_URL } from "./config";
import { ApiError, AppErrorResponse } from "./errors";
import { apiFetch } from "./helpers";

// eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters
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
      errorBody ?? (errorText ? { message: errorText } : undefined)
    );
  }

  const text = await res.text();
  return { data: (text ? JSON.parse(text) : null) as T };
}

export const apiClient = {
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters
  get: async <T = unknown>(url: string): Promise<{ data: T }> => {
    const res = await apiFetch(url);
    return handleResponse<T>(res);
  },
  
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters
  post: async <T = unknown>(url: string, body?: unknown): Promise<{ data: T }> => {
    const res = await apiFetch(url, {
      method: "POST",
      body: body ? JSON.stringify(body) : undefined,
    });
    return handleResponse<T>(res);
  },
  
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters
  put: async <T = unknown>(url: string, body?: unknown): Promise<{ data: T }> => {
    const res = await apiFetch(url, {
      method: "PUT",
      body: body ? JSON.stringify(body) : undefined,
    });
    return handleResponse<T>(res);
  },
  
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters
  delete: async <T = unknown>(url: string): Promise<{ data: T }> => {
    const res = await apiFetch(url, { method: "DELETE" });
    return handleResponse<T>(res);
  },
  
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters
  patch: async <T = unknown>(url: string, body?: unknown): Promise<{ data: T }> => {
    const res = await apiFetch(url, {
      method: "PATCH",
      body: body ? JSON.stringify(body) : undefined,
    });
    return handleResponse<T>(res);
  }
};

async function extractRequestInfo(
  url: Request,
  initOptions: RequestInit
): Promise<{ urlStr: string; options: RequestInit }> {
  const options = { ...initOptions };
  const urlStr = url.url;
  options.method = options.method ?? url.method;
  
  if (!options.headers) {
    const headers: Record<string, string> = {};
    url.headers.forEach((value, key) => {
      headers[key] = value;
    });
    options.headers = headers;
  }
  
  if (!options.body) {
    const method = options.method.toUpperCase();
    if (["POST", "PUT", "PATCH", "DELETE"].includes(method)) {
      try {
        const bodyText = await url.clone().text();
        if (bodyText) {
          options.body = bodyText;
        }
      } catch {
        // Ignore if body reading fails
      }
    }
  }
  
  return { urlStr, options };
}

export const client = createClient<paths>({
  baseUrl: API_BASE_URL,
  fetch: async (url: string | URL | Request, init?: RequestInit) => {
    let urlStr: string;
    let options: RequestInit = { ...init };

    if (url instanceof Request) {
      const extracted = await extractRequestInfo(url, options);
      urlStr = extracted.urlStr;
      options = extracted.options;
    } else {
      urlStr = url.toString();
    }

    const res = await apiFetch(urlStr, options);
    
    if (!res.ok) {
      let errorBody: AppErrorResponse | undefined;
      let errorText = "";
      
      try {
        const clonedRes = res.clone();
        errorText = await clonedRes.text();
        if (errorText) {
          errorBody = JSON.parse(errorText) as AppErrorResponse;
        }
      } catch {
        // Ignore parsing errors
      }

      throw new ApiError(
        res.status,
        res.statusText,
        errorBody?.error_code,
        errorBody ?? (errorText ? { message: errorText } : undefined)
      );
    }
    
    return res;
  }
});
