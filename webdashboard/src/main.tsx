import React from "react";
import { createRoot } from "react-dom/client";

import App from "./app";
import { ToastProvider } from "./components/ui/toast";
import "./index.css";
import "@fontsource/inter";

import "./i18n/config";
import {
  isChunkLoadError,
  scheduleChunkRetryReset,
  shouldRetryChunkLoad,
} from "./lib/chunk-error";
import { initSentry } from "./lib/sentry";
import { trackWebVitals } from "./lib/web-vitals";

initSentry();
trackWebVitals();

function reloadForChunkError(): void {
  if (shouldRetryChunkLoad()) {
    window.location.reload();
  }
}

function handleChunkError(event: PromiseRejectionEvent): void {
  const reason = event.reason as Error | undefined;
  if (!isChunkLoadError(reason)) return;
  event.preventDefault();
  reloadForChunkError();
}

window.addEventListener("unhandledrejection", handleChunkError);
window.addEventListener("vite:preloadError", (event) => {
  event.preventDefault();
  reloadForChunkError();
});

const rootElement = document.getElementById("root");
if (rootElement) {
  createRoot(rootElement).render(
    <React.StrictMode>
      <ToastProvider>
        <App />
      </ToastProvider>
    </React.StrictMode>,
  );
  // A lazy chunk can still fail after the root mounts. Keep the retry budget
  // alive until the app has remained stable long enough to be trusted.
  scheduleChunkRetryReset();
}
