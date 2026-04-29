import fs from "fs";
import path from "path";

import { describe, it, expect } from "vitest";

describe("API Standardization Enforcement", () => {
  const apiDir = path.resolve(__dirname, "../api");
  const files = fs.readdirSync(apiDir).filter(f => f.endsWith(".ts") && f !== "helpers.ts" && f !== "types.ts");

  it("should not use direct fetch() in API implementation files", () => {
    for (const file of files) {
      const content = fs.readFileSync(path.join(apiDir, file), "utf8");
      
      // Look for fetch() calls that are not preceded by 'api'
      // This regex matches "fetch(" but not "apiFetch("
      const directFetchMatch = content.match(/(?<!api)fetch\(/g);
      
      expect(directFetchMatch, `File ${file} contains direct fetch() calls. Use apiFetch() instead.`).toBeNull();
    }
  });

  it("should always import apiFetch from helpers", () => {
    for (const file of files) {
      const content = fs.readFileSync(path.join(apiDir, file), "utf8");
      
      // If the file makes API calls, it should import apiFetch
      if (content.includes("apiFetch(")) {
        const importMatch = content.match(/import\s*{[^}]*apiFetch[^}]*}\s*from\s*"(\.\/helpers|..\/api\/helpers)"/);
        expect(importMatch, `File ${file} uses apiFetch but does not import it correctly from helpers.`).not.toBeNull();
      }
    }
  });
});
