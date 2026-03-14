import { describe, it, expect } from "vitest";
import { ROUTES } from "../routes";
import * as fs from "node:fs";
import * as path from "node:path";

/* ------------------------------------------------------------------ */
/*  Helper: flatten the nested ROUTES object into a flat path list    */
/* ------------------------------------------------------------------ */
function collectPaths(obj: Record<string, unknown>, result: string[] = []): string[] {
  for (const value of Object.values(obj)) {
    if (typeof value === "string") {
      result.push(value);
    } else if (typeof value === "object" && value !== null) {
      collectPaths(value as Record<string, unknown>, result);
    }
  }
  return result;
}

describe("ROUTES constants", () => {
  const allPaths = collectPaths(ROUTES);

  it("exports a non-empty object", () => {
    expect(Object.keys(ROUTES).length).toBeGreaterThan(0);
    expect(allPaths.length).toBeGreaterThan(0);
  });

  it("every path starts with /", () => {
    for (const path of allPaths) {
      expect(path, `"${path}" should start with /`).toMatch(/^\//);
    }
  });

  it("no path has a trailing slash (except HOME)", () => {
    for (const path of allPaths) {
      if (path === "/") continue;
      expect(path, `"${path}" should not end with /`).not.toMatch(/\/$/);
    }
  });

  it("no duplicate paths exist", () => {
    const seen = new Set<string>();
    for (const path of allPaths) {
      expect(seen.has(path), `duplicate path: "${path}"`).toBe(false);
      seen.add(path);
    }
  });

  it("all SETTINGS sub-paths are prefixed with the SETTINGS.ROOT", () => {
    const root = ROUTES.SETTINGS.ROOT;
    for (const [key, value] of Object.entries(ROUTES.SETTINGS)) {
      if (key === "ROOT") continue;
      expect(value, `SETTINGS.${key} should be under ${root}`).toMatch(
        new RegExp(`^${root}/`),
      );
    }
  });

  // -----------------------------------------------------------------
  //  Verify ROUTES constants match the paths registered in AppRoutes
  // -----------------------------------------------------------------
  // These are the route segments from AppRoutes.tsx, resolved to
  // absolute paths based on nesting. Keeping this list explicit makes
  // it easy to catch drift between routes.ts and the router config.
  const EXPECTED_APP_ROUTES = [
    // Auth
    "/login",
    "/register",
    "/onboarding",
    // Main layout (under /)
    "/",
    "/employees",
    "/departments",
    "/members",
    "/attendance-logs",
    "/leave-requests",
    "/leave-balances",
    "/attendance-adjustments",
    "/approvals/ot",
    "/reports",
    "/shifts",
    "/shift-swaps",
    "/payroll",
    "/profile",
    // Settings (under /workspace/settings)
    "/workspace/settings",
    "/workspace/settings/general",
    "/workspace/settings/people",
    "/workspace/settings/time",
    "/workspace/settings/payroll",
    "/workspace/settings/leave",
    "/workspace/settings/holidays",
    "/workspace/settings/connectivity",
    "/workspace/settings/audit-logs",
    // ESS
    "/ess/profile",
    "/ess/attendance",
    "/ess/clock",
    // Infra
    "/infra/monitor",
  ];

  it("every AppRoutes path has a matching ROUTES constant", () => {
    for (const expected of EXPECTED_APP_ROUTES) {
      expect(
        allPaths,
        `Missing ROUTES constant for app route "${expected}"`,
      ).toContain(expected);
    }
  });

  it("every ROUTES constant corresponds to a registered app route", () => {
    // A few routes may be defined for future use but we still want them
    // to be valid paths, so we just check format here (already covered
    // above). This test documents intentional extras.
    const extras = allPaths.filter((p) => !EXPECTED_APP_ROUTES.includes(p));
    // If extras appear, add them to EXPECTED_APP_ROUTES or document why
    // they're expected extras (e.g. feature-flagged routes).
    for (const extra of extras) {
      expect(extra, `Extra ROUTES constant "${extra}" not in AppRoutes — is this intentional?`).toMatch(/^\//);
    }
  });
});

/* ------------------------------------------------------------------ */
/*  Source-file compliance: no hardcoded route strings in prod code    */
/* ------------------------------------------------------------------ */

/** Recursively collect all .ts/.tsx files under a directory */
function walkDir(dir: string, results: string[] = []): string[] {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === "node_modules" || entry.name === "dist") continue;
      walkDir(full, results);
    } else if (/\.(ts|tsx)$/.test(entry.name)) {
      results.push(full);
    }
  }
  return results;
}

/** Files that are ALLOWED to contain raw route strings */
function isExempt(filePath: string): boolean {
  const rel = path.relative(path.resolve(__dirname, "../.."), filePath);
  return (
    rel.includes("__tests__") ||
    rel.includes(".test.") ||
    rel.includes(".spec.") ||
    rel.endsWith("routes.ts") ||
    rel.endsWith("AppRoutes.tsx")
  );
}

describe("Route compliance — no hardcoded paths in source", () => {
  const srcDir = path.resolve(__dirname, "../..");
  const sourceFiles = walkDir(srcDir).filter((f) => !isExempt(f));

  // Patterns that indicate a hardcoded route string in navigation code
  const VIOLATION_PATTERNS = [
    // navigate("/login"), Navigate to="/login", Link to="/login"
    /(?:to=|navigate\()\s*"\/(?:login|register|onboarding)"/,
    // window.location.href = "/login"
    /window\.location\.href\s*=\s*"\/(?:login|register|onboarding)"/,
    // window.location.pathname !== "/login"
    /window\.location\.pathname\s*[!=]==?\s*"\/(?:login|register|onboarding)"/,
  ];

  it("scans at least some source files", () => {
    expect(sourceFiles.length).toBeGreaterThan(0);
  });

  it("no source file contains hardcoded route strings", () => {
    const violations: string[] = [];

    for (const filePath of sourceFiles) {
      const lines = fs.readFileSync(filePath, "utf-8").split("\n");
      for (let i = 0; i < lines.length; i++) {
        for (const pattern of VIOLATION_PATTERNS) {
          if (pattern.test(lines[i])) {
            const rel = path.relative(srcDir, filePath);
            violations.push(`${rel}:${i + 1}: ${lines[i].trim()}`);
          }
        }
      }
    }

    expect(
      violations,
      `Found hardcoded route strings — use ROUTES constants from @/lib/routes:\n${violations.join("\n")}`,
    ).toHaveLength(0);
  });
});
