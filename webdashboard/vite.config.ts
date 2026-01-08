import path from 'path';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';
import fs from 'fs';
import net from 'node:net';

async function findAvailablePort(startPort: number, maxAttempts: number = 100): Promise<number> {
  for (let i = 0; i < maxAttempts; i++) {
    const port = startPort + i;
    const isAvailable = await new Promise<boolean>((resolve) => {
      const server = net.createServer();
      server.unref();
      server.on('error', () => resolve(false));
      server.listen(port, () => {
        server.close(() => resolve(true));
      });
    });
    if (isAvailable) return port;
    console.warn(`📡 Vite: Port ${port} is already in use, trying next one...`);
  }
  throw new Error(`📡 Vite: Could not find an available port in the range ${startPort}-${startPort + maxAttempts}. Throwing exception.`);
}

export default defineConfig(async () => {
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

  const port = await findAvailablePort(5173);

  // Ghi port vào file để Playwright/AI có thể tự động khám phá
  try {
    fs.writeFileSync('.web_port', port.toString());
    console.log(`📡 Vite: Saved actual port to .web_port: ${port}`);
  } catch (e) {
    console.warn('📡 Vite: Could not save .web_port file');
  }

  return {
    plugins: [react()],
    resolve: {
      alias: {
        '@': path.resolve(__dirname, './src'),
      },
    },
    server: {
      port,
      strictPort: true,
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
  };
});
