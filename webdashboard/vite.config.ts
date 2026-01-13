import path from "path";
import react from "@vitejs/plugin-react";
import { defineConfig, loadEnv } from "vite";
import fs from "fs";
import net from "node:net";

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

export default defineConfig(async ({ mode }) => {
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
  const startPort = isTest ? (5173 + (workerIndex * 10)) : 5173;
  const port = await findAvailablePort(startPort);

  return {
    plugins: [
      react(),
      {
        name: 'save-port-file',
        configureServer(server) {
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
      port,
      strictPort: true,
      allowedHosts: ["leeattend.nport.link"],
      proxy: {
        "/api": {
          target: apiTarget,
          changeOrigin: true,
          configure: (proxy, _options) => {
            proxy.on('error', (err, _req, _res) => {
              console.log('📡 Vite: Proxy error', err);
            });
            proxy.on('proxyReq', (_proxyReq, req, _res) => {
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
    // @ts-ignore
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
