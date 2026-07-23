import React from "react";
import { useTranslation } from "react-i18next";

import type { CustomFieldDefinition } from "@/lib/api/types";

import Sheet from "@/components/ui/sheet";

import { CustomFieldFormContent } from "./custom-field-form-content";

interface CustomFieldDrawerProps {
  editingField: CustomFieldDefinition | null;
  isOpen: boolean;
  onClose: () => void;
}

export default function CustomFieldDrawer({
  isOpen,
  onClose,
  editingField,
}: CustomFieldDrawerProps) {
  const { t } = useTranslation("custom_fields");

  return (
    <Sheet
      isOpen={isOpen}
      onClose={onClose}
      title={editingField ? t("edit_field") : t("create_field")}
    >
      {isOpen ? (
        <CustomFieldFormContent editingField={editingField} onClose={onClose} />
      ) : null}
    </Sheet>
  );
}
