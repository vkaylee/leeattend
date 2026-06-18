import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";

export function AuditLogHeader() {
    const { t } = useTranslation();
    return (
        <div>
            <Typography className="text-2xl font-bold tracking-tight text-slate-850 dark:text-slate-100 normal-case mb-0.5" variant="h3">{t("audit_logs.title")}</Typography>
            <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium normal-case tracking-normal" variant="small">
                {t("audit_logs.subtitle")}
            </Typography>
        </div>
    );
}
