import React, { useState, useCallback } from 'react';

import ConfirmDialog, { ConfirmDialogProps } from '@/components/ui/confirm-dialog';

type ConfirmOptions = Omit<ConfirmDialogProps, 'isOpen' | 'onClose' | 'onConfirm'>;

export function useConfirm() {
  const [isOpen, setIsOpen] = useState(false);
  const [options, setOptions] = useState<ConfirmOptions | null>(null);
  const [resolver, setResolver] = useState<(value: boolean) => void>();

  const confirm = useCallback((options_: ConfirmOptions): Promise<boolean> => {
    setOptions(options_);
    setIsOpen(true);
    return new Promise((resolve) => {
      setResolver(() => resolve);
    });
  }, []);

  const handleClose = useCallback(() => {
    setIsOpen(false);
    if (resolver) {
      resolver(false);
    }
  }, [resolver]);

  const handleConfirm = useCallback(() => {
    setIsOpen(false);
    if (resolver) {
      resolver(true);
    }
  }, [resolver]);

  const ConfirmationDialog = useCallback(() => {
    if (!options) return null;
    return (
      <ConfirmDialog
        isOpen={isOpen}
        onClose={handleClose}
        onConfirm={handleConfirm}
        {...options}
      />
    );
  }, [isOpen, options, handleClose, handleConfirm]);

  return { confirm, ConfirmationDialog };
}
