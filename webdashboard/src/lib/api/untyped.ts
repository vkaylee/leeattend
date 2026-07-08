import type { paths } from "@/types/api-generated";

import { client } from "./client";

// Paths not yet registered in the generated OpenAPI typed `paths` map fall
// outside the typed `createClient<paths>` contract. Rather than cast
// `(client as any)` at every call site — which spams eslint and loses type
// information — we expose a single untyped accessor here whose response is
// generic-typed at the call site.
interface FetchParams { path?: Record<string, unknown>; query?: Record<string, unknown> }

interface UntypedFetcher {
  // Single-use type parameter here is intentional: T is given at call time
  // to assert the data shape. Disabling the rule at the site is clearer
  // than a module-level suppression.
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters
  DELETE: <T>(path: string, init?: { params?: FetchParams }) => Promise<{ data: T }>;
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters
  GET: <T>(path: string, init?: { params?: FetchParams }) => Promise<{ data: T }>;
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters
  POST: <T>(path: string, init?: { params?: FetchParams; body?: Record<string, unknown> | FormData }) => Promise<{ data: T }>;
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters
  PUT: <T>(path: string, init?: { params?: FetchParams; body?: Record<string, unknown> | FormData }) => Promise<{ data: T }>;
}

// Dynamic paths live outside the generated OpenAPI schema, so we cannot go
// through the typed `createClient<paths>` client. A single `as any` cast at
// this boundary keeps the rest of the API layer fully typed without
// spreading the cast (or per-call type assertions) across every API module.
// eslint-disable-next-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-assignment
export const http: UntypedFetcher = client as any;

// Re-export so tree-shaking doesn't strip `paths` if some consumers still use
// `client.METHOD('/typed-path', ...)` for paths that ARE in the schema.
export type { paths };
