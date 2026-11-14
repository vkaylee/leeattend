import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";

export interface RestoreTypeModalProps {
  isRestoring: boolean;
  name: string;
  onCancel: () => void;
  onConfirm: () => void;
  open: boolean;
}

export function RestoreTypeModal({
  open,
  name,
  isRestoring,
  onCancel,
  onConfirm,
}: RestoreTypeModalProps) {
  const { t } = useTranslation();
  if (!open) return null;

  return (
    <Modal
      isOpen={true}
      onClose={onCancel}
      title={t("settings.leave.type_manager.restore_dialog_title", "Restore Leave Type")}
    >
      <div className="p-8">
        <Typography className="text-sm" variant="p">
          {t(
            "settings.leave.type_manager.restore_dialog_msg",
            "A leave type with this code was retired. Restore \"{{name}}\" instead of creating a new one?",
            { name },
          )}
        </Typography>
        <div className="mt-8 flex justify-end gap-3">
          <Button
            className="px-6 py-2 text-xs text-slate-400 dark:text-slate-500 hover:bg-transparent"
            onClick={onCancel}
            variant="ghost"
          >
            {t("common.cancel", "Cancel")}
          </Button>
          <Button
            className="text-[10px] px-6 py-2.5 h-auto rounded-xl"
            disabled={isRestoring}
            onClick={onConfirm}
            variant="primary"
          >
            {t("settings.leave.type_manager.restore_btn", "Restore")}
          </Button>
        </div>
      </div>
    </Modal>
  );
}

export default RestoreTypeModal;
