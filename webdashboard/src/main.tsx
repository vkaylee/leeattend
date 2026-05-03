import React from "react";
import { createRoot } from "react-dom/client";

import App from "./app";
import { ToastProvider } from "./components/ui/toast";
import "./index.css";
import "@fontsource/inter";

import "./i18n/config";
import { initSentry } from "./lib/sentry";
import { trackWebVitals } from "./lib/web-vitals";

initSentry();
trackWebVitals();

function isChunkLoadError(reason: Error | undefined): boolean {
  const url = (reason?.message ?? "") + " " + (reason?.stack ?? "");
  return url.includes("chunk") || url.includes(".js") || /loading.*chunk/i.test(reason?.message ?? "");
}

function handleChunkError(event: PromiseRejectionEvent): void {
  const reason = event.reason as Error | undefined;
  if (!isChunkLoadError(reason)) return;
  event.preventDefault();
  const retryCount = Number(sessionStorage.getItem("chunkRetry") ?? "0");
  if (retryCount < 3) {
    sessionStorage.setItem("chunkRetry", String(retryCount + 1));
    window.location.reload();
  }
}

window.addEventListener("unhandledrejection", handleChunkError);

const rootElement = document.getElementById("root");
if (rootElement) {
  createRoot(rootElement).render(
    <React.StrictMode>
      <ToastProvider>
        <App />
      </ToastProvider>
    </React.StrictMode>,
  );
}
