import { CheckCircle2, XCircle, Info, AlertTriangle, X } from 'lucide-react';
import React, { createContext, useContext, useState, useCallback, ReactNode, useRef, useEffect } from 'react';
import { useTranslation } from 'react-i18next';

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


export function ToastProvider({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  const [toasts, setToasts] = useState<Toast[]>([]);
  const timeouts = useRef<Map<string, NodeJS.Timeout>>(new Map());

  const remove = useCallback((id: string) => {
    setToasts((previous) => previous.filter((t) => t.id !== id));
    const timeout = timeouts.current.get(id);
    if (timeout) {
      clearTimeout(timeout);
      timeouts.current.delete(id);
    }
  }, []);

  const show = useCallback((message: string, type: ToastType = 'info', title?: string) => {
    const id = Math.random().toString(36).substring(2, 9);
    setToasts((previous) => [...previous, { id, type, message, title }]);
    
    // Tự động đóng sau 5 giây
    const timeoutId = setTimeout(() => { remove(id); }, 5000);
    timeouts.current.set(id, timeoutId);
  }, [remove]);

  // Cleanup on unmount
  useEffect(() => {
    const currentTimeouts = timeouts.current;
    return () => {
      currentTimeouts.forEach((timeout) => { clearTimeout(timeout); });
      currentTimeouts.clear();
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
          <ToastItem key={toast.id} onClose={() => { remove(toast.id); }} toast={toast} />
        ))}
      </div>
    </ToastContext.Provider>
  );
}

function ToastItem({ toast, onClose }: { toast: Toast; onClose: () => void }) {
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

  return (
    <div 
      className={`pointer-events-auto flex items-start gap-4 p-4 rounded-[24px] border shadow-xl shadow-slate-200/50 dark:shadow-black/30 animate-in slide-in-from-right-8 duration-300 ${bgColors[toast.type]}`}
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
    </div>
  );
}

// eslint-disable-next-line react-refresh/only-export-components
export function useToast() {
  const context = useContext(ToastContext);
  if (!context) throw new Error('useToast must be used within ToastProvider');
  return context;
}
