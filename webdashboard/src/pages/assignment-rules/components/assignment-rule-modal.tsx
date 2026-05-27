import { Save } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { AssignmentRule, AssignmentRuleConditions } from "@/lib/api";

import { AssignmentRuleForm } from "./assignment-rule-form";

interface AssignmentRuleModalProps {
  conditions: AssignmentRuleConditions;
  departments: { id: string; name: string }[] | undefined;
  editingRule: AssignmentRule | null;
  isOpen: boolean;
  isPending: boolean;
  name: string;
  onClose: () => void;
  onSubmit: () => void;
  patternId: string;
  patterns: { id: string; name: string }[] | undefined;
  priority: number;
  setConditions: (conditions: AssignmentRuleConditions) => void;
  setName: (name: string) => void;
  setPatternId: (id: string) => void;
  setPriority: (priority: number) => void;
}

export const AssignmentRuleModal: React.FC<AssignmentRuleModalProps> = ({
  conditions,
  departments,
  editingRule,
  isPending,
  isOpen,
  name,
  onClose,
  onSubmit,
  patternId,
  patterns,
  priority,
  setConditions,
  setName,
  setPatternId,
  setPriority,
}) => {
  const { t } = useTranslation();

  return (
    <Modal
      isOpen={isOpen}
      maxWidth="2xl"
      onClose={onClose}
      subtitle={t("rules.form_subtitle", "Define when this pattern should be applied")}
      title={editingRule ? t("rules.edit_title", "Edit Rule") : t("rules.create_title", "Create New Rule")}
    >
      <div className="space-y-6">
        <AssignmentRuleForm
          conditions={conditions}
          departments={departments}
          name={name}
          patternId={patternId}
          patterns={patterns}
          priority={priority}
          setConditions={setConditions}
          setName={setName}
          setPatternId={setPatternId}
          setPriority={setPriority}
        />

        <div className="flex gap-3 pt-2">
          <Button className="flex-1" onClick={onClose} size="lg" variant="ghost">
            {t("common.cancel")}
          </Button>
          <Button className="flex-1" 
            icon={<Save className="w-4 h-4" />} 
            loading={isPending} 
            onClick={() => { onSubmit(); }} 
            size="lg" 
            variant="primary"
          >
            {editingRule ? t("common.save_changes") : t("common.create")}
          </Button>
        </div>
      </div>
    </Modal>
  );
};
