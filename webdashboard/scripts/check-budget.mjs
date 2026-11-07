import { readdirSync, statSync, readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

const __dirname = dirname(fileURLToPath(import.meta.url));
const distDir = resolve(__dirname, "../dist/assets");

const BUDGETS = [
  { name: "Main bundle", prefix: "index-", suffix: ".js", maxSize: 750_000, maxGzip: 210_000 },
  { name: "Vendor charts", prefix: "vendor-charts-", suffix: ".js", maxSize: 500_000, maxGzip: 150_000 },
  { name: "Vendor UI", prefix: "vendor-ui-", suffix: ".js", maxSize: 150_000, maxGzip: 50_000 },
  { name: "Vendor i18n", prefix: "vendor-i18n-", suffix: ".js", maxSize: 100_000, maxGzip: 30_000 },
  { name: "Vendor query", prefix: "vendor-query-", suffix: ".js", maxSize: 200_000, maxGzip: 60_000 },
  { name: "Vendor router", prefix: "vendor-router-", suffix: ".js", maxSize: 150_000, maxGzip: 40_000 },
  { name: "Vendor ExcelJS", prefix: "vendor-exceljs-", suffix: ".js", maxSize: 950_000, maxGzip: 280_000 },
  { name: "Vendor dnd", prefix: "vendor-dnd-", suffix: ".js", maxSize: 100_000, maxGzip: 30_000 },
  { name: "Vendor react", prefix: "vendor-react-", suffix: ".js", maxSize: 450_000, maxGzip: 130_000 },
  { name: "Login page", prefix: "login.lazy-", suffix: ".js", maxSize: 20_000, maxGzip: 5_000 },
  { name: "Dashboard page", prefix: "index.lazy-", suffix: ".js", maxSize: 80_000, maxGzip: 20_000 },
  // Locale raw caps are loose: non-ASCII translations cost 2-3 bytes per character
  // and compress away, so gzip is the meaningful gate for these chunks.
  { name: "Locale en", prefix: "locale-en-", suffix: ".js", maxSize: 260_000, maxGzip: 62_000 },
  { name: "Locale vi", prefix: "locale-vi-", suffix: ".js", maxSize: 260_000, maxGzip: 65_000 },
  { name: "Locale zh", prefix: "locale-zh-", suffix: ".js", maxSize: 260_000, maxGzip: 64_000 },
];

function findFiles(prefix, suffix) {
  try {
    return readdirSync(distDir)
      .filter((f) => f.startsWith(prefix) && f.endsWith(suffix))
      .map((f) => resolve(distDir, f));
  } catch {
    return [];
  }
}

function formatBytes(bytes) {
  if (bytes >= 1_000_000) return `${(bytes / 1_000_000).toFixed(2)} MB`;
  if (bytes >= 1_000) return `${(bytes / 1_000).toFixed(1)} kB`;
  return `${bytes} B`;
}

function isLazyChunk(name) {
  return name.includes(".lazy-") && name.endsWith(".js");
}

let allPassed = true;

for (const budget of BUDGETS) {
  const files = findFiles(budget.prefix, budget.suffix);
  if (files.length === 0) {
    console.log(`  ⚠️  No files matched: ${budget.name} (build may have changed)`);
    continue;
  }

  for (const file of files) {
    const stats = statSync(file);
    const rawSize = stats.size;
    const content = readFileSync(file);
    const gzipSize = gzipSync(content).length;

    const name = file.split("/").pop();
    const violations = [];

    if (rawSize > budget.maxSize) violations.push(`raw ${formatBytes(rawSize)} > ${formatBytes(budget.maxSize)}`);
    if (gzipSize > budget.maxGzip) violations.push(`gzip ${formatBytes(gzipSize)} > ${formatBytes(budget.maxGzip)}`);

    if (violations.length > 0) {
      allPassed = false;
      console.log(`  ❌ ${name}: ${violations.join(", ")}`);
    } else {
      console.log(`  ✓ ${name}: ${formatBytes(rawSize)} (gzip: ${formatBytes(gzipSize)})`);
    }
  }
}

// Check all lazy chunks (excluding dashboard + login which are covered above)
try {
  const lazyFiles = readdirSync(distDir)
    .filter((f) => isLazyChunk(f) && !f.startsWith("index.lazy-") && !f.startsWith("login.lazy-"))
    .map((f) => resolve(distDir, f));

  for (const file of lazyFiles) {
    const stats = statSync(file);
    const rawSize = stats.size;
    const content = readFileSync(file);
    const gzipSize = gzipSync(content).length;
    const name = file.split("/").pop();
    const violations = [];

    if (rawSize > 300_000) violations.push(`raw ${formatBytes(rawSize)} > 300 kB`);
    if (gzipSize > 60_000) violations.push(`gzip ${formatBytes(gzipSize)} > 60 kB`);

    if (violations.length > 0) {
      allPassed = false;
      console.log(`  ❌ ${name}: ${violations.join(", ")}`);
    } else {
      console.log(`  ✓ ${name}: ${formatBytes(rawSize)} (gzip: ${formatBytes(gzipSize)})`);
    }
  }
} catch {
  // distDir may not exist yet
}

if (!allPassed) {
  console.log("\n❌ Performance budget check FAILED");
  process.exit(1);
} else {
  console.log("\n✓ Performance budget check PASSED");
}
