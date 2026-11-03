/// <reference types="vite/client" />

interface ImportMetaEnvironment {
  readonly VITE_API_URL: string;
  readonly VITE_STANDALONE_WORKSPACE_ID?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnvironment;
}
