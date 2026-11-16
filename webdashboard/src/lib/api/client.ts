import { ApiError } from "./errors";
import { apiFetch } from "./helpers";

/** Path parameters interpolated into `{name}` placeholders. */
export type PathParams = Record<string, string | number | undefined>;

/** Query parameters appended to the request URL. */
export type QueryParams = Record<string, unknown>;

export interface ClientRequestInit {
  body?: unknown;
  headers?: Record<string, string>;
  params?: { path?: PathParams; query?: QueryParams };
}

export interface ClientResponse<T> {
  data: T;
  response: Response;
}

const QUERY_OMITTED = new Set<unknown>([undefined, null, "all"]);

function formatQueryParamValue(val: unknown): string {
  if (typeof val === "string") return val;
  if (typeof val === "number" || typeof val === "boolean" || typeof val === "bigint") return String(val);
  if (val instanceof Date) return val.toISOString();
  return JSON.stringify(val);
}

function buildQueryString(query: QueryParams | undefined): string {
  if (!query) return "";
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(query)) {
    if (QUERY_OMITTED.has(value)) continue;
    if (Array.isArray(value)) {
      for (const item of value) {
        if (QUERY_OMITTED.has(item)) continue;
        search.append(key, formatQueryParamValue(item));
      }
      continue;
    }
    if (value !== null && typeof value === "object") {
      search.set(key, JSON.stringify(value));
      continue;
    }
    search.set(key, formatQueryParamValue(value));
  }
  const serialized = search.toString();
  return serialized ? `?${serialized}` : "";
}

function buildRequestOptions(method: string, init?: ClientRequestInit): RequestInit {
  const options: RequestInit = { method };
  if (init?.headers) options.headers = init.headers;
  if (init?.body === undefined || !["POST", "PUT", "PATCH"].includes(method)) return options;

  if (init.body instanceof FormData || init.body instanceof Blob) {
    options.body = init.body;
    return options;
  }

  options.headers = { "Content-Type": "application/json", ...(init.headers ?? {}) };
  options.body = JSON.stringify(init.body);
  return options;
}

async function parseResponseData<T>(method: string, response: Response): Promise<T> {
  if (method === "DELETE" && response.status === 204) return undefined as T;

  const contentType = response.headers.get("content-type") ?? "";
  return (contentType.includes("application/json")
    ? await response.json()
    : await response.text()) as T;
}

export function buildUrl(path: string, params?: { path?: PathParams; query?: QueryParams }): string {
  const interpolated = path.replace(/\{([^{}]+)\}/g, (_match, name: string) => {
    const value = params?.path?.[name];
    if (value === undefined) {
      throw new TypeError(`Missing path parameter: ${name}`);
    }
    return encodeURIComponent(String(value));
  });
  return `${interpolated}${buildQueryString(params?.query)}`;
}


async function request<T = unknown>(
  method: string,
  path: string,
  init?: ClientRequestInit,
): Promise<ClientResponse<T>> {
  const response = await apiFetch(buildUrl(path, init?.params), buildRequestOptions(method, init));
  if (!response.ok) await ApiError.rejectResponse(response);

  return { data: await parseResponseData<T>(method, response), response };
}

export const client = {
  GET: <T = unknown>(path: string, init?: ClientRequestInit) => request<T>("GET", path, init),
  POST: <T = unknown>(path: string, init?: ClientRequestInit) => request<T>("POST", path, init),
  PUT: <T = unknown>(path: string, init?: ClientRequestInit) => request<T>("PUT", path, init),
  PATCH: <T = unknown>(path: string, init?: ClientRequestInit) => request<T>("PATCH", path, init),
  DELETE: <T = unknown>(path: string, init?: ClientRequestInit) => request<T>("DELETE", path, init),
};

export type HttpMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH";

/**
 * Escape hatch for endpoints whose response shape is not described by a
 * generated schema. Callers assert `T`; the backend DTO is the source of truth.
 */
// eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters -- T is public response type for schema-drift endpoints.
export async function fetch<T>(
  method: HttpMethod,
  path: string,
  init?: ClientRequestInit,
): Promise<{ data: T }> {
  const { data } = await request<T>(method, path, init);
  return { data };
}

/** Fire-and-forget variant for mutations whose response body is discarded. */
export async function fire(
  method: HttpMethod,
  path: string,
  init?: ClientRequestInit,
): Promise<void> {
  await request(method, path, init);
}

export { ApiError };
