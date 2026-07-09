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
      // Thresholds set to current baseline + modest margin. Raising these
      // should be done incrementally as new tests are added for critical
        // paths (auth, payroll, attendance, RBAC). See P6 in the 5-debt plan.
      thresholds: {
        lines: 65,
        functions: 55,
        branches: 55,
        statements: 65,
      },
    },
  },
});