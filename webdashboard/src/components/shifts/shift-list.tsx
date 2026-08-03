import { Clock, Search } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import ResponsiveTable from "@/components/ui/responsive-table";

import { useDateTimeFormatter } from "../../hooks/use-datetime";
import type { Shift } from "../../lib/api";
import { isNightShift, shiftMatchesQuery } from "./shift-utils";

import { ShiftMobileCard } from "./shift-list/shift-mobile-card";
import { ShiftListTable } from "./shift-list/shift-list-table";
import { ShiftListToolbar } from "./shift-list/shift-list-toolbar";
import type { ListFilter } from "./shift-list/types";

// ── Types ──────────────────────────────────────────────────────────────

interface ShiftListProps {
  shifts: Shift[] | undefined;
  onEdit: (shift: Shift) => void;
  onDelete?: (shift: Shift) => void;
  onViewHistory?: (shift: Shift) => void;
}

// ── ShiftList ──────────────────────────────────────────────────────────

export function ShiftList({ shifts, onEdit, onDelete, onViewHistory }: ShiftListProps) {
  const { t } = useTranslation();
  const { formatTime } = useDateTimeFormatter();

  const [searchQuery, setSearchQuery] = useState("");
  const [activeFilter, setActiveFilter] = useState<ListFilter>("all");

  const filteredShifts = useMemo(() => {
    const all = shifts ?? [];
    let result = all;

    if (searchQuery.trim()) {
      result = result.filter((s) => shiftMatchesQuery(s, searchQuery));
    }

    switch (activeFilter) {
      case "active":
        result = result.filter((s) => s.is_active);
        break;
      case "inactive":
        result = result.filter((s) => !s.is_active);
        break;
      case "night":
        result = result.filter((s) => isNightShift(s));
        break;
      default:
        break;
    }

    return result;
  }, [shifts, searchQuery, activeFilter]);

  const isFiltered = searchQuery.trim() !== "" || activeFilter !== "all";

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm animate-in fade-in slide-in-from-bottom-4 duration-700">
      {/* ── Header ──────────────────────────────────────────────────── */}
      <div className="p-8 border-b border-slate-50 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30">
        <ShiftListToolbar
          activeFilter={activeFilter}
          filteredCount={filteredShifts.length}
          onFilterChange={setActiveFilter}
          onSearchChange={setSearchQuery}
          searchPlaceholder={t("shifts.picker.search_placeholder")}
          searchQuery={searchQuery}
        />
      </div>

      {/* ── Table / Mobile Cards ────────────────────────────────────── */}
      <ResponsiveTable<Shift>
        data={filteredShifts}
        emptyState={{
          icon: isFiltered
            ? <Search className="h-10 w-10" />
            : <Clock className="h-10 w-10" />,
          title: isFiltered
            ? t("shifts.picker.no_results_title")
            : t("shifts.messages.no_shifts"),
          description: isFiltered
            ? t("shifts.picker.no_results_desc")
            : `${t("shifts.directory_title")} is empty`,
        }}
        getCardKey={(shift) => shift.id}
        renderCard={(shift) => (
          <ShiftMobileCard
            onDelete={onDelete}
            onEdit={onEdit}
            onViewHistory={onViewHistory}
            shift={shift}
          />
        )}
        renderDesktopTable={() => (
          <ShiftListTable
            formatTime={formatTime}
            isFiltered={isFiltered}
            onDelete={onDelete}
            onEdit={onEdit}
            onViewHistory={onViewHistory}
            shifts={filteredShifts}
          />
        )}
      />
    </div>
  );
}
