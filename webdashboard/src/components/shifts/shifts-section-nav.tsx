import { useLocation, useNavigate } from "@tanstack/react-router";
import { CalendarRange, LayoutGrid, Zap } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { ROUTES } from "@/lib/routes";

type Section = "roster" | "patterns" | "rules";

const baseClass =
  "flex items-center gap-2 px-6 py-1.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all";
const activeClass =
  "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800";
const inactiveClass =
  "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200";

export function ShiftsSectionNav() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { pathname, searchStr } = useLocation();

  // Determine active section from URL.
  // Can't use <Link> for rules segments — TanStack Router considers both
  // /shifts/rules?tab=patterns and /shifts/rules?tab=rules as exact matches
  // for the same path, auto-activating both <Link> children simultaneously.
  // Manual navigation avoids ambiguous query-string route matches.
  const isOnRules = pathname.startsWith(ROUTES.SHIFTS_RULES);
  const tab = isOnRules
    ? new URLSearchParams(searchStr).get("tab") ?? "patterns"
    : null;

  const active: Section =
    isOnRules && tab === "rules" ? "rules"
    : isOnRules ? "patterns"
    : "roster";

  return (
    <div className="flex items-center gap-1 bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50">
      <Button
        className={`${baseClass} ${active === "roster" ? activeClass : inactiveClass}`}
        onClick={() => void navigate({ to: ROUTES.SHIFTS, search: { tab: "weekly-roster", view: "week" } })}
        size="sm"
        type="button"
        variant="ghost"
      >
        <LayoutGrid size={14} /> {t("shifts.title")}
      </Button>
      <Button
        className={`${baseClass} ${active === "patterns" ? activeClass : inactiveClass}`}
        onClick={() => void navigate({ to: ROUTES.SHIFTS_RULES, search: { tab: "patterns" } })}
        size="sm"
        type="button"
        variant="ghost"
      >
        <CalendarRange size={14} /> {t("patterns.title", "Working Patterns")}
      </Button>
      <Button
        className={`${baseClass} ${active === "rules" ? activeClass : inactiveClass}`}
        onClick={() => void navigate({ to: ROUTES.SHIFTS_RULES, search: { tab: "rules" } })}
        size="sm"
        type="button"
        variant="ghost"
      >
        <Zap size={14} /> {t("rules.title", "Assignment Rules")}
      </Button>
    </div>
  );
}
