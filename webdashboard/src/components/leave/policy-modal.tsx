import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { LeavePolicy } from "@/lib/api/types";

import {
  AccrualLogicSection,
  PolicyIdentitySection,
  SeniorityBonusSection,
} from "./leave-policy-components/policy-modal-fields";
import { usePolicyModal } from "./leave-policy-components/use-policy-modal";

export interface PolicyModalProps {
  isOpen: boolean;
  isSaving: boolean;
  leaveTypes: { id: string; name: string }[];
  onClose: () => void;
  onSave: (data: Partial<LeavePolicy>) => void;
  policy: LeavePolicy | null;
}

export function PolicyModal({ isOpen, onClose, leaveTypes, policy, onSave, isSaving }: PolicyModalProps) {
  const { t } = useTranslation();
  const {
    formData,
    setFormData,
    seniorityBonus,
    addBonusMilestone,
    removeBonusMilestone,
    updateBonusMilestone,
    handleSubmit
  } = usePolicyModal(policy, leaveTypes, onSave);

  return (
    <Modal isOpen={isOpen} maxWidth="lg" onClose={onClose} title={policy ? t("settings.leave.policy_modal.edit_title", "Edit Policy") : t("settings.leave.policy_modal.create_title", "Create Leave Policy")}>
      <form className="space-y-6" onSubmit={handleSubmit}>
        <div className="space-y-4">
          <PolicyIdentitySection formData={formData} leaveTypes={leaveTypes} setFormData={setFormData} />
          <AccrualLogicSection formData={formData} setFormData={setFormData} />
          <SeniorityBonusSection 
            addBonusMilestone={addBonusMilestone} 
            removeBonusMilestone={removeBonusMilestone} 
            seniorityBonus={seniorityBonus} 
            updateBonusMilestone={updateBonusMilestone} 
          />
        </div>
        
        <div className="flex justify-end gap-4 pt-6 border-t border-slate-100 dark:border-slate-800 mt-6">
          <Button
            className="px-8 py-3 text-xs text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 tracking-[0.2em]"
            onClick={onClose}
            size="md"
            type="button"
            variant="ghost"
          >
            {t("common.cancel", "Cancel")}
          </Button>
          <Button
            className="shadow-primary-500/20 px-10 py-4 text-xs font-black tracking-[0.2em]"
            data-testid="save-policy-btn"
            disabled={isSaving}
            size="md"
            type="submit"
            variant="primary"
          >
            {isSaving ? t("settings.leave.policy_modal.saving", "Saving...") : (policy ? t("settings.leave.policy_modal.update_btn", "Update Policy") : t("settings.leave.policy_modal.create_btn", "Create Policy"))}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
