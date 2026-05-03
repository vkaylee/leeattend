import React from "react";
import ReactDOM from "react-dom/client";

import App from "./app";
import { ToastProvider } from "./components/ui/toast";
import "./index.css";
import "@fontsource/inter";

import "./i18n/config";
import { initSentry } from "./lib/sentry";
import { trackWebVitals } from "./lib/web-vitals";

initSentry();
trackWebVitals();

window.addEventListener("unhandledrejection", (event) => {
  const reason = event.reason as Error | undefined;
  const url = (reason?.message ?? "") + " " + (reason?.stack ?? "");
  if (url.includes("chunk") || url.includes(".js") || /loading.*chunk/i.test(reason?.message ?? "")) {
    event.preventDefault();
    const retryCount = Number(sessionStorage.getItem("chunkRetry") ?? "0");
    if (retryCount < 3) {
      sessionStorage.setItem("chunkRetry", String(retryCount + 1));
      window.location.reload();
    }
  }
});

const rootElement = document.getElementById("root");
if (rootElement) {
  ReactDOM.createRoot(rootElement).render(
    <React.StrictMode>
      <ToastProvider>
        <App />
      </ToastProvider>
    </React.StrictMode>,
  );
}
