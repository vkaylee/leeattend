import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

/**
 * Parse a Cloudflare Pages _headers file into structured form.
 *
 * Format:
 *   /path
 *     Header-Name: value
 *     Header-Name: value
 *
 * Blank lines and lines starting with # (comments) are ignored.
 */
function parseHeaders(raw: string): Map<string, Map<string, string[]>> {
  const rules = new Map<string, Map<string, string[]>>();
  const lines = raw.split("\n");
  let currentPath: string | null = null;

  for (const line of lines) {
    const trimmed = line.trim();
    // skip empty lines and comments
    if (trimmed === "" || trimmed.startsWith("#")) continue;

    // path lines don't start with a header name (no colon, or colon is part of wildcard)
    if (!trimmed.includes(":")) {
      currentPath = trimmed;
      if (!rules.has(currentPath)) {
        rules.set(currentPath, new Map());
      }
      continue;
    }

    // header line under current path
    const colonIdx = trimmed.indexOf(":");
    const name = trimmed.slice(0, colonIdx).trim();
    const value = trimmed.slice(colonIdx + 1).trim();

    if (!currentPath) {
      throw new Error(`Header line without a path: ${trimmed}`);
    }

    const pathHeaders = rules.get(currentPath)!;
    if (!pathHeaders.has(name)) {
      pathHeaders.set(name, []);
    }
    pathHeaders.get(name)!.push(value);
  }

  return rules;
}

describe("Cloudflare Pages _headers", () => {
  // Read the source of truth: public/_headers
  const filePath = resolve(__dirname, "../../public/_headers");
  const raw = readFileSync(filePath, "utf-8");
  const rules = parseHeaders(raw);

  it("should set no-cache on / (root path) so Cloudflare edge never caches the SPA entry point", () => {
    const rootHeaders = rules.get("/");
    expect(rootHeaders, 'Missing "/" path rule in _headers').toBeDefined();

    const cacheControl = rootHeaders!.get("Cache-Control");
    expect(cacheControl, 'Missing Cache-Control header for "/"').toBeDefined();

    // Order matters: last value wins in Cloudflare. Must-revalidate ensures
    // the edge always checks origin before serving from cache.
    const joined = cacheControl!.join(", ");
    expect(joined).toContain("no-cache");
    expect(joined).toContain("must-revalidate");
    expect(joined).toContain("private");
  });

  it("should set no-cache on /index.html for direct requests", () => {
    const indexHeaders = rules.get("/index.html");
    expect(indexHeaders, 'Missing "/index.html" path rule in _headers').toBeDefined();

    const cacheControl = indexHeaders!.get("Cache-Control");
    expect(cacheControl, 'Missing Cache-Control header for "/index.html"').toBeDefined();

    const joined = cacheControl!.join(", ");
    expect(joined).toContain("no-cache");
    expect(joined).toContain("must-revalidate");
    expect(joined).toContain("private");
  });

  it("should set immutable long-lived cache on /assets/* for hashed build artifacts", () => {
    const assetHeaders = rules.get("/assets/*");
    expect(assetHeaders, 'Missing "/assets/*" path rule in _headers').toBeDefined();

    const cacheControl = assetHeaders!.get("Cache-Control");
    expect(cacheControl, 'Missing Cache-Control header for "/assets/*"').toBeDefined();

    const joined = cacheControl!.join(", ");
    expect(joined).toContain("public");
    expect(joined).toContain("max-age=31536000");
    expect(joined).toContain("immutable");
  });

  it("should set security headers on /* for all paths", () => {
    const wildcard = rules.get("/*");
    expect(wildcard, 'Missing "/*" path rule in _headers').toBeDefined();

    expect(wildcard!.get("X-Frame-Options")).toEqual(["SAMEORIGIN"]);
    expect(wildcard!.get("X-Content-Type-Options")).toEqual(["nosniff"]);
    expect(wildcard!.get("Referrer-Policy")).toEqual(["strict-origin-when-cross-origin"]);
  });

  it("should NOT set Cache-Control on /* (would override path-specific rules)", () => {
    const wildcard = rules.get("/*");
    expect(wildcard, 'Missing "/*" path rule in _headers').toBeDefined();

    const cacheControl = wildcard!.get("Cache-Control");
    expect(cacheControl, '/* should NOT have Cache-Control (path-specific rules handle it)').toBeUndefined();
  });

  it("should have / and /index.html with matching Cache-Control policies", () => {
    const rootHeaders = rules.get("/");
    const indexHeaders = rules.get("/index.html");

    expect(rootHeaders).toBeDefined();
    expect(indexHeaders).toBeDefined();

    const rootCC = rootHeaders!.get("Cache-Control");
    const indexCC = indexHeaders!.get("Cache-Control");

    // Both must agree on no-cache — otherwise one path could serve stale HTML
    expect(rootCC).toEqual(indexCC);
  });
});
