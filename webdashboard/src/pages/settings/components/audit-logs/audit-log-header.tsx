import React from "react";
import { useTranslation } from "react-i18next";

export function AuditLogHeader() {
    const { t } = useTranslation();
    return (
        <div>
            <h2 className="text-3xl font-bold tracking-tight text-slate-900 dark:text-white">{t("audit_logs.title")}</h2>
            <p className="text-slate-500 dark:text-slate-400 mt-1">
                {t("audit_logs.subtitle")}
            </p>
        </div>
    );
}
