import React from "react";
import Modal from "@/components/ui/modal";
import { LeavePolicy } from "@/lib/api/types";
import { PolicyIdentitySection, AccrualLogicSection, SeniorityBonusSection } from "./leave-policy-components/policy-modal-fields";
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
    <Modal isOpen={isOpen} maxWidth="lg" onClose={onClose} title={policy ? "Edit Policy" : "Create Leave Policy"}>
      <form className="p-8 space-y-12 max-h-[85vh] overflow-y-auto custom-scrollbar" onSubmit={handleSubmit}>
        <PolicyIdentitySection formData={formData} leaveTypes={leaveTypes} setFormData={setFormData} />
        <AccrualLogicSection formData={formData} setFormData={setFormData} />
        <SeniorityBonusSection 
          addBonusMilestone={addBonusMilestone} 
          removeBonusMilestone={removeBonusMilestone} 
          seniorityBonus={seniorityBonus} 
          updateBonusMilestone={updateBonusMilestone} 
        />
        
        <div className="flex justify-end gap-4 pt-8 border-t border-slate-100 dark:border-slate-800 sticky bottom-0 bg-white dark:bg-slate-900 pb-2">
          <button className="px-8 py-3 text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] hover:text-slate-600 dark:hover:text-slate-300 dark:text-slate-600" onClick={onClose} type="button">Cancel</button>
          <button className="rounded-2xl bg-primary-600 px-10 py-4 text-xs font-black text-white uppercase tracking-[0.2em] shadow-2xl shadow-primary-200 hover:bg-primary-700 active:scale-95 transition-all" data-testid="save-policy-btn" disabled={isSaving} type="submit">
            {isSaving ? "Saving..." : (policy ? "Update Policy" : "Create Policy")}
          </button>
        </div>
      </form>
    </Modal>
  );
}
