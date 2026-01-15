import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';
import { CheckCircle2, XCircle, Info, AlertTriangle, X } from 'lucide-react';

type ToastType = 'success' | 'error' | 'info' | 'warning';

interface Toast {
  id: string;
  type: ToastType;
  message: string;
  title?: string;
}

interface ToastContextType {
  show: (message: string, type?: ToastType, title?: string) => void;
  success: (message: string, title?: string) => void;
  error: (message: string, title?: string) => void;
  info: (message: string, title?: string) => void;
  warning: (message: string, title?: string) => void;
}

const ToastContext = createContext<ToastContextType | undefined>(undefined);

import { useTranslation } from 'react-i18next';

export function ToastProvider({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  const [toasts, setToasts] = useState<Toast[]>([]);

  const remove = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const show = useCallback((message: string, type: ToastType = 'info', title?: string) => {
    const id = Math.random().toString(36).substring(2, 9);
    setToasts((prev) => [...prev, { id, type, message, title }]);
    
    // Tự động đóng sau 5 giây
    setTimeout(() => remove(id), 5000);
  }, [remove]);

  const success = (msg: string, title?: string) => show(msg, 'success', title || t('common.success'));
  const error = (msg: string, title?: string) => show(msg, 'error', title || t('common.error'));
  const info = (msg: string, title?: string) => show(msg, 'info', title || t('common.info'));
  const warning = (msg: string, title?: string) => show(msg, 'warning', title || t('common.warning'));

  return (
    <ToastContext.Provider value={{ show, success, error, info, warning }}>
      {children}
      <div className="fixed bottom-8 right-8 z-[9999] flex flex-col gap-3 pointer-events-none max-w-sm w-full">
        {toasts.map((toast) => (
          <ToastItem key={toast.id} toast={toast} onClose={() => remove(toast.id)} />
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
    success: 'bg-emerald-50 border-emerald-100',
    error: 'bg-rose-50 border-rose-100',
    info: 'bg-primary-50 border-blue-100',
    warning: 'bg-amber-50 border-amber-100',
  };

  return (
    <div className={`pointer-events-auto flex items-start gap-4 p-4 rounded-[24px] border shadow-xl shadow-slate-200/50 animate-in slide-in-from-right-8 duration-300 ${bgColors[toast.type]}`}>
      <div className="shrink-0 mt-0.5">{icons[toast.type]}</div>
      <div className="flex-1">
        {toast.title && <div className="text-sm font-black text-slate-900 mb-0.5 uppercase tracking-tight">{toast.title}</div>}
        <div className="text-sm text-slate-600 font-medium leading-relaxed">{toast.message}</div>
      </div>
      <button onClick={onClose} className="shrink-0 p-1 hover:bg-white/50 rounded-lg transition-colors">
        <X className="w-4 h-4 text-slate-400" />
      </button>
    </div>
  );
}

export function useToast() {
  const context = useContext(ToastContext);
  if (!context) throw new Error('useToast must be used within ToastProvider');
  return context;
}
