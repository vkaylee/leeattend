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
        const contentType = url.headers.get("content-type") ?? "";
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

// ── SafeClient: typed wrapper with runtime guard ────────────────────
// createClient<paths> returns FetchResponse<T> = { data } | { error },
// but customFetch always throws ApiError on non-2xx, so the error branch
// is unreachable at runtime. SafeClient encodes this invariant in the types,
// giving every call-site a clean `{ data }` destructure.
//
// Each method goes through safeWrap which adds a runtime guard: if a 2xx
// response somehow lands in the error branch (or data is missing), it
// throws immediately rather than letting undefined data propagate silently.
// See: .ai/ADRs/safe-client-wrapper.md

const _rawClient = createClient<paths>({
  baseUrl: API_BASE_URL,
  fetch: customFetch,
});

function safeWrap<M extends "GET" | "POST" | "PUT" | "PATCH" | "DELETE">(
  method: M,
): SafeMethod<M> {
  return (async (url: any, ...init: any[]) => {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call -- raw openapi-fetch method call, types erased by safeWrap generic
    const result = await (_rawClient[method] as Function)(url, ...init);
    // Runtime guard: defend against a future where openapi-fetch stops
    // respecting our customFetch throw-on-error contract and returns
    // the error branch instead of throwing.  (data MAY be undefined on
    // 204 No Content / empty-body DELETE responses — that is normal.)
    if (result.error !== undefined) {
      throw new ApiError(
        result.response?.status ?? 500,
        "SafeClient invariant violated",
        "INVALID_RESPONSE",
        { message: "API returned error branch or missing data on a response that passed customFetch" },
      );
    }
    return result;
  }) as SafeMethod<M>;
}

export const client: SafeClient = {
  DELETE: safeWrap("DELETE"),
  GET: safeWrap("GET"),
  PATCH: safeWrap("PATCH"),
  POST: safeWrap("POST"),
  PUT: safeWrap("PUT"),
};

// ── Legacy escape hatch for paths not yet in the generated schema ──────────
// Migrate callers to typed `client.GET()` / `client.POST()` whenever the
// path becomes available in the OpenAPI-generated types.
// See: https://github.com/lee-lee/leeattend/issues/type-safety-migration

type HttpMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH";

interface LegacyFetchInit {
  body?: unknown;
  params?: { path?: Record<string, string | number | undefined>; query?: Record<string, unknown> };
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

