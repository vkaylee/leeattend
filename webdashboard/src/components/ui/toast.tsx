import { CheckCircle2, XCircle, Info, AlertTriangle, X } from 'lucide-react';
import React, { createContext, useContext, useState, useCallback, ReactNode, useRef, useEffect } from 'react';
import { useTranslation } from 'react-i18next';

const TOAST_DURATION_MS = 5000;

type ToastType = 'success' | 'error' | 'info' | 'warning';

interface Toast {
  id: string;
  message: string;
  title?: string;
  type: ToastType;
}

export interface ToastContextType {
  error: (message: string, title?: string) => void;
  info: (message: string, title?: string) => void;
  show: (message: string, type?: ToastType, title?: string) => void;
  success: (message: string, title?: string) => void;
  warning: (message: string, title?: string) => void;
}

const ToastContext = createContext<ToastContextType | undefined>(undefined);

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
  show: (message, type = 'info', title) => { notify(message, type, title); },
  success: (message, title) => { notify(message, 'success', title); },
  error: (message, title) => { notify(message, 'error', title); },
  info: (message, title) => { notify(message, 'info', title); },
  warning: (message, title) => { notify(message, 'warning', title); },
};

/** Tracks remaining time for each toast so hover-pause can resume correctly */
interface ToastTimer {
  remaining: number;
  startedAt: number;
  timeout: ReturnType<typeof setTimeout>;
}

export function ToastProvider({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  const [toasts, setToasts] = useState<Toast[]>([]);
  const timers = useRef<Map<string, ToastTimer>>(new Map());

  const remove = useCallback((id: string) => {
    setToasts((previous) => previous.filter((toast) => toast.id !== id));
    const timer = timers.current.get(id);
    if (timer) {
      clearTimeout(timer.timeout);
      timers.current.delete(id);
    }
  }, []);

  const pauseTimer = useCallback((id: string) => {
    const timer = timers.current.get(id);
    if (!timer) return;
    clearTimeout(timer.timeout);
    const elapsed = Date.now() - timer.startedAt;
    timer.remaining = Math.max(0, timer.remaining - elapsed);
  }, []);

  const resumeTimer = useCallback((id: string) => {
    const timer = timers.current.get(id);
    if (!timer) return;
    timer.startedAt = Date.now();
    timer.timeout = setTimeout(() => { remove(id); }, timer.remaining);
  }, [remove]);

  const show = useCallback((message: string, type: ToastType = 'info', title?: string) => {
    const id = Math.random().toString(36).substring(2, 9);
    setToasts((previous) => [...previous, { id, type, message, title }]);
    
    const timeout = setTimeout(() => { remove(id); }, TOAST_DURATION_MS);
    timers.current.set(id, {
      remaining: TOAST_DURATION_MS,
      startedAt: Date.now(),
      timeout,
    });
  }, [remove]);

  // Wire the imperative toast API so callers outside the React tree
  // (e.g. QueryClient onError) render through this provider.
  useEffect(() => {
    listeners.add(show);
    return () => { listeners.delete(show); };
  }, [show]);

  // Cleanup on unmount
  useEffect(() => {
    const currentTimers = timers.current;
    return () => {
      currentTimers.forEach((timer) => { clearTimeout(timer.timeout); });
      currentTimers.clear();
    };
  }, []);

  const success = (message: string, title?: string) => { show(message, 'success', title ?? t('common.success')); };
  const error = (message: string, title?: string) => { show(message, 'error', title ?? t('common.error')); };
  const info = (message: string, title?: string) => { show(message, 'info', title ?? t('common.info')); };
  const warning = (message: string, title?: string) => { show(message, 'warning', title ?? t('common.warning')); };

  return (
    <ToastContext.Provider value={{ show, success, error, info, warning }}>
      {children}
      <div className="fixed bottom-8 right-8 z-[9999] flex flex-col gap-3 pointer-events-none max-w-sm w-full">
        {toasts.map((toast) => (
          <ToastItem
            key={toast.id}
            onClose={() => { remove(toast.id); }}
            onPause={() => { pauseTimer(toast.id); }}
            onResume={() => { resumeTimer(toast.id); }}
            toast={toast}
          />
        ))}
      </div>
    </ToastContext.Provider>
  );
}

const progressColors: Record<ToastType, string> = {
  success: 'bg-emerald-500',
  error: 'bg-rose-500',
  info: 'bg-blue-500',
  warning: 'bg-amber-500',
};

function ToastItem({ toast, onClose, onPause, onResume }: {
  toast: Toast;
  onClose: () => void;
  onPause: () => void;
  onResume: () => void;
}) {
  const [isPaused, setIsPaused] = useState(false);

  const icons = {
    success: <CheckCircle2 className="w-5 h-5 text-emerald-500" />,
    error: <XCircle className="w-5 h-5 text-rose-500" />,
    info: <Info className="w-5 h-5 text-blue-500" />,
    warning: <AlertTriangle className="w-5 h-5 text-amber-500" />,
  };

  const bgColors = {
    success: 'bg-emerald-50 border-emerald-100 dark:bg-emerald-950 dark:border-emerald-900',
    error: 'bg-rose-50 border-rose-100 dark:bg-rose-950 dark:border-rose-900',
    info: 'bg-primary-50 border-blue-100 dark:bg-blue-950 dark:border-blue-900',
    warning: 'bg-amber-50 border-amber-100 dark:bg-amber-950 dark:border-amber-900',
  };

  const handleMouseEnter = () => {
    setIsPaused(true);
    onPause();
  };

  const handleMouseLeave = () => {
    setIsPaused(false);
    onResume();
  };

  return (
    <div 
      className={`pointer-events-auto flex items-start gap-4 p-4 rounded-[24px] border shadow-xl shadow-slate-200/50 dark:shadow-black/30 animate-in slide-in-from-right-8 duration-300 relative overflow-hidden ${bgColors[toast.type]}`}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      role="status"
    >
      <div className="shrink-0 mt-0.5">{icons[toast.type]}</div>
      <div className="flex-1">
        {!!toast.title && <div className="text-sm font-bold text-slate-700 dark:text-slate-200 mb-0.5 uppercase tracking-tight">{toast.title}</div>}
        <div className="text-sm text-slate-600 dark:text-slate-300 font-medium leading-relaxed">{toast.message}</div>
      </div>
      <button className="shrink-0 p-1 hover:bg-white/50 dark:hover:bg-slate-700/50 rounded-lg transition-colors" onClick={onClose}>
        <X className="w-4 h-4 text-slate-500 dark:text-slate-400" />
      </button>

      {/* Progress bar — counts down the auto-dismiss timer */}
      <div className="absolute bottom-0 left-0 right-0 h-[3px] overflow-hidden">
        <div
          className={`h-full rounded-b-[24px] opacity-60 ${progressColors[toast.type]}`}
          style={{
            animation: `toast-progress ${TOAST_DURATION_MS}ms linear forwards`,
            animationPlayState: isPaused ? 'paused' : 'running',
          }}
        />
      </div>
    </div>
  );
}

// eslint-disable-next-line react-refresh/only-export-components
export function useToast() {
  const context = useContext(ToastContext);
  if (!context) throw new Error('useToast must be used within ToastProvider');
  return context;
}
