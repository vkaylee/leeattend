import { CalendarDays } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { SettingsHeader } from "@/components/ui/settings/settings-header";

export function LeaveSettingsHeader() {
  const { t } = useTranslation();
  return (
    <SettingsHeader
      description={t("settings.leave.desc")}
      icon={<CalendarDays size={20} />}
      title={t("settings.leave.title")}
    />
  );
}
