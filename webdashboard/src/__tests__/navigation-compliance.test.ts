import fs from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

describe("Navigation & Layout Compliance Standards", () => {
  it("should prevent key={location.pathname} on layout/outlet wrapper div inside app-shell.tsx to prevent full remount lag", () => {
    const appShellPath = path.resolve(__dirname, "../components/layout/app-shell.tsx");
    const content = fs.readFileSync(appShellPath, "utf8");

    // Ensure the key={location.pathname} is not present on wrapper element enclosing the Outlet
    // The previous anti-pattern was:
    // <div className={...} key={location.pathname}>
    //   <ErrorBoundary><Outlet /></ErrorBoundary>
    // </div>
    const remountAntiPattern = /<div[^>]*key\s*=\s*\{\s*location\.pathname\s*\}[^>]*>[\s\S]*?<Outlet/i;
    expect(
      remountAntiPattern.test(content),
      "Detected key={location.pathname} wrapping <Outlet /> in app-shell.tsx! " +
        "This forces full layout remount on every route switch, destroying performance. " +
        "Ensure transitions are handled at the router page level, not by unmounting the layout wrapper."
    ).toBe(false);
  });

  it("should warn about performance-heavy key={location.pathname} on generic divs", () => {
    const srcPath = path.resolve(__dirname, "..");

    // Recursive search files
    function walk(dir: string): string[] {
      let results: string[] = [];
      const list = fs.readdirSync(dir);
      for (const file of list) {
        const fullPath = path.join(dir, file);
        const stat = fs.statSync(fullPath);
        if (stat.isDirectory()) {
          // Skip tests, node_modules, and UI kit
          if (!["__tests__", "node_modules", "ui"].includes(file)) {
            results = results.concat(walk(fullPath));
          }
        } else if (file.endsWith(".tsx")) {
          results.push(fullPath);
        }
      }
      return results;
    }

    const files = walk(srcPath);
    const violations: string[] = [];

    // Match generic HTML tags with key={location.pathname}
    // (e.g. <div key={location.pathname}> or <main key={location.pathname}>)
    // Avoid checking ErrorBoundary because reset key is valid there.
    const genericKeyRegex = /<(div|main|section|span|aside|header)[^>]*key\s*=\s*\{\s*location\.pathname\s*\}/g;

    for (const file of files) {
      const content = fs.readFileSync(file, "utf8");
      if (genericKeyRegex.test(content)) {
        violations.push(path.relative(srcPath, file));
      }
    }

    expect(
      violations,
      `Found key={location.pathname} on generic layout tags in: ${violations.join(", ")}.
This forces React to discard the DOM node and recreate it. Please remove it or use React transitions.`
    ).toHaveLength(0);
  });
});
