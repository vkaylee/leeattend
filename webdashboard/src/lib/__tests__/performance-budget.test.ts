import { execSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it, expect } from "vitest";

describe("performance budget", () => {
  const distDir = resolve(__dirname, "../../../dist/assets");

  it("dist/assets directory exists (build has been run)", () => {
    expect(existsSync(distDir)).toBe(true);
  });

  it("main bundle is under 750 kB", () => {
    const files = readdirSync(distDir).filter(
      (f: string) => f.startsWith("index-") && f.endsWith(".js") && !f.includes(".lazy-"),
    );
    expect(files.length).toBeGreaterThan(0);
    for (const file of files) {
      const size = statSync(resolve(distDir, file)).size;
      expect(size).toBeLessThan(750_000);
    }
  });

  it("vendor chunks are under their individual budgets", () => {
    const files = readdirSync(distDir).filter(
      (f: string) => f.startsWith("vendor-") && f.endsWith(".js"),
    );

    const budgets: Record<string, number> = {
      "vendor-charts-": 500_000,
      "vendor-ui-": 200_000,
      "vendor-i18n-": 100_000,
      // @tanstack/react-query + @tanstack/react-router are bundled together
      // in the vendor-query chunk — they are genuinely large.
      "vendor-query-": 200_000,
      "vendor-dnd-": 100_000,
      "vendor-react-": 450_000,
      // Heavy deps isolated to their own chunks (loaded on-demand only).
      "vendor-exceljs-": 1_000_000,
      "vendor-xyflow-": 200_000,
      "vendor-sentry-": 100_000,
    };

    for (const file of files) {
      for (const [prefix, maxSize] of Object.entries(budgets)) {
        if (file.startsWith(prefix)) {
          const size = statSync(resolve(distDir, file)).size;
          expect(size, `${file} exceeds budget of ${maxSize}`).toBeLessThan(maxSize);
        }
      }
    }
  });

  it("lazy route chunks are under 300 kB", () => {
    const files = readdirSync(distDir).filter(
      (f: string) => f.includes(".lazy-") && f.endsWith(".js"),
    );
    expect(files.length).toBeGreaterThan(20); // at least 20 lazy chunks
    for (const file of files) {
      const size = statSync(resolve(distDir, file)).size;
      expect(size, `${file} exceeds 300 kB budget`).toBeLessThan(300_000);
    }
  });

  it("check-budget script exits 0 against current build", () => {
    const result = execSync("node scripts/check-budget.mjs", {
      cwd: resolve(__dirname, "../../.."),
      encoding: "utf-8",
    });
    expect(result).toContain("PASSED");
  });

  it("_headers file is present in root of dist", () => {
    const headersPath = resolve(__dirname, "../../../dist/_headers");
    expect(existsSync(headersPath)).toBe(true);
    const content = readFileSync(headersPath, "utf-8");
    expect(content).toContain("no-cache");
    expect(content).toContain("immutable");
  });

  it("build-version meta tag is injected into index.html", () => {
    const html = readFileSync(resolve(__dirname, "../../../dist/index.html"), "utf-8");
    expect(html).toMatch(/<meta\s+name="build-version"\s+content="[a-zA-Z0-9-]+"\s*\/>/);
  });

  it("version manifest is emitted with the same build id as index.html", () => {
    const html = readFileSync(resolve(__dirname, "../../../dist/index.html"), "utf-8");
    const match = /<meta\s+name="build-version"\s+content="([a-zA-Z0-9-]+)"\s*\/>/.exec(html);
    expect(match).not.toBeNull();

    const manifest = JSON.parse(
      readFileSync(resolve(__dirname, "../../../dist/version.json"), "utf-8"),
    ) as { buildId?: string };
    expect(manifest.buildId).toBe(match![1]);
  });
});
