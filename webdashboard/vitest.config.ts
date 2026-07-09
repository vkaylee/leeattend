import react from '@vitejs/plugin-react';

import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'happy-dom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
    testTimeout: 60000,
    hookTimeout: 60000,
    exclude: ['**/node_modules/**', '**/dist/**', '**/tests/**', '**/e2e/**'],
    coverage: {
      provider: 'v8',
      // Exclude test infrastructure and pure utility glue from coverage
      // thresholds. `src/test/*` are setup/factory helpers exercised indirectly
      // by every test; `src/utils/hash.ts` is a thin platform-API wrapper.
      // Neither contains business logic worth gating CI on.
      exclude: [
        'src/test/**',
        'src/utils/hash.ts',
        'src/**/*.test.{ts,tsx}',
        'src/**/__tests__/**',
        'src/test/**/*.ts',
        'src/test/**/*.tsx',
      ],
      // Global floors track the post-debt-2 whole-project baseline (July
      // 2026: ~68% stmts / 60% branch / 62% funcs / 70% lines) with a
      // small margin above it so CI fails on real regression. The untested
      // UI modules sit well below the API layer, so the per-path floor for
      // ./src/lib/api/ is set higher to protect the backbone (see below).
      //
      // Branch coverage on the API layer is structurally capped below ~70%
      // because the test boundary is the apiFetch chokepoint: the branches
      // inside customFetch (URL sanitization, body extraction) and inside
      // openapi-fetch's response parser are exercised by client.test.ts but
      // not by per-module tests that mock apiFetch. Raising this further
      // requires integration tests that hit a real server.
      thresholds: {
        statements: 65,
        branches: 58,
        functions: 60,
        lines: 68,
        // Per-path floor for the API data-fetching backbone (570+ imports
        // depend on it). These numbers mirror the coverage achieved in the
        // debt-2 pass; do not lower them without re-running coverage.
        "./src/lib/api/": {
          statements: 80,
          branches: 55,
          functions: 90,
          lines: 85,
        },
      },
    },
  },
});