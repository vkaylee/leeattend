import path from "path";
import react from "@vitejs/plugin-react";
import { defineConfig, loadEnv, ConfigEnv } from "vite";
import fs from "fs";
import net from "node:net";
import { IncomingMessage, ServerResponse } from "http"; // Correct types for proxy handlers

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
  // Load env variables based on mode (development, production, etc.)
  const env = loadEnv(mode, process.cwd());
  const viteApiUrl = env.VITE_API_URL;

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

  // Ưu tiên discovery port nếu có, nếu không mới dùng env hoặc mặc định 3000
  const apiTarget = apiPort ? `http://127.0.0.1:${apiPort}` : (viteApiUrl || `http://localhost:3000`);
  const wsTarget = apiTarget.replace(/^http/, "ws");

  console.log(`📡 Vite: Proxying /api to ${apiTarget}`);
  console.log(`📡 Vite: Proxying /ws to ${wsTarget}`);

  // Dùng port động nếu đang chạy test (có VITE_WEB_PORT_FILE)
  const workerIndex = parseInt(process.env.VITE_BACKEND_PORT_FILE?.split('_').pop() || "0", 10);
  const isTest = !!process.env.VITE_WEB_PORT_FILE;
  const startPort = isTest ? (5173 + (workerIndex * 50)) : 5173;
  const port = await findAvailablePort(startPort);

  return {
    plugins: [
      react(),
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
      host: env.VITE_DEV_SERVER_HOST || "127.0.0.1",
      port,
      strictPort: true,
      allowedHosts: ["leeattend.nport.link"],
      proxy: {
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
  };
});
