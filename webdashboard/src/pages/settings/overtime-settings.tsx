import { Clock } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { SettingsHeader } from "@/components/ui/settings/settings-header";
import { useWorkspace } from "@/hooks/use-workspace";

import { OvertimeQuotasTab } from "./components/overtime/overtime-quotas-tab";

export default function OvertimeSettings() {
  const { t } = useTranslation();
  const { workspace: ws } = useWorkspace();

  if (!ws) return null;

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <SettingsHeader
        description={t("settings.categories.overtime_desc", { defaultValue: "Quản lý hạn mức tăng ca và luồng phê duyệt" })}
        icon={<Clock size={20} />}
        title={t("settings.categories.overtime", { defaultValue: "Cài đặt tăng ca" })}
      />

      <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 p-6 sm:p-8 shadow-2xs">
        <div className="animate-in fade-in duration-300">
          <OvertimeQuotasTab wsId={ws.id} />
        </div>
      </div>
    </div>
  );
}
