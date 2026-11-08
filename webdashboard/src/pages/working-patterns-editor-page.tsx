import { useBlocker, useNavigate, useParams } from "@tanstack/react-router";
import {
  Clock,
  ArrowLeft,
  Save,
  Trash2,
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import ConfirmDialog from "@/components/ui/confirm-dialog";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useCalendarPreview } from "@/hooks/use-calendar-preview";
import { useWorkingPatternForm, isCycleLengthInvalid, convertGridToDetails } from "@/hooks/use-working-pattern-form";
import { cn } from "@/lib/utils";
import { WorkingPatternCalendarPreview } from "@/pages/attendance/components/working-pattern-calendar-preview";
import { WorkingPatternDesignEditor } from "@/pages/attendance/components/working-pattern-design-editor";
import { WorkingPatternSettingsSidebar } from "@/pages/attendance/components/working-pattern-settings-sidebar";

/* eslint-disable complexity, max-lines-per-function */
export default function WorkingPatternsEditorPage() {
  const { t } = useTranslation();
  const { patternId } = useParams({ strict: false });
  const navigate = useNavigate();
  const toast = useToast();
  const [isClearConfirmOpen, setIsClearConfirmOpen] = useState(false);

  const {
    isEditMode,
    isLoadingPatterns,
    shifts,
    name,
    setName,
    description,
    setDescription,
    patternType,
    handlePatternTypeChange,
    cycleLengthDays,
    handleCycleLengthChange,
    grid,
    addShiftToDay,
    removeShiftFromDay,
    clearAllDays,
    prefillStandard,
    createPattern,
    updatePattern,
    isDirty,
    setIsDirty,
    isSaving,
  } = useWorkingPatternForm(patternId);

  const handleSubmit = (e: React.SyntheticEvent) => {
    e.preventDefault();
    if (!name.trim()) {
      toast.error(t("patterns.validation_name_required"));
      return;
    }
    if (isCycleLengthInvalid(patternType, cycleLengthDays)) {
      toast.error(t("patterns.validation_cycle_length_invalid"));
      return;
    }

    const details = convertGridToDetails(grid);

    const payload = {
      name: name.trim(),
      description: description.trim() || undefined,
      pattern_type: patternType,
      cycle_length_days: patternType === "rotation" ? cycleLengthDays : undefined,
      details,
    };

    if (isEditMode) {
      updatePattern(payload);
    } else {
      createPattern(payload);
    }
  };

  const {
    activeViewMode,
    setActiveViewMode,
    previewAnchorDate,
    setPreviewAnchorDate,
    previewYear,
    previewMonth,
    handlePreviousMonth,
    handleNextMonth,
    generateCalendarCells,
  } = useCalendarPreview();

  const blocker = useBlocker({
    shouldBlockFn: () => isDirty,
    withResolver: true,
  });

  const handleConfirmClear = () => {
    clearAllDays();
    setIsClearConfirmOpen(false);
  };


  if (isEditMode && isLoadingPatterns) {
    return (
      <div className="p-12 space-y-8 animate-pulse max-w-7xl mx-auto">
        <div className="h-10 w-64 bg-slate-200 dark:bg-slate-800 rounded-full" />
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-10">
          <div className="h-96 bg-slate-200 dark:bg-slate-800 rounded-2xl sm:rounded-[2.5rem]" />
          <div className="lg:col-span-2 h-[30rem] bg-slate-200 dark:bg-slate-800 rounded-2xl sm:rounded-[2.5rem]" />
        </div>
      </div>
    );
  }

  const daysOfWeekFull = [
    t("days.sunday"),
    t("days.monday"),
    t("days.tuesday"),
    t("days.wednesday"),
    t("days.thursday"),
    t("days.friday"),
    t("days.saturday"),
  ];

  const gridRows = patternType === "weekly"
    ? daysOfWeekFull
    : Array.from({ length: cycleLengthDays }, (_, i) => t("patterns.form.day_index", { index: i + 1 }));

  return (
    <form className="max-w-7xl mx-auto space-y-6 pb-20 animate-in fade-in duration-500" onSubmit={handleSubmit}>
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
        <div className="flex items-center gap-4">
          <Button className="p-3 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-850 hover:bg-slate-50 dark:hover:bg-slate-800 rounded-2xl shadow-sm text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200 active:scale-95 transition-all"
            onClick={() => {
              void navigate({ to: "/shifts/rules", search: { tab: "patterns" } });
            }}
            type="button"
            variant="ghost"
          >
            <ArrowLeft size={18} />
          </Button>
          <div>
            <Typography className="text-2xl font-bold tracking-tight text-slate-850 dark:text-slate-100 normal-case mb-0.5" variant="h3">
              {isEditMode ? t("patterns.form.editor_title_edit") : t("patterns.form.editor_title_new")}
            </Typography>
            <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium normal-case tracking-normal" variant="small">
              {isEditMode ? t("patterns.view_details") : t("patterns.modal.subtitle")}
            </Typography>
          </div>
        </div>
        <div className="flex items-center gap-4">
          <Button
            onClick={() => {
              void navigate({ to: "/shifts/rules", search: { tab: "patterns" } });
            }}
            size="lg"
            variant="outline"
          >
            {t("common.cancel").toUpperCase()}
          </Button>
          <Button
            className="flex items-center gap-2.5 shadow-md shadow-primary-500/10"
            disabled={isSaving}
            size="lg"
            type="submit"
            variant="secondary"
          >
            <Save size={16} />
            {t("patterns.modal.submit_btn").toUpperCase()}
          </Button>
        </div>
      </div>

      {/* Main Two-Column Content */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-10">
        <WorkingPatternSettingsSidebar
          cycleLengthDays={cycleLengthDays}
          description={description}
          handleCycleLengthChange={handleCycleLengthChange}
          handlePatternTypeChange={handlePatternTypeChange}
          name={name}
          patternType={patternType}
          setDescription={setDescription}
          setIsDirty={setIsDirty}
          setName={setName}
        />

        {/* Right Column: Interactive Schedule Grid */}
        <div className="lg:col-span-2 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] p-6 sm:p-10 shadow-sm flex flex-col">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-50 dark:border-slate-800 pb-6 mb-6">
            <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] flex items-center gap-2.5" variant="h3">
              <Clock className="text-primary-500" size={16} />
              {activeViewMode === "preview"
                ? t("patterns.form.view_mode_preview")
                : patternType === "weekly"
                ? t("patterns.form.weekly_grid_title")
                : t("patterns.form.cycle_grid_title") || "Cycle Schedule Grid"}
            </Typography>
            <div className="flex flex-wrap items-center gap-4">
              {patternType === "rotation" && (
                <div className="flex bg-slate-50 dark:bg-slate-800 p-1.5 rounded-2xl border border-slate-100 dark:border-slate-700/50 shadow-inner">
                  <Button className={cn(
                      "px-4 py-1.5 rounded-xl text-[10px] font-bold uppercase tracking-wider transition-all",
                      activeViewMode === "design"
                        ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100/50 dark:border-slate-700"
                        : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-350"
                    )}
                    onClick={() => {
                      setActiveViewMode("design");
                    }}
                    type="button"
                    variant="ghost"
                  >
                    {t("patterns.form.view_mode_design") || "Design"}
                  </Button>
                  <Button className={cn(
                      "px-4 py-1.5 rounded-xl text-[10px] font-bold uppercase tracking-wider transition-all",
                      activeViewMode === "preview"
                        ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100/50 dark:border-slate-700"
                        : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-350"
                    )}
                    onClick={() => {
                      setActiveViewMode("preview");
                    }}
                    type="button"
                    variant="ghost"
                  >
                    {t("patterns.form.view_mode_preview") || "Preview"}
                  </Button>
                </div>
              )}

              {activeViewMode === "design" && (
                <div className="flex items-center gap-3">
                  {patternType === "weekly" && (
                    <Button
                      className="text-[9px] text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950 border-primary-100 dark:border-primary-800 hover:bg-primary-100 dark:hover:bg-primary-900"
                      onClick={prefillStandard}
                      size="sm"
                      type="button"
                      variant="outline"
                    >
                      {t("patterns.form.prefill_standard").toUpperCase()}
                    </Button>
                  )}
                    <Button
                      className="text-[9px] text-rose-600 dark:text-rose-400 bg-rose-50 dark:bg-rose-950/20 border-rose-100 dark:border-rose-950 hover:bg-rose-100 dark:hover:bg-rose-900/35"
                      onClick={() => { setIsClearConfirmOpen(true); }}
                    size="sm"
                    type="button"
                    variant="outline"
                  >
                    <Trash2 className="mr-1 inline-block" size={10} />
                    {t("patterns.form.clear_all").toUpperCase()}
                  </Button>
                </div>
              )}
            </div>
          </div>

          {activeViewMode === "preview" ? (
            <WorkingPatternCalendarPreview
              cycleLengthDays={cycleLengthDays}
              generateCalendarCells={generateCalendarCells}
              grid={grid}
              handleNextMonth={handleNextMonth}
              handlePreviousMonth={handlePreviousMonth}
              previewAnchorDate={previewAnchorDate}
              previewMonth={previewMonth}
              previewYear={previewYear}
              setPreviewAnchorDate={setPreviewAnchorDate}
              shifts={shifts}
            />
          ) : (
            <WorkingPatternDesignEditor
              addShiftToDay={addShiftToDay}
              grid={grid}
              gridRows={gridRows}
              patternType={patternType}
              removeShiftFromDay={removeShiftFromDay}
              shifts={shifts}
            />
          )}
        </div>
      </div>

      {/* ConfirmDialogs */}
      <ConfirmDialog
        cancelLabel={t("patterns.form.unsaved_changes_cancel")}
        confirmLabel={t("patterns.form.unsaved_changes_confirm")}
        isOpen={blocker.status === "blocked"}
        message={t("patterns.form.unsaved_changes")}
        onClose={() => { blocker.reset?.(); }}
        onConfirm={() => { blocker.proceed?.(); }}
        title={t("common.warning")}
        variant="warning"
      />

      <ConfirmDialog
        cancelLabel={t("common.cancel")}
        confirmLabel={t("common.confirm")}
        isOpen={isClearConfirmOpen}
        message={t("patterns.form.clear_all_confirm")}
        onClose={() => { setIsClearConfirmOpen(false); }}
        onConfirm={handleConfirmClear}
        title={t("patterns.form.clear_all")}
        variant="danger"
      />
    </form>
  );
}
