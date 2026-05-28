import { defineConfig } from "orval";

export default defineConfig({
  api: {
    input: {
      target: "./openapi.json",
    },
    output: {
      mode: "tags-split",
      target: "./src/lib/api-generated",
      schemas: "./src/lib/api-generated/models",
      client: "react-query",
      override: {
        mutator: {
          path: "./src/lib/api-generated/mutator.ts",
          name: "customFetch",
        },
      },
    },
  },
});
