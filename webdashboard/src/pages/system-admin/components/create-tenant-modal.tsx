import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import {
  createSystemTenant,
  type CreateTenantRequestPayload,
} from "@/lib/api/system";

interface CreateTenantModalProps {
  isOpen: boolean;
  isPending: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export function CreateTenantModal({
  isOpen,
  isPending,
  onClose,
  onSuccess,
}: CreateTenantModalProps) {
  const { t } = useTranslation(["common", "system_roles"]);
  const toast = useToast();

  const [name, setName] = useState("");
  const [databaseUrl, setDatabaseUrl] = useState("");

  const handleSubmit = async (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!name) return;

    const payload: CreateTenantRequestPayload = {
      name,
      database_url: databaseUrl || undefined,
    };

    try {
      await createSystemTenant(payload);
      toast.success(t("system_admin.tenants.messages.create_success"));
      resetForm();
      onSuccess();
    } catch (error) {
      toast.error((error as Error).message);
    }
  };

  const resetForm = () => {
    setName("");
    setDatabaseUrl("");
  };

  const handleClose = () => {
    resetForm();
    onClose();
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleClose}
      subtitle="Create a new workspace tenant. Leave Database URL empty for automatic provisioning."
      title={t("system_admin.tenants.modal.create_title")}
    >
      <form onSubmit={(e) => { void handleSubmit(e); }}>
        <div className="grid gap-6 py-4">
          <div className="space-y-2">
            <label className="text-sm font-medium text-slate-700 dark:text-slate-300" htmlFor="name">
              {t("system_admin.tenants.modal.name_label", { defaultValue: "Workspace Name" })} <span className="text-red-500">*</span>
            </label>
            <input
              className="flex h-10 w-full rounded-md border border-slate-300 dark:border-slate-700 bg-transparent px-3 py-2 text-sm placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent dark:text-white"
              id="name"
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => { setName(e.target.value); }}
              placeholder="My Workspace"
              required
              value={name}
            />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium text-slate-700 dark:text-slate-300" htmlFor="databaseUrl">
              {t("system_admin.tenants.modal.database_url_label", { defaultValue: "Database URL (BYOD only)" })}
            </label>
            <input
              className="flex h-10 w-full rounded-md border border-slate-300 dark:border-slate-700 bg-transparent px-3 py-2 text-sm placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent dark:text-white"
              id="databaseUrl"
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => { setDatabaseUrl(e.target.value); }}
              placeholder="postgres://user:password@host:5432/db"
              type="text"
              value={databaseUrl}
            />
          </div>
        </div>
        <div className="flex justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800 mt-2">
          <Button
            onClick={handleClose}
            type="button"
            variant="outline"
          >
            {t("common.cancel", { defaultValue: "Cancel" })}
          </Button>
          <Button
            disabled={isPending || !name}
            loading={isPending}
            type="submit"
            variant="primary"
          >
            {isPending
              ? t("system_admin.tenants.modal.creating")
              : t("system_admin.tenants.modal.submit_btn")}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
