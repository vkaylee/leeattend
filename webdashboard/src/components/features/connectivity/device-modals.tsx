import { Power, AlertTriangle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
import { Device } from "@/lib/api";
import { cn } from "@/lib/utils";

interface DeviceModalsProps {
  device: Device;
  handleDeleteConfirm: () => void;
  handleToggleConfirm: () => void;
  onCloseDelete: () => void;
  onCloseToggle: () => void;
  showDeleteModal: boolean;
  showToggleModal: boolean;
}

export function DeviceModals({
  device,
  showToggleModal,
  onCloseToggle,
  showDeleteModal,
  onCloseDelete,
  handleToggleConfirm,
  handleDeleteConfirm,
}: DeviceModalsProps) {
  const { t } = useTranslation();

  return (
    <>
      <Modal icon={<Power size={20} />} isOpen={showToggleModal} maxWidth="sm" onClose={onCloseToggle} testId={`toggle-modal-${device.id}`} title={t(device.is_enabled ? "devices.modal.disable_title" : "devices.modal.enable_title")}>
        <Typography className="text-sm text-slate-600 dark:text-slate-300 mb-6" variant="p">{t(device.is_enabled ? "devices.confirm_disable" : "devices.confirm_enable", { name: device.name })}</Typography>
        <div className="flex justify-end gap-3">
          <button className="px-4 py-2 text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 rounded-xl transition-colors" data-testid={`toggle-cancel-${device.id}`} onClick={onCloseToggle}>{t("common.cancel")}</button>
          <button className={cn("px-4 py-2 text-xs font-bold uppercase tracking-widest rounded-xl transition-colors", device.is_enabled ? "bg-rose-600 text-white" : "bg-emerald-600 text-white")} data-testid={`toggle-confirm-${device.id}`} onClick={handleToggleConfirm}>
            {device.is_enabled ? t("common.disable") : t("common.enable")}
          </button>
        </div>
      </Modal>

      <Modal icon={<AlertTriangle size={20} />} isOpen={showDeleteModal} maxWidth="sm" onClose={onCloseDelete} testId={`delete-modal-${device.id}`} title={t("devices.modal.delete_title")}>
        <Typography className="text-sm text-slate-600 dark:text-slate-300 mb-6" variant="p">{t("devices.toast.delete_confirm", { name: device.name })}</Typography>
        <div className="flex justify-end gap-3">
          <button className="px-4 py-2 text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 rounded-xl transition-colors" data-testid={`delete-cancel-${device.id}`} onClick={onCloseDelete}>{t("common.cancel")}</button>
          <button className="px-4 py-2 text-xs font-bold uppercase tracking-widest bg-rose-600 text-white rounded-xl transition-colors" data-testid={`delete-confirm-${device.id}`} onClick={handleDeleteConfirm}>{t("devices.table.delete")}</button>
        </div>
      </Modal>
    </>
  );
}
