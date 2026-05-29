import fs from "fs";
import path from "path";
import { describe, it, expect } from "vitest";

// Helper to recursively get all .ts and .tsx files in a directory
function getAllTsFiles(dir: string): string[] {
  let results: string[] = [];
  const list = fs.readdirSync(dir);
  for (const file of list) {
    const fullPath = path.join(dir, file);
    const stat = fs.statSync(fullPath);
    if (stat && stat.isDirectory()) {
      results = results.concat(getAllTsFiles(fullPath));
    } else if (fullPath.endsWith(".ts") || fullPath.endsWith(".tsx")) {
      results.push(fullPath);
    }
  }
  return results;
}

describe("API Standardization Enforcement", () => {
  const srcDir = path.resolve(__dirname, "../../../src");
  const allFiles = getAllTsFiles(srcDir);

  // Filter out the helpers file where fetch and apiFetch are defined
  const sourceFiles = allFiles.filter(f => !f.endsWith("helpers.ts") && !f.endsWith("types.ts"));

  it("should not use direct fetch() in source files", () => {
    for (const file of sourceFiles) {
      // Exclude generated files, tests, and files that legitimately need raw fetch
      if (
        file.includes("api-generated") || 
        file.includes("__tests__") ||
        file.endsWith("use-version-check.ts")
      ) continue;
      
      const content = fs.readFileSync(file, "utf8");
      
      // Look for fetch() calls that are not preceded by word characters
      // This regex matches "fetch(" but not "apiFetch(", "customFetch(", or "refetch("
      const directFetchMatch = content.match(/(?<![a-zA-Z0-9_])fetch\(/g);
      
      expect(directFetchMatch, `File ${file} contains direct fetch() calls. Use apiFetch() instead.`).toBeNull();
    }
  });

  it("should always import apiFetch from helpers", () => {
    for (const file of sourceFiles) {
      if (file.includes("api-generated") || file.includes("__tests__")) continue;

      const content = fs.readFileSync(file, "utf8");
      
      if (content.includes("apiFetch(")) {
        // Matches standard import paths to helpers.ts or api index
        const importMatch = /import\s*{[^}]*apiFetch[^}]*}\s*from\s*"[^"]*(helpers|api)"/.exec(content);
        expect(importMatch, `File ${file} uses apiFetch but does not import it correctly from helpers.`).not.toBeNull();
      }
    }
  });

  it("should not pass API_BASE_URL into apiFetch()", () => {
    for (const file of sourceFiles) {
      if (file.includes("api-generated") || file.includes("__tests__")) continue;

      const content = fs.readFileSync(file, "utf8");
      
      // Look for apiFetch calls that interpolate API_BASE_URL
      // e.g. apiFetch(`${API_BASE_URL}/workspaces...`)
      const apiUrlMatch = content.match(/apiFetch\([^)]*API_BASE_URL/g);
      
      expect(apiUrlMatch, `File ${file} passes API_BASE_URL into apiFetch(). apiFetch handles the base URL automatically.`).toBeNull();
    }
  });

  it("should always prefix internal endpoints with /v1/ when using apiFetch()", () => {
    for (const file of sourceFiles) {
      // Ignore standardization tests and generated files
      if (file.includes("api-generated") || file.includes("__tests__")) continue;

      const content = fs.readFileSync(file, "utf8");
      
      // Find all string arguments passed to apiFetch that start with a slash
      // Matches: apiFetch("/...") or apiFetch(`/...`)
      const endpointRegex = /apiFetch\(\s*[`'"](\/[^`'"]*)[`'"]/g;
      
      let match;
      while ((match = endpointRegex.exec(content)) !== null) {
        const endpoint = match[1];
        // Allow /debug endpoints for tests if needed, but in our project they are /v1/debug
        expect(endpoint.startsWith("/v1/"), `File ${file} calls apiFetch("${endpoint}") which does not start with /v1/. All API routes must be prefixed with /v1/!`).toBe(true);
      }
    }
  });
});
