import { AlertTriangle, Info, Trash2 } from 'lucide-react';
import React, { memo, useCallback } from 'react';
import { useTranslation } from 'react-i18next';

import Button from './button';
import Modal from './modal';

export interface ConfirmDialogProps {
  cancelLabel?: string;
  confirmLabel?: string;
  isLoading?: boolean;
  isOpen: boolean;
  message: string | React.ReactNode;
  onClose: () => void;
  onConfirm: () => void | Promise<void>;
  title: string;
  variant?: 'danger' | 'warning' | 'info';
}

const ConfirmDialog = memo(function ConfirmDialog({
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

  const handleConfirm = useCallback(async () => {
    await onConfirm();
  }, [onConfirm]);

  const getIcon = useCallback(() => {
    switch (variant) {
      case 'danger': return <Trash2 size={24} />;
      case 'warning': return <AlertTriangle size={24} />;
      case 'info': return <Info size={24} />;
    }
  }, [variant]);

  return (
    <Modal
      icon={getIcon()}
      isOpen={isOpen}
      maxWidth="sm"
      onClose={onClose}
      title={title}
    >
      <div className="space-y-6">
        <div className="text-slate-600 dark:text-slate-300 text-sm font-medium">
          {message}
        </div>

        <div className="flex items-center justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800">
          <Button
            disabled={isLoading}
            onClick={onClose}
            variant="outline"
          >
            {cancelLabel ?? t('common.cancel', { defaultValue: 'Cancel' })}
          </Button>
          <Button
            loading={isLoading}
            onClick={handleConfirm}
            variant={variant === 'danger' ? 'destructive' : 'primary'}
          >
            {confirmLabel ?? t('common.confirm', { defaultValue: 'Confirm' })}
          </Button>
        </div>
      </div>
    </Modal>
  );
});

ConfirmDialog.displayName = "ConfirmDialog";

export default ConfirmDialog;
