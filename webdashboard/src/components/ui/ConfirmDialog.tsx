import React from 'react';
import Modal from './Modal';
import Button from './Button';
import { AlertTriangle, Info, Trash2 } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export interface ConfirmDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void | Promise<void>;
  title: string;
  message: string | React.ReactNode;
  confirmLabel?: string;
  cancelLabel?: string;
  variant?: 'danger' | 'warning' | 'info';
  isLoading?: boolean;
}

export default function ConfirmDialog({
  isOpen,
  onClose,
  onConfirm,
  title,
  message,
  confirmLabel,
  cancelLabel,
  variant = 'danger',
  isLoading = false
}: ConfirmDialogProps) {
  const { t } = useTranslation();

  const handleConfirm = async () => {
    await onConfirm();
  };

  const getIcon = () => {
    switch (variant) {
      case 'danger': return <Trash2 size={24} />;
      case 'warning': return <AlertTriangle size={24} />;
      case 'info': return <Info size={24} />;
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={title}
      icon={getIcon()}
      maxWidth="sm"
    >
      <div className="space-y-6">
        <div className="text-slate-600 dark:text-slate-300 text-sm font-medium">
          {message}
        </div>
        
        <div className="flex items-center justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800">
          <Button
            variant="outline"
            onClick={onClose}
            disabled={isLoading}
          >
            {cancelLabel || t('common.cancel', { defaultValue: 'Cancel' })}
          </Button>
          <Button
            variant={variant === 'danger' ? 'destructive' : 'primary'}
            onClick={handleConfirm}
            loading={isLoading}
          >
            {confirmLabel || t('common.confirm', { defaultValue: 'Confirm' })}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
