import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";

interface AssignRoleModalProps {
  assignError: string;
  isAssigning: boolean;
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (e: React.SyntheticEvent) => void;
  onUserIdChange: (value: string) => void;
  selectedRole: string | null;
  userIdInput: string;
}

export function AssignRoleModal({
  assignError,
  isAssigning,
  isOpen,
  onClose,
  onSubmit,
  onUserIdChange,
  selectedRole,
  userIdInput,
}: AssignRoleModalProps) {
  const { t } = useTranslation();

  return (
    <Modal
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      subtitle={t("system_roles.grant_privileges")}
      title={t("system_roles.modal.assign_title", { role: selectedRole })}
    >
      <form className="space-y-6" onSubmit={onSubmit}>
        <div className="space-y-2">
          <label className="text-sm font-bold text-slate-700 dark:text-slate-300">
            {t("system_roles.user_id_label")}
          </label>
          <input
            className="w-full px-4 py-3 rounded-xl bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 focus:ring-2 focus:ring-primary-500 focus:border-primary-500 transition-all font-mono text-sm"
            onChange={(e) => { onUserIdChange(e.target.value); }}
            placeholder="e.g. 123e4567-e89b-12d3-a456-426614174000"
            required
            type="text"
            value={userIdInput}
          />
          {assignError ? <Typography className="text-sm text-rose-500 font-medium mt-1" variant="p">{assignError}</Typography> : null}
        </div>
        
        <div className="flex justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800">
          <Button
            className="px-5 py-2.5 text-sm font-bold text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-xl transition-colors"
            onClick={onClose}
            type="button"
          >
            {t("common.cancel")}
          </Button>
          <Button
            className="px-5 py-2.5 text-sm font-bold text-white bg-primary-600 hover:bg-primary-700 rounded-xl transition-all disabled:opacity-50 shadow-md shadow-primary-200 dark:shadow-none"
            disabled={isAssigning || !userIdInput.trim()}
            type="submit"
          >
            {isAssigning ? t("system_roles.modal.assigning") : t("system_roles.modal.submit_btn")}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
