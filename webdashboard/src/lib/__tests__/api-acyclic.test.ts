import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const API_DIR = path.resolve(__dirname, "../api");
// Match runtime value imports/re-exports only (exclude `import type` / `export type`).
const IMPORT_RE = /(?:import|export)\s+(?!type\b)(?:[^'"]*?\s+from\s+)?["'](\.[^"']+)["']/g;

function localImports(filePath: string): string[] {
  const src = readFileSync(filePath, "utf8");
  const targets: string[] = [];
  for (const match of src.matchAll(IMPORT_RE)) {
    const spec = match[1];
    if (!spec) continue;
    const base = path.resolve(path.dirname(filePath), spec);
    const tsFile = base.endsWith(".ts") ? base : `${base}.ts`;
    if (tsFile.startsWith(API_DIR + path.sep)) {
      targets.push(path.relative(API_DIR, tsFile));
    }
  }
  return targets;
}

describe("lib/api module dependency graph", () => {
  it("contains no circular local imports", () => {
    const files = readdirSync(API_DIR).filter((f) => f.endsWith(".ts") && !f.endsWith(".test.ts"));
    const graph = new Map<string, string[]>();
    for (const file of files) {
      graph.set(file, localImports(path.join(API_DIR, file)));
    }

    const visiting = new Set<string>();
    const visited = new Set<string>();
    const cycles: string[][] = [];

    const dfs = (node: string, trail: string[]) => {
      visiting.add(node);
      for (const next of graph.get(node) ?? []) {
        if (!graph.has(next)) continue;
        if (visiting.has(next)) {
          const idx = trail.indexOf(next);
          cycles.push([...trail.slice(idx), next]);
          continue;
        }
        if (!visited.has(next)) {
          dfs(next, [...trail, next]);
        }
      }
      visiting.delete(node);
      visited.add(node);
    };

    for (const file of files) {
      if (!visited.has(file)) dfs(file, [file]);
    }

    expect(cycles).toEqual([]);
  });
});
