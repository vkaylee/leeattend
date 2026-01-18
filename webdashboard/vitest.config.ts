import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
    testTimeout: 15000,
    hookTimeout: 30000,
    exclude: ['**/node_modules/**', '**/dist/**', '**/tests/**', '**/e2e/**'],
  },
});