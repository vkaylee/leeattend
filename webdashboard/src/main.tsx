import React from "react";

import ReactDOM from "react-dom/client";

import App from "./App";
import { ToastProvider } from "./components/ui/Toast";
import "./index.css";
import "@fontsource/inter";
import "./i18n/config";
import { initSentry } from "./lib/sentry";
import { trackWebVitals } from "./lib/web-vitals";

initSentry();
trackWebVitals();

window.addEventListener("unhandledrejection", (event) => {
  const url = (event.reason?.message || "") + " " + (event.reason?.stack || "");
  if (url.includes("chunk") || url.includes(".js") || /loading.*chunk/i.test(event.reason?.message || "")) {
    event.preventDefault();
    const retryCount = Number(sessionStorage.getItem("chunkRetry") || "0");
    if (retryCount < 3) {
      sessionStorage.setItem("chunkRetry", String(retryCount + 1));
      window.location.reload();
    }
  }
});

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ToastProvider>
      <App />
    </ToastProvider>
  </React.StrictMode>,
);
