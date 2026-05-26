import { Power, AlertTriangle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
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
          <Button className="h-auto px-4 py-2 text-xs text-slate-500 dark:text-slate-400 rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors shadow-none" data-testid={`toggle-cancel-${device.id}`} onClick={onCloseToggle} variant="ghost">{t("common.cancel")}</Button>
          <Button className={cn("h-auto px-4 py-2 text-xs rounded-xl", device.is_enabled ? "bg-rose-600 text-white hover:bg-rose-700" : "bg-emerald-600 text-white hover:bg-emerald-700")} data-testid={`toggle-confirm-${device.id}`} onClick={handleToggleConfirm} variant={device.is_enabled ? "destructive" : "primary"}>
            {device.is_enabled ? t("common.disable") : t("common.enable")}
          </Button>
        </div>
      </Modal>

      <Modal icon={<AlertTriangle size={20} />} isOpen={showDeleteModal} maxWidth="sm" onClose={onCloseDelete} testId={`delete-modal-${device.id}`} title={t("devices.modal.delete_title")}>
        <Typography className="text-sm text-slate-600 dark:text-slate-300 mb-6" variant="p">{t("devices.toast.delete_confirm", { name: device.name })}</Typography>
        <div className="flex justify-end gap-3">
          <Button className="h-auto px-4 py-2 text-xs text-slate-500 dark:text-slate-400 rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors shadow-none" data-testid={`delete-cancel-${device.id}`} onClick={onCloseDelete} variant="ghost">{t("common.cancel")}</Button>
          <Button className="h-auto px-4 py-2 text-xs bg-rose-600 text-white hover:bg-rose-700 rounded-xl transition-colors" data-testid={`delete-confirm-${device.id}`} onClick={handleDeleteConfirm} variant="destructive">{t("devices.table.delete")}</Button>
        </div>
      </Modal>
    </>
  );
}

