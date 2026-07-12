import createClient from "openapi-fetch";

import type { paths } from "@/types/api-generated";
import type { Client, FetchResponse, MaybeOptionalInit, MethodResponse } from "openapi-fetch";
import type { MediaType, PathsWithMethod, RequiredKeysOf } from "openapi-typescript-helpers";

import { API_BASE_URL } from "./config";
import { ApiError } from "./errors";
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
  return ApiError.rejectResponse(res);
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

type ClientInitParam<Init> =
  RequiredKeysOf<Init> extends never
    ? [(Init & Record<string, unknown>)?]
    : [Init & Record<string, unknown>];

/**
 * SafeMethod re-uses upstream generic constraints so `url` and `init` are
 * validated exactly as before, overrides the return so `data` is never `| undefined`.
 *
 * `MethodResponse` from openapi-fetch unwraps `NonNullable<FetchResponse["data"]>`,
 * the success branch's data type without `| undefined`.
 *
 * customFetch converts non-2xx to thrown ApiError, so `data` is always present
 * when the promise resolves.
 */
type SafeMethod<M extends "GET" | "POST" | "PUT" | "PATCH" | "DELETE"> = <
  P extends PathsWithMethod<paths, Lowercase<M>>,
  Init extends MaybeOptionalInit<paths[P], Lowercase<M>>,
>(
  url: P,
  ...init: ClientInitParam<Init>
) => Promise<{
  data: MethodResponse<Client<paths>, Lowercase<M>, P, Init>;
  error?: unknown;
  response: Response;
}>;

type SafeClient = {
  GET: SafeMethod<"GET">;
  POST: SafeMethod<"POST">;
  PUT: SafeMethod<"PUT">;
  PATCH: SafeMethod<"PATCH">;
  DELETE: SafeMethod<"DELETE">;
};

export const client = createClient<paths>({
  baseUrl: API_BASE_URL,
  fetch: customFetch,
}) as unknown as SafeClient;

// @deprecated — DO NOT use `fetch<T>()` or `fire()` for new code.
// Use the typed `client.GET()`, `client.POST()`, etc. instead.
// These helpers bypass openapi-fetch type checking and exist only for
// legacy call sites whose paths are absent from the generated schema.
// See: https://github.com/lee-lee/leeattend/issues/type-safety-migration
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

