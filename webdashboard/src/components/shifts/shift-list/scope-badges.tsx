import { Building2, MapPin } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { Shift } from "@/lib/api";

// ── Scope Badges ────────────────────────────────────────────────────────

interface ScopeBadgesProps {
  shift: Shift;
}

/**
 * Renders scope badges showing whether a shift is scoped to a location,
 * employer, or is global.
 */
export function ScopeBadges({ shift }: ScopeBadgesProps) {
  const { t } = useTranslation();
  const hasLocation = !!shift.location_id;
  const hasEmployer = !!shift.employer_id;

  if (!hasLocation && !hasEmployer) {
    return (
      <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-lg bg-slate-100 dark:bg-slate-800 text-[9px] font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500">
        <Building2 size={10} /> {t("common.global")}
      </span>
    );
  }

  return (
    <div className="flex items-center gap-1.5">
      {hasLocation ? (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-lg bg-blue-50 dark:bg-blue-950/40 text-[9px] font-bold uppercase tracking-wider text-blue-600 dark:text-blue-400">
          <MapPin size={10} /> {t("common.office_branch")}
        </span>
      ) : null}
      {hasEmployer ? (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-lg bg-violet-50 dark:bg-violet-950/40 text-[9px] font-bold uppercase tracking-wider text-violet-600 dark:text-violet-400">
          <Building2 size={10} /> {t("common.employer")}
        </span>
      ) : null}
    </div>
  );
}
