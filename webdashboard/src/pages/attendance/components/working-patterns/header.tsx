import { CalendarDays, Plus, Users } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import PageHeader from "@/components/ui/page-header";

interface PatternHeaderProps {
  onBulkAssignClick: () => void;
  onCreateClick: () => void;
}

export function PatternHeader({
  onBulkAssignClick,
  onCreateClick,
}: PatternHeaderProps) {
  const { t } = useTranslation();

  return (
    <PageHeader
      actions={
        <>
          <Button
            icon={<Users size={18} />}
            onClick={() => {
              onBulkAssignClick();
            }}
            variant="outline"
          >
            {t("patterns.bulk_assign_btn").toUpperCase()}
          </Button>
          <Button
            icon={<Plus size={18} />}
            onClick={() => {
              onCreateClick();
            }}
          >
            {t("patterns.create_btn").toUpperCase()}
          </Button>
        </>
      }
      data-testid="patterns-title"
      icon={<CalendarDays size={24} />}
      subtitle={t("patterns.subtitle")}
      title={t("patterns.title")}
    />
  );
}
