import path from 'path';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';
import fs from 'fs';

// Tự động tìm port của API từ file Discovery
let apiPort = '3000';
try {
  const apiPortFile = path.resolve(__dirname, '../apiserver/.api_port');
  if (fs.existsSync(apiPortFile)) {
    apiPort = fs.readFileSync(apiPortFile, 'utf8').trim();
    console.log(`📡 Vite: Discovered API server on port ${apiPort}`);
  }
} catch (e) {
  console.warn('📡 Vite: Could not discover API port, using default 3000');
}

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    strictPort: false,
    proxy: {
      '/api': {
        target: `http://localhost:${apiPort}`,
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
      '/ws': {
        target: `ws://localhost:${apiPort}`,
        ws: true,
      },
    },
  },
  // @ts-ignore
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    exclude: ['**/node_modules/**', '**/dist/**', '**/tests/**'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
    },
  },
});
