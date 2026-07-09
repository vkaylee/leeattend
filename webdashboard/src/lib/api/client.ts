import createClient from "openapi-fetch";

import type { paths } from "@/types/api-generated";

import { API_BASE_URL } from "./config";
import { ApiError, AppErrorResponse } from "./errors";
import { apiFetch } from "./helpers";

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
        const contentType = url.headers.get("content-type") || "";
        if (contentType.includes("multipart/form-data")) {
          const formData = await url.clone().formData();
          options.body = formData;
        } else if (
          contentType.includes("application/json") ||
          contentType.includes("application/x-www-form-urlencoded") ||
          contentType.startsWith("text/")
        ) {
          const bodyText = await url.clone().text();
          if (bodyText) {
            options.body = bodyText;
          }
        } else {
          // For binary payloads (octet-stream, files, etc.)
          const bodyBlob = await url.clone().blob();
          if (bodyBlob && bodyBlob.size > 0) {
            options.body = bodyBlob;
          }
        }
      } catch {
        // Ignore if body reading fails
      }
    }
  }
  
  return { urlStr, options };
}

function cleanApiPath(urlStr: string): string {
  if (urlStr.includes('/api/api/v1')) {
    return urlStr.replace('/api/api/v1', '/api/v1');
  }
  if (urlStr.includes('/api/') && !urlStr.includes('/api/v1/')) {
    return urlStr.replace('/api/', '/api/v1/');
  }
  return urlStr;
}

async function handleResponseError(res: Response): Promise<never> {
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

export async function customFetch(url: string | URL | Request, init?: RequestInit): Promise<Response> {
  let urlStr: string;
  let options: RequestInit = { ...init };

  if (url instanceof Request) {
    const extracted = await extractRequestInfo(url, options);
    urlStr = cleanApiPath(extracted.urlStr);
    options = extracted.options;
  } else {
    urlStr = url.toString();
  }

  const res = await apiFetch(urlStr, options);
  
  if (!res.ok) {
    await handleResponseError(res);
  }
  
  return res;
}

export const client = createClient<paths>({
  baseUrl: API_BASE_URL,
  fetch: customFetch
});

// Typed wrappers for paths whose response type is a hand-written override
// (Group-B Omit/& or Group-C interface) rather than the raw generated schema.
// The single cast below is the sanctioned generic-dispatch escape hatch
// (coding-standard §10.1 `<T>` on helper = PREFERRED): `client[method]` with a
// method union yields an un-callable union of function types, so we collapse it
// to one callable signature here and let every call site stay cast-free.
// When a path's hand-written type is deleted in favour of the generated schema,
// migrate the call site to the typed `client.METHOD(...)` and drop the helper.
// Deliberately NOT called `http` — that name previously masked an `any` client.
// eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters
export async function fetch<T>(
  method: "GET" | "POST" | "PUT" | "DELETE" | "PATCH",
  path: string,
  init?: {
    params?: { path?: Record<string, unknown>; query?: Record<string, unknown> };
    body?: unknown;
  },
): Promise<{ data: T }> {
  const fn = client[method] as (
    p: string,
    i?: unknown,
  ) => Promise<{ data: T }>;
  return fn(path, init);
}

// Fire-and-forget variant for mutations whose response body is discarded.
// Avoids passing `void` as a generic type argument (banned by lint).
export async function fire(
  method: "GET" | "POST" | "PUT" | "DELETE" | "PATCH",
  path: string,
  init?: {
    params?: { path?: Record<string, unknown>; query?: Record<string, unknown> };
    body?: unknown;
  },
): Promise<void> {
  await fetch<undefined>(method, path, init);
}

