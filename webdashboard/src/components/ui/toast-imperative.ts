type ToastType = "success" | "error" | "info" | "warning";

export interface ToastContextType {
  error: (message: string, title?: string) => void;
  info: (message: string, title?: string) => void;
  show: (message: string, type?: ToastType, title?: string) => void;
  success: (message: string, title?: string) => void;
  warning: (message: string, title?: string) => void;
}

// ── Imperative (module-level) toast API ─────────────────────────────
// So non-React callers (e.g. QueryClient onError in app.tsx) can fire
// toasts without being inside a component tree.  ToastProvider registers
// its `show` callback on mount; the exported `toast` object fans out to
// all registered listeners (normally just one).

type ToastListener = (message: string, type: ToastType, title?: string) => void;
const listeners = new Set<ToastListener>();

function notify(message: string, type: ToastType, title?: string): void {
  listeners.forEach((fn) => { fn(message, type, title); });
}

/** Imperative toast — usable outside React (e.g. QueryClient callbacks). */
export const toast: ToastContextType = {
  show: (message, type = "info", title) => { notify(message, type, title); },
  success: (message, title) => { notify(message, "success", title); },
  error: (message, title) => { notify(message, "error", title); },
  info: (message, title) => { notify(message, "info", title); },
  warning: (message, title) => { notify(message, "warning", title); },
};

/** Register/deregister a listener callback (used by ToastProvider). */
export function addToastListener(fn: ToastListener): () => void {
  listeners.add(fn);
  return () => { listeners.delete(fn); };
}
