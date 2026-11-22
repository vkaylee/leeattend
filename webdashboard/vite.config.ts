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
  const configuredBuildId = process.env.LEEATTEND_BUILD_ID?.trim();
  if (configuredBuildId) return configuredBuildId;

  try {
    const commit = execSync("git rev-parse --short HEAD", { encoding: "utf-8" }).trim();
    return `${commit}-${Date.now().toString(36)}`;
  } catch {
    return Date.now().toString(36);
  }
}

function buildVersionPlugin(): Plugin {
  const buildHash = getBuildHash();
  return {
    name: "build-version",
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        if (req.url && req.url.split("?")[0] === "/version.json") {
          res.setHeader("Content-Type", "application/json");
          res.end(`${JSON.stringify({ buildId: buildHash })}\n`);
          return;
        }
        next();
      });
    },
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
    generateBundle() {
      this.emitFile({
        type: "asset",
        fileName: "version.json",
        source: `${JSON.stringify({ buildId: buildHash })}\n`,
      });
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

// @ts-expect-error - async config callback returns a Promise<UserConfig> not covered by Vite's current overloads
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
        babel: {
          plugins: [
            ['babel-plugin-react-compiler', { target: '19' }],
            ...(mode === 'production' && !isTest ? [
              [
                'react-remove-properties',
                {
                  properties: ['data-debug-context', 'data-testid'],
                },
              ],
            ] : []),
          ],
        }
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
      watch: {
        // ponytail: polling uses 500ms interval with ignored heavy paths; upgrade to native inotify if running outside container bind mounts
        usePolling: process.env.CHOKIDAR_USEPOLLING === "true" || process.env.VITE_USE_POLLING === "true",
        interval: 500,
        ignored: [
          "**/node_modules/**",
          "**/.git/**",
          "**/dist/**",
          "**/.tanstack/**",
        ],
      },
      host: process.env.VITE_DEV_SERVER_HOST ?? "127.0.0.1",
      port,
      strictPort: true,
      allowedHosts: ["leeattend.nport.link", "caddy"],
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
    build: {
      // Warn when any single chunk exceeds 500KB (down from 1000KB). This
      // surfaces accidental bundling of heavy deps into the main entry.
      chunkSizeWarningLimit: 500,
      rollupOptions: {
        output: {
          // Explicit vendor chunking keeps third-party code in long-lived
          // chunks (cached across deploys) and prevents a single huge vendor
          // blob. App code is left to Rollup's default splitting — TanStack
          // Router's lazy routes already code-split per route.
          manualChunks(id: string) {
            const normalized = id.replace(/\\/g, "/");
            // i18n: keep config + locale JSON out of the shared "common" chunks.
            // Without this, Rollup inlines ~35 KB of locale data into every
            // common chunk (we measured 65-77% of each common chunk = locale
            // JSON), triplicating ~106 KB across the app. A stable per-locale
            // chunk is lazy-loaded by i18next *only* when that language activates.
            if (normalized.includes("/src/i18n/")) {
              if (normalized.includes("/locales/")) {
                const lng = normalized.split("/locales/")[1]?.split("/")[0];
                return lng ? `locale-${lng}` : "i18n-locales";
              }
              return "i18n-config";
            }
            if (!id.includes("node_modules")) return undefined;
            if (
              normalized.includes("node_modules/react/") ||
              normalized.includes("node_modules/react-dom/") ||
              normalized.includes("node_modules/react-router-dom/")
            ) {
              return "vendor-react";
            }
            if (
              normalized.includes("node_modules/@tanstack/react-router") ||
              normalized.includes("node_modules/@tanstack/router-devtools") ||
              normalized.includes("node_modules/@tanstack/router-vite-plugin")
            ) {
              return "vendor-router";
            }
            if (normalized.includes("node_modules/@tanstack/")) {
              return "vendor-query";
            }
            if (normalized.includes("node_modules/recharts/")) {
              return "vendor-charts";
            }
            if (
              normalized.includes("node_modules/i18next/") ||
              normalized.includes("node_modules/react-i18next/") ||
              normalized.includes("node_modules/i18next-browser-languagedetector/")
            ) {
              return "vendor-i18n";
            }
            if (normalized.includes("node_modules/@dnd-kit/")) {
              return "vendor-dnd";
            }
            if (
              normalized.includes("node_modules/@radix-ui/") ||
              normalized.includes("node_modules/cmdk/") ||
              normalized.includes("node_modules/lucide-react/")
            ) {
              return "vendor-ui";
            }
            // Heavy deps used only on specific async paths — give them their
            // own chunk so they never land in the initial bundle.
            if (normalized.includes("node_modules/exceljs/")) {
              return "vendor-exceljs";
            }
            if (normalized.includes("node_modules/@xyflow/")) {
              return "vendor-xyflow";
            }
            if (normalized.includes("node_modules/sentry/") || normalized.includes("node_modules/@sentry/")) {
              return "vendor-sentry";
            }
            if (normalized.includes("node_modules/sonner/")) {
              return "vendor-sonner";
            }
            // Return undefined for anything else — let Rollup apply its
            // default splitting. (A catch-all "vendor-shared" here creates a
            // circular dependency with vendor-react because some shared libs
            // depend on react and vice-versa.)
            return undefined;
          },
        },
      },
    },
  };
});
