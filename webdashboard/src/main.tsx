import React from "react";
import { createRoot } from "react-dom/client";

import App from "./app";
import { ToastProvider } from "./components/ui/toast";
import "./index.css";
import "@fontsource/inter";

import "./i18n/config";
import {
  clearChunkRetry,
  isChunkLoadError,
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
  // App mounted successfully — clear the chunk-reload counter so a future
  // deploy gets a fresh set of retries instead of a stale cumulative count.
  clearChunkRetry();
}
