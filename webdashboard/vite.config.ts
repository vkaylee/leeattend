import path from "node:path";
import react from "@vitejs/plugin-react";
import { defineConfig, ConfigEnv } from "vite";
import fs from "node:fs";
import net from "node:net";
import { fileURLToPath } from "node:url";
import { IncomingMessage, ServerResponse } from "node:http";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

async function findAvailablePort(
  startPort: number,
  maxAttempts: number = 100,
): Promise<number> {
  for (let i = 0; i < maxAttempts; i++) {
    const port = startPort + i;
    const isAvailable = await new Promise<boolean>((resolve) => {
      const server = net.createServer();
      server.unref();
      server.on("error", () => resolve(false));
      server.listen(port, () => {
        server.close(() => resolve(true));
      });
    });
    if (isAvailable) return port;
    console.warn(`📡 Vite: Port ${port} is already in use, trying next one...`);
  }
  throw new Error(
    `📡 Vite: Could not find an available port in the range ${startPort}-${startPort + maxAttempts}. Throwing exception.`,
  );
}

//@ts-expect-error - legacy next.js config override during transition to vite
export default defineConfig(async ({ mode }: ConfigEnv) => {
  // Use process.env directly since we are in Docker and don't want to load .env files
  const viteApiUrl = process.env.VITE_API_URL;

  // Extract just origin (host:port) from VITE_API_URL for proxy target
  // VITE_API_URL now always includes /api (e.g., http://apiserver:3000/api)
  // but Vite proxy needs just the origin since it preserves the request path
  let apiOrigin = "";
  if (viteApiUrl) {
    try {
      apiOrigin = new URL(viteApiUrl).origin;
    } catch {
      apiOrigin = viteApiUrl;
    }
  }

  // Tự động tìm port của API từ file Discovery nếu không có biến môi trường
  let apiPort = "";
  try {
    const portFileName = process.env.VITE_BACKEND_PORT_FILE || ".api_port";
    const apiPortFile = path.resolve(__dirname, `../apiserver/${portFileName}`);
    if (fs.existsSync(apiPortFile)) {
      apiPort = fs.readFileSync(apiPortFile, "utf8").trim();
      console.log(`📡 Vite: Discovered API server on port ${apiPort} via ${portFileName}`);
    }
  } catch (e) {
    console.warn("📡 Vite: Could not discover API port");
  }

  // Ưu tiên discovery port nếu có, nếu không mới dùng origin từ env hoặc mặc định
  const apiTarget = apiPort ? `http://127.0.0.1:${apiPort}` : (apiOrigin || `http://localhost:3000`);
  const wsTarget = apiTarget.replace(/^http/, "ws");

  console.log(`📡 Vite: Proxying /api to ${apiTarget}`);
  console.log(`📡 Vite: Proxying /ws to ${wsTarget}`);

  // Dùng port động nếu đang chạy test
  const workerIndex = parseInt(process.env.VITE_BACKEND_PORT_FILE?.split('_').pop() || "0", 10);
  const isTest = !!process.env.VITE_WEB_PORT_FILE || mode === 'test' || !!process.env.PLAYWRIGHT_TEST || !!process.env.PW_TEST;
  const startPort = isTest ? (5173 + (workerIndex * 50)) : 5173;
  const port = await findAvailablePort(startPort);

  return {
    plugins: [
      react({
        babel: mode === 'production' ? {
          plugins: [
            [
              'react-remove-properties',
              {
                properties: ['data-debug-context', 'data-testid'],
              },
            ],
          ],
        } : undefined
      }),
      {
        name: 'save-port-file',
        configureServer(server: any) { // Explicitly declare server type as any
          server.httpServer?.once('listening', () => {
            const address = server.httpServer?.address();
            if (address && typeof address === 'object') {
              const actualPort = address.port;
              const webPortFile = process.env.VITE_WEB_PORT_FILE || ".web_port";
              fs.writeFileSync(webPortFile, actualPort.toString());
              console.log(`📡 Vite: Saved actual assigned port to ${webPortFile}: ${actualPort}`);
            }
          });
        }
      }
    ],
    resolve: {
      alias: {
        "@": path.resolve(__dirname, "./src"),
      },
    },
    server: {
      host: process.env.VITE_DEV_SERVER_HOST || "127.0.0.1",
      port,
      strictPort: true,
      allowedHosts: ["leeattend.nport.link"],
      proxy: isTest ? {} : {
        // WebSocket proxy for report streaming (must be before /api)
        "/api/ws": {
          target: wsTarget,
          changeOrigin: true,
          ws: true,
          configure: (proxy: any, _options: any) => {
            proxy.on('error', (err: Error, _req: IncomingMessage, _res: ServerResponse) => {
              console.log('📡 Vite: WebSocket proxy error', err);
            });
            proxy.on('proxyReq', (_proxyReq: IncomingMessage, req: IncomingMessage, _res: ServerResponse) => {
              console.log(`📡 Vite: Proxying WebSocket ${req.method} ${req.url} -> ${wsTarget}${req.url}`);
            });
          },
        },
        // HTTP API proxy
        "/api": {
          target: apiTarget,
          changeOrigin: true,
          configure: (proxy: any, _options: any) => {
            proxy.on('error', (err: Error, _req: IncomingMessage, _res: ServerResponse) => {
              console.log('📡 Vite: Proxy error', err);
            });
            proxy.on('proxyReq', (_proxyReq: IncomingMessage, req: IncomingMessage, _res: ServerResponse) => {
              console.log(`📡 Vite: Proxying request ${req.method} ${req.url} -> ${apiTarget}${req.url}`);
            });
          },
        },
        // Legacy WebSocket proxy (kept for compatibility)
        "/ws": {
          target: wsTarget,
          ws: true,
        },
      },
    },
    test: {
      globals: true,
      environment: "jsdom",
      setupFiles: ["./src/test/setup.ts"],
      exclude: ["**/node_modules/**", "**/dist/**", "**/tests/**"],
      coverage: {
        provider: "v8",
        reporter: ["text", "json", "html"],
      },
    },
    build: {
      rollupOptions: {
        output: {
          manualChunks: {
            'vendor-react': ['react', 'react-dom', 'react-router-dom'],
            'vendor-query': ['@tanstack/react-query'],
            'vendor-charts': ['recharts'],
            'vendor-i18n': ['i18next', 'react-i18next', 'i18next-browser-languagedetector'],
            'vendor-dnd': ['@dnd-kit/core', '@dnd-kit/sortable', '@dnd-kit/utilities'],
            'vendor-ui': ['@radix-ui/react-dialog', '@radix-ui/react-dropdown-menu', '@radix-ui/react-popover', 'cmdk'],
          },
        },
      },
    },
  };
});
