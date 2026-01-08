import { defineConfig, mergeConfig } from "vitest/config";
import viteConfig from "./vite.config";

export default defineConfig(async (env) => {
  const baseConfig = await (typeof viteConfig === "function"
    ? viteConfig(env)
    : viteConfig);

  return mergeConfig(
    baseConfig,
    defineConfig({
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
    }),
  );
});
