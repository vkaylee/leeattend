import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";

export function AuditLogHeader() {
    const { t } = useTranslation();
    return (
        <div>
            <Typography className="text-3xl font-bold tracking-tight text-slate-900 dark:text-white" variant="h2">{t("audit_logs.title")}</Typography>
            <Typography className="text-slate-500 dark:text-slate-400 mt-1" variant="p">
                {t("audit_logs.subtitle")}
            </Typography>
        </div>
    );
}
