import type { Client, MaybeOptionalInit, MethodResponse } from "openapi-fetch";
import type { PathsWithMethod, RequiredKeysOf } from "openapi-typescript-helpers";

import createClient from "openapi-fetch";

import type { paths } from "@/types/api-generated";

import { API_BASE_URL } from "./config";
import { ApiError } from "./errors";
import { apiFetch } from "./helpers";

// eslint-disable-next-line complexity
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
  // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
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
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
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

interface SafeClient {
  DELETE: SafeMethod<"DELETE">;
  GET: SafeMethod<"GET">;
  PATCH: SafeMethod<"PATCH">;
  POST: SafeMethod<"POST">;
  PUT: SafeMethod<"PUT">;
}

export const client = createClient<paths>({
  baseUrl: API_BASE_URL,
  fetch: customFetch,
}) as unknown as SafeClient;

// ── Legacy escape hatch for paths not yet in the generated schema ──────────
// Migrate callers to typed `client.GET()` / `client.POST()` whenever the
// path becomes available in the OpenAPI-generated types.
// See: https://github.com/lee-lee/leeattend/issues/type-safety-migration

type HttpMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH";

interface LegacyFetchInit {
  body?: unknown;
  params?: { path?: Record<string, unknown>; query?: Record<string, unknown> };
}

/**
 * Typed escape hatch for API paths absent from the generated schema.
 *
 * Prefer {@link client}.GET / POST / etc. for any path covered by the
 * OpenAPI spec.  This helper is intentionally generic over `T` because the
 * response shape is unknown to the type-checker — **callers must ensure `T`
 * matches the actual backend response**.
 */
// eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters -- T encodes the caller-expected response shape for paths absent from the generated schema
export async function fetch<T>(
  method: HttpMethod,
  path: string,
  init?: LegacyFetchInit,
): Promise<{ data: T }> {
  // The cast through SafeClient's method overloads is necessary because
  // `path` is a raw string (not validated against the schema).  We keep
  // the escape hatch narrow by only allowing valid HTTP methods and a
  // controlled init shape.
  const typedClient = client as Record<HttpMethod, (p: string, i?: LegacyFetchInit) => Promise<{ data: T }>>;
  return typedClient[method](path, init);
}

/**
 * Fire-and-forget variant for mutations whose response body is discarded.
 *
 * Prefer `client.POST(...)` / `client.PUT(...)` for schema-covered paths.
 */
export async function fire(
  method: HttpMethod,
  path: string,
  init?: LegacyFetchInit,
): Promise<void> {
  await fetch<undefined>(method, path, init);
}

