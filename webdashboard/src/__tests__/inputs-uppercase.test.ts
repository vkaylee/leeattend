import fs from "fs";
import path from "path";

import { describe, it, expect } from "vitest";

function walk(dir: string): string[] {
  let results: string[] = [];
  const list = fs.readdirSync(dir);
  list.forEach(function (file) {
    file = path.join(dir, file);
    const stat = fs.statSync(file);
    if (stat && stat.isDirectory()) {
      results = results.concat(walk(file));
    } else if (file.endsWith(".tsx") || file.endsWith(".jsx")) {
      results.push(file);
    }
  });
  return results;
}

describe("UI Standards: Inputs", () => {
  it("should not force uppercase on text inputs", () => {
    const srcPath = path.resolve(__dirname, "..");
    const files = walk(srcPath);
    const violations: string[] = [];

    // Regex to find input or textarea with uppercase class
    const inputRegex = /<(?:input|textarea)[^>]*className=(?:\{`|["'])[^>]*>/g;

    files.forEach((file) => {
      const content = fs.readFileSync(file, "utf8");
      let match;
      while ((match = inputRegex.exec(content)) !== null) {
        if (/\buppercase\b/.test(match[0])) {
          violations.push(`File: ${file.replace(srcPath, "")}\nTag: ${match[0]}`);
        }
      }
    });

    expect(
      violations,
      "Found inputs or textareas with 'uppercase' class. Users should see their exact input case."
    ).toHaveLength(0);
  });
});
