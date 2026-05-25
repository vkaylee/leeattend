/* eslint-disable @typescript-eslint/no-deprecated, complexity, max-lines-per-function */
import react from "@vitejs/plugin-react";

import { TanStackRouterVite } from "@tanstack/router-vite-plugin";
import { execSync } from "node:child_process";
import fs from "node:fs";
import { IncomingMessage, ServerResponse } from "node:http";
import net from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { visualizer } from "rollup-plugin-visualizer";
import { defineConfig, ConfigEnv, Plugin } from "vite";



const __dirname = path.dirname(fileURLToPath(import.meta.url));

function getBuildHash(): string {
  try {
    return execSync("git rev-parse --short HEAD", { encoding: "utf-8" }).trim();
  } catch {
    return Date.now().toString(36);
  }
}

function buildVersionPlugin(): Plugin {
  const buildHash = getBuildHash();
  return {
    name: "build-version",
    transformIndexHtml(html) {
      return html.replace(
        "</head>",
        `    <meta name="build-version" content="${buildHash}" />\n  </head>`,
      );
    },
    config() {
      return {
        define: {
          __BUILD_HASH__: JSON.stringify(buildHash),
        },
      };
    },
  };
}

async function findAvailablePort(
  startPort: number,
  maxAttempts = 100,
): Promise<number> {
  for (let index = 0; index < maxAttempts; index++) {
    const port = startPort + index;
    const isAvailable = await new Promise<boolean>((resolve) => {
      const server = net.createServer();
      server.unref();
      server.on("error", () => { resolve(false); });
      server.listen(port, () => {
        server.close(() => { resolve(true); });
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
    const portFileName = process.env.VITE_BACKEND_PORT_FILE ?? ".api_port";
    const apiPortFile = path.resolve(__dirname, `../apiserver/${portFileName}`);
    if (fs.existsSync(apiPortFile)) {
      apiPort = fs.readFileSync(apiPortFile, "utf8").trim();
      
    }
  } catch {
    console.warn("📡 Vite: Could not discover API port");
  }

  // Ưu tiên discovery port nếu có, nếu không mới dùng origin từ env hoặc mặc định
  const apiTarget = apiPort ? `http://127.0.0.1:${apiPort}` : (apiOrigin || `http://localhost:3000`);
  const wsTarget = apiTarget.replace(/^http/, "ws");

  
  

  // Dùng port động nếu đang chạy test
  const workerIndex = parseInt(process.env.VITE_BACKEND_PORT_FILE?.split('_').pop() ?? "0", 10);
  const isTest = !!process.env.VITE_WEB_PORT_FILE || mode === 'test' || !!process.env.PLAYWRIGHT_TEST || !!process.env.PW_TEST;
  const startPort = isTest ? (5173 + (workerIndex * 50)) : 5173;
  const port = await findAvailablePort(startPort);

  return {
    plugins: [
      buildVersionPlugin(),
      TanStackRouterVite({
        routesDirectory: './src/routes',
        generatedRouteTree: './src/route-tree.gen.ts',
        quoteStyle: 'double',
        routeFileIgnorePattern: '.test.tsx',
      }),
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
      ...(process.env.ANALYZE === 'true' ? [visualizer({
        filename: 'dist/stats.html',
        open: true,
        gzipSize: true,
        brotliSize: true,
      })] : []),
      {
        name: 'save-port-file',
 
        configureServer(server: { httpServer: net.Server | null; address?: () => unknown }) { 
          server.httpServer?.once('listening', () => {
            const address = server.httpServer?.address();
            if (address && typeof address === 'object') {
              const actualPort = address.port;
              const webPortFile = process.env.VITE_WEB_PORT_FILE ?? ".web_port";
              fs.writeFileSync(webPortFile, actualPort.toString());
              
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
      host: process.env.VITE_DEV_SERVER_HOST ?? "127.0.0.1",
      port,
      strictPort: true,
      allowedHosts: ["leeattend.nport.link"],
      proxy: isTest ? {} : {
        // WebSocket proxy for report streaming (must be before /api)
        "/api/ws": {
          target: wsTarget,
          changeOrigin: true,
          ws: true,
          configure: (proxy: net.Socket, _options: unknown) => {
            proxy.on('_error', (_error: Error, _request: IncomingMessage, _res: ServerResponse) => { /* empty */ });
            proxy.on('proxyReq', (_proxyRequest: unknown, _request: IncomingMessage, _res: ServerResponse) => { /* empty */ });
          },
        },
        // HTTP API proxy
        "/api": {
          target: apiTarget,
          changeOrigin: true,
          configure: (proxy: net.Socket, _options: unknown) => {
            proxy.on('_error', (_error: Error, _request: IncomingMessage, _res: ServerResponse) => { /* empty */ });
            proxy.on('proxyReq', (_proxyRequest: unknown, _request: IncomingMessage, _res: ServerResponse) => { /* empty */ });
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
            'vendor-ui': ['@radix-ui/react-dialog', '@radix-ui/react-dropdown-menu', '@radix-ui/react-popover', 'cmdk', 'lucide-react'],
          },
        },
      },
    },
  };
});
