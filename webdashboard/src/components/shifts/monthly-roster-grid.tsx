import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import { vi, enUS, zhCN, type Locale } from "date-fns/locale";

import { useShifts } from "@/pages/shifts/hooks/use-shifts";

import { useMonthlyRoster } from "./hooks/use-monthly-roster";
import { GridScrollButtons } from "./roster/grid-scroll-buttons";
import {
  NoEmployeesState,
  NoResultsState,
  NoSelectionState,
  RosterErrorState,
  RosterSkeleton,
} from "./roster/roster-empty-states";
import { RosterGridTable } from "./roster/roster-grid-table";
import { ShiftPickerModal } from "./shift-picker-modal";

const dateLocales: Partial<Record<string, Locale>> = {
  vi,
  en: enUS,
  zh: zhCN,
};

interface MonthlyRosterGridProps {
  hook: ReturnType<typeof useShifts>;
  monthlyRoster: ReturnType<typeof useMonthlyRoster>;
}

export function MonthlyRosterGrid({
  hook,
  monthlyRoster,
}: MonthlyRosterGridProps) {
  const { i18n, t } = useTranslation();
  const currentLocale = dateLocales[i18n.language] ?? vi;

  const gridContainerRef = useRef<HTMLDivElement | null>(null);
  const [showLeftArrow, setShowLeftArrow] = useState(false);
  const [showRightArrow, setShowRightArrow] = useState(false);

  const {
    monthDays,
    isMonthLoading,
    isMonthError,
    refetchMonthSchedules,
    shifts,
    employees,
    isEmployeesLoading,
    isEmployeesError,
    refetchEmployees,
  } = hook;

  const {
    selectedCells,
    isShiftPickerOpen,
    setIsShiftPickerOpen,
    selectedEmployeeIds,
    filteredSchedules,
    isDragging,
    handleCellPointerDown,
    handleCellPointerEnter,
    handleApplyShift,
    rowStatus,
  } = monthlyRoster;

  // ── Scroll arrow visibility ────────────────────────────────────────

  const updateArrowVisibility = useCallback(() => {
    if (gridContainerRef.current) {
      const { clientWidth, scrollLeft, scrollWidth } =
        gridContainerRef.current;
      setShowLeftArrow(scrollLeft > 10);
      setShowRightArrow(scrollLeft < scrollWidth - clientWidth - 10);
    }
  }, []);

  const scrollGrid = useCallback(
    (direction: "left" | "right") => {
      if (gridContainerRef.current) {
        const scrollAmount = 300;
        gridContainerRef.current.scrollBy({
          behavior: "smooth",
          left: direction === "left" ? -scrollAmount : scrollAmount,
        });
      }
    },
    [],
  );

  const handleScrollLeft = useCallback(() => { scrollGrid("left"); }, [scrollGrid]);
  const handleScrollRight = useCallback(
    () => { scrollGrid("right"); },
    [scrollGrid],
  );

  // ── Resize/scroll listeners ─────────────────────────────────────────

  useEffect(() => {
    const container = gridContainerRef.current;
    if (!container) return;

    updateArrowVisibility();
    container.addEventListener("scroll", updateArrowVisibility);

    const resizeObserver = new ResizeObserver(() => {
      updateArrowVisibility();
    });
    resizeObserver.observe(container);

    return () => {
      container.removeEventListener("scroll", updateArrowVisibility);
      resizeObserver.disconnect();
    };
  }, [isMonthLoading, isEmployeesLoading, filteredSchedules.length, updateArrowVisibility]);

  // ── Retry handler ───────────────────────────────────────────────────

  const handleRetry = useCallback(() => {
    void refetchMonthSchedules();
    void refetchEmployees();
  }, [refetchMonthSchedules, refetchEmployees]);

  // ── Shift picker close handler ──────────────────────────────────────

  const handleClosePicker = useCallback(() => {
    setIsShiftPickerOpen(false);
  }, [setIsShiftPickerOpen]);

  // ── Derived: target dates for picker ────────────────────────────────

  const targetDates = useMemo(
    () => Array.from(selectedCells).map((c) => c.split(":")[1]),
    [selectedCells],
  );

  // ── Render content ──────────────────────────────────────────────────

  const renderContent = () => {
    if (isMonthError || isEmployeesError) {
      return <RosterErrorState onRetry={handleRetry} />;
    }

    if (isMonthLoading || isEmployeesLoading || !employees) {
      return <RosterSkeleton />;
    }

    if (employees.length === 0) {
      return <NoEmployeesState />;
    }

    if (selectedEmployeeIds.length === 0) {
      return <NoSelectionState />;
    }

    if (filteredSchedules.length === 0) {
      return <NoResultsState />;
    }

    return (
      <RosterGridTable
        containerRef={gridContainerRef}
        currentLocale={currentLocale}
        employeesLabel={t("shifts.monthly.all_employees")}
        isDragging={isDragging}
        monthDays={monthDays}
        monthSchedules={filteredSchedules}
        onCellPointerDown={handleCellPointerDown}
        onCellPointerEnter={handleCellPointerEnter}
        rowStatus={rowStatus}
        selectedCells={selectedCells}
        shifts={shifts}
      />
    );
  };

  return (
    <div className="space-y-4">
      {isShiftPickerOpen ? (
        <ShiftPickerModal
          isOpen={isShiftPickerOpen}
          onClose={handleClosePicker}
          onSelectShift={handleApplyShift}
          shifts={shifts}
          targetDates={targetDates}
        />
      ) : null}

      <div className="bg-white dark:bg-slate-900 rounded-[28px] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden relative group/grid">
        <GridScrollButtons
          onScrollLeft={handleScrollLeft}
          onScrollRight={handleScrollRight}
          showLeft={showLeftArrow}
          showRight={showRightArrow}
        />

        {renderContent()}
      </div>
    </div>
  );
}
