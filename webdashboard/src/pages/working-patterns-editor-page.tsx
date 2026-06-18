import { useBlocker, useNavigate, useParams } from "@tanstack/react-router";
import {
  CalendarDays,
  Clock,
  RefreshCw,
  X,
  ArrowLeft,
  Save,
  Trash2,
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import ConfirmDialog from "@/components/ui/confirm-dialog";
import FormField, { LABEL_BASE } from "@/components/ui/form-field";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useCalendarPreview } from "@/hooks/use-calendar-preview";
import { useWorkingPatternForm, isCycleLengthInvalid, convertGridToDetails } from "@/hooks/use-working-pattern-form";
import { cn } from "@/lib/utils";

/* eslint-disable complexity, max-lines-per-function */
export default function WorkingPatternsEditorPage() {
  const { t, i18n } = useTranslation();
  const { patternId } = useParams({ strict: false });
  const navigate = useNavigate();
  const toast = useToast();
  const [isClearConfirmOpen, setIsClearConfirmOpen] = useState(false);

  const {
    wsId,
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

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  if (isEditMode && isLoadingPatterns) {
    return (
      <div className="p-12 space-y-8 animate-pulse max-w-7xl mx-auto">
        <div className="h-10 w-64 bg-slate-200 dark:bg-slate-800 rounded-full" />
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-10">
          <div className="h-96 bg-slate-200 dark:bg-slate-800 rounded-[2.5rem]" />
          <div className="lg:col-span-2 h-[30rem] bg-slate-200 dark:bg-slate-800 rounded-[2.5rem]" />
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
              void navigate({ to: "/working-patterns" });
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
              void navigate({ to: "/working-patterns" });
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
        {/* Left Column: Metadata & General Config */}
        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] p-10 shadow-sm space-y-8 flex flex-col justify-start">
          <div className="space-y-6">
            <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] border-b border-slate-50 dark:border-slate-800 pb-4" variant="h3">
              {t("patterns.form.desc_label").toUpperCase()}
            </Typography>
            <FormField
              inputClassName="font-bold tracking-tight rounded-2xl"
              label={t("patterns.form.name_label")}
              onChange={(e) => {
                setName(e.target.value);
                setIsDirty(true);
              }}
              placeholder={t("patterns.form.name_placeholder")}
              required
              value={name}
            />
            <FormField
              inputClassName="rounded-2xl min-h-[5rem]"
              label={t("patterns.form.desc_label")}
              onChange={(e) => {
                setDescription(e.target.value);
                setIsDirty(true);
              }}
              placeholder={t("patterns.form.desc_placeholder")}
              type="textarea"
              value={description}
            />
          </div>

          <div className="space-y-6">
            <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] border-b border-slate-50 dark:border-slate-800 pb-4" variant="h3">
              {t("patterns.form.pattern_type").toUpperCase()}
            </Typography>
            
            <div className="space-y-4">
              <label className={LABEL_BASE}>{t("patterns.form.pattern_type").toUpperCase()}</label>
              <div className="flex bg-slate-50 dark:bg-slate-800 p-1.5 rounded-2xl border border-slate-100 dark:border-slate-700/50">
                <Button className={cn(
                    "flex-1 flex items-center justify-center gap-2.5 py-3 rounded-xl text-[10px] font-bold uppercase tracking-wider transition-all",
                    patternType === "weekly"
                      ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100/50 dark:border-slate-850"
                      : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-350"
                  )}
                  onClick={() => { handlePatternTypeChange("weekly"); }}
                  type="button"
                  variant="ghost"
                >
                  <CalendarDays size={14} />
                  {t("patterns.form.pattern_type_weekly") || "Weekly"}
                </Button>
                <Button className={cn(
                    "flex-1 flex items-center justify-center gap-2.5 py-3 rounded-xl text-[10px] font-bold uppercase tracking-wider transition-all",
                    patternType === "rotation"
                      ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100/50 dark:border-slate-850"
                      : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-350"
                  )}
                  onClick={() => { handlePatternTypeChange("rotation"); }}
                  type="button"
                  variant="ghost"
                >
                  <RefreshCw size={14} />
                  {t("patterns.form.pattern_type_rotation") || "Rotation"}
                </Button>
              </div>
            </div>

            {patternType === "rotation" && (
              <div className="p-5 bg-slate-50/50 dark:bg-slate-850/30 border border-slate-100 dark:border-slate-800 rounded-2xl space-y-4 animate-in fade-in slide-in-from-top-4 duration-300">
                <div className="flex items-center gap-2 text-slate-700 dark:text-slate-200">
                  <RefreshCw className="text-primary-500" size={14} />
                  <span className="text-[10px] font-extrabold uppercase tracking-wider">
                    {t("patterns.form.cycle_config_title") || "Rotation Configuration"}
                  </span>
                </div>
                
                <div className="w-28">
                  <FormField
                    className="!px-0"
                    inputClassName="font-bold tracking-tight rounded-2xl text-center"
                    label={t("patterns.form.cycle_length") || "Days"}
                    onChange={(e) => {
                      handleCycleLengthChange(parseInt(e.target.value) || 1);
                    }}
                    placeholder="e.g. 6"
                    required
                    type="number"
                    value={String(cycleLengthDays)}
                  />
                </div>

                <Typography className="text-[9px] text-slate-400 font-bold uppercase leading-relaxed italic opacity-75" variant="p">
                  {t("patterns.form.cycle_length_note")}
                </Typography>
              </div>
            )}
          </div>
        </div>

        {/* Right Column: Interactive Schedule Grid */}
        <div className="lg:col-span-2 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] p-10 shadow-sm flex flex-col">
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
            <div className="flex-1 flex flex-col space-y-6">
              {/* Calendar Config Controls */}
              <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-4 p-5 bg-slate-50/50 dark:bg-slate-850/30 border border-slate-100 dark:border-slate-800 rounded-2xl">
                {/* Anchor Date Picker */}
                <div className="flex-1 flex flex-col gap-2">
                  <label className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider">
                    {t("patterns.form.anchor_date_preview")}
                  </label>
                  <input
                    className="w-full h-10 px-4 rounded-xl border border-slate-100 dark:border-slate-700 bg-white dark:bg-slate-900 text-sm font-bold text-slate-700 dark:text-slate-200 outline-none focus:border-primary-500 transition-colors"
                    onChange={(e) => { setPreviewAnchorDate(e.target.value); }}
                    type="date"
                    value={previewAnchorDate}
                  />
                </div>

                {/* Month Selector */}
                <div className="flex-1 flex flex-col gap-2">
                  <label className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider text-right sm:text-right">
                    {t("patterns.form.preview_month")}
                  </label>
                  <div className="flex items-center justify-end gap-3 h-10">
                    <Button className="p-2 border border-slate-100 dark:border-slate-700 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800 active:scale-95 transition-all"
                      onClick={handlePreviousMonth}
                      type="button"
                      variant="ghost"
                    >
                      <ArrowLeft size={16} />
                    </Button>
                    <span className="text-sm font-extrabold text-slate-700 dark:text-slate-200 uppercase tracking-tight min-w-[8rem] text-center">
                      {new Date(previewYear, previewMonth).toLocaleString(i18n.language || "vi", {
                        month: "long",
                        year: "numeric",
                      })}
                    </span>
                    <Button className="p-2 border border-slate-100 dark:border-slate-700 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800 active:scale-95 transition-all rotate-180"
                      onClick={handleNextMonth}
                      type="button"
                      variant="ghost"
                    >
                      <ArrowLeft size={16} />
                    </Button>
                  </div>
                </div>
              </div>

              {/* Month Calendar Grid */}
              <div className="flex-1 flex flex-col">
                {/* Day of Week Headers */}
                <div className="grid grid-cols-7 gap-2 mb-2 text-center">
                  {[
                    t("days.sunday"),
                    t("days.monday"),
                    t("days.tuesday"),
                    t("days.wednesday"),
                    t("days.thursday"),
                    t("days.friday"),
                    t("days.saturday"),
                  ].map((dName, idx) => (
                    <span
                      className="text-[9px] font-extrabold text-slate-400 dark:text-slate-500 uppercase tracking-widest py-1"
                      key={idx}
                    >
                      {dName.slice(0, 3)}
                    </span>
                  ))}
                </div>

                {/* Calendar Cells */}
                <div className="grid grid-cols-7 gap-2 flex-1 auto-rows-fr">
                  {(() => {
                    const cells = generateCalendarCells(cycleLengthDays, grid);
                    
                    return cells.map(({ date, isCurrentMonth, isToday, isAnchor, cycleIndex, dayShifts }, idx) => {

                      return (
                        <div
                          className={cn(
                            "min-h-[5.5rem] p-3 rounded-2xl border transition-all duration-300 flex flex-col justify-between",
                            isCurrentMonth
                              ? "bg-white dark:bg-slate-900 border-slate-100 dark:border-slate-800"
                              : "bg-slate-50/20 dark:bg-slate-900/10 border-slate-50 dark:border-slate-900/50 opacity-40",
                            isToday && "ring-2 ring-primary-500/50 border-primary-100 dark:border-primary-850",
                            isAnchor && "border-secondary-500 bg-secondary-50/10 dark:bg-secondary-950/10"
                          )}
                          key={idx}
                        >
                          <div className="flex justify-between items-start">
                            <span
                              className={cn(
                                "text-xs font-extrabold",
                                isToday
                                  ? "text-primary-500"
                                  : isCurrentMonth
                                  ? "text-slate-700 dark:text-slate-200"
                                  : "text-slate-400 dark:text-slate-650"
                              )}
                            >
                              {date.getDate()}
                            </span>
                            {cycleIndex >= 0 && (
                              <span className="text-[8px] font-extrabold uppercase px-1.5 py-0.5 rounded bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-750 text-slate-400 dark:text-slate-500">
                                D{cycleIndex + 1}
                              </span>
                            )}
                          </div>

                          <div className="flex flex-col gap-1 mt-2">
                            {dayShifts.length > 0 ? (
                              dayShifts.map((shiftId) => {
                                const shiftName = shifts.find((s) => s.id === shiftId)?.name ?? shiftId;
                                return (
                                  <div
                                    className="flex items-center gap-1 px-1.5 py-0.5 bg-primary-50 dark:bg-primary-950 border border-primary-100/50 dark:border-primary-900 rounded-md shadow-sm"
                                    key={shiftId}
                                  >
                                    <Clock className="text-primary-500 flex-shrink-0" size={8} />
                                    <span className="text-[8px] font-bold text-primary-750 dark:text-primary-400 truncate uppercase tracking-tight">
                                      {shiftName}
                                    </span>
                                  </div>
                                );
                              })
                            ) : (
                              <div className="text-center py-1">
                                <span className="text-[8px] font-extrabold text-slate-300 dark:text-slate-650 uppercase tracking-wider italic">
                                  {t("status_off")}
                                </span>
                              </div>
                            )}
                          </div>
                        </div>
                      );
                    });
                  })()}
                </div>
              </div>
            </div>
          ) : (
            /* Design Mode */
            <div className="flex-1">
              {patternType === "weekly" ? (
                /* Weekly Mode: vertical list */
                <div className="space-y-4 max-h-[60vh] overflow-y-auto pr-2 scrollbar-thin scrollbar-thumb-slate-200 dark:scrollbar-thumb-slate-800 scrollbar-track-transparent">
                  {gridRows.map((dayName, index) => {
                    const dayShifts = grid[index] || [];
                    return (
                      <div
                        className={cn(
                          "flex flex-col sm:flex-row sm:items-center justify-between gap-4 px-6 py-4 rounded-2xl border transition-all duration-300",
                          dayShifts.length > 0
                            ? "bg-white dark:bg-slate-900 border-primary-100 dark:border-primary-800/50 shadow-sm"
                            : "bg-slate-50/50 dark:bg-slate-850/30 border-slate-100 dark:border-slate-850"
                        )}
                        key={index}
                      >
                        <div className="w-28 flex-shrink-0">
                          <span
                            className={cn(
                              "text-[10px] font-extrabold uppercase tracking-widest block",
                              dayShifts.length > 0
                                ? "text-slate-700 dark:text-slate-200"
                                : "text-slate-400 dark:text-slate-500"
                            )}
                          >
                            {dayName}
                          </span>
                        </div>

                        <div className="flex-1 flex items-center gap-2 flex-wrap min-h-[2.5rem]">
                          {dayShifts.map((shiftId) => {
                            const shiftName = shifts.find((s) => s.id === shiftId)?.name ?? shiftId;
                            return (
                              <div
                                className="flex items-center gap-1.5 px-3 py-1.5 bg-primary-50 dark:bg-primary-950 border border-primary-100 dark:border-primary-850 rounded-xl shadow-sm animate-in zoom-in-95 duration-350"
                                key={shiftId}
                              >
                                <Clock className="text-primary-500 dark:text-primary-400 flex-shrink-0" size={10} />
                                <span className="text-[10px] font-bold text-primary-700 dark:text-primary-300 uppercase tracking-tight">
                                  {shiftName.toUpperCase()}
                                </span>
                                <Button aria-label={`Remove ${shiftName}`}
                                  className="p-0.5 rounded-md text-primary-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-950/50 transition-colors flex-shrink-0 ml-1"
                                  onClick={() => {
                                    removeShiftFromDay(index, shiftId);
                                  }}
                                  type="button"
                                  variant="destructive"
                                >
                                  <X size={12} />
                                </Button>
                              </div>
                            );
                          })}
                          {dayShifts.length === 0 && (
                            <span className="text-[10px] font-bold text-slate-300 dark:text-slate-650 uppercase tracking-widest italic">
                              {t("status_off")}
                            </span>
                          )}
                        </div>

                        <Combobox
                          className="w-36 flex-shrink-0 !py-1.5 !px-3 rounded-xl text-[9px] font-bold uppercase tracking-tight outline-none border border-dashed border-primary-200 dark:border-primary-800 bg-transparent text-primary-500 dark:text-primary-400 cursor-pointer transition-all hover:bg-primary-50 dark:hover:bg-primary-950/50"
                          onChange={(val) => {
                            if (val) addShiftToDay(index, val);
                          }}
                          options={shifts
                            .filter((s) => !dayShifts.includes(s.id))
                            .map((s) => ({ label: s.name.toUpperCase(), value: s.id }))}
                          placeholder={`+ ${t("patterns.form.add_shift") || "ADD"}`}
                          value=""
                        />
                      </div>
                    );
                  })}
                </div>
              ) : (
                /* Rotation Mode: Adaptive Grid Layout */
                <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4 max-h-[60vh] overflow-y-auto pr-2 scrollbar-thin scrollbar-thumb-slate-200 dark:scrollbar-thumb-slate-800 scrollbar-track-transparent p-1">
                  {gridRows.map((dayName, index) => {
                    const dayShifts = grid[index] || [];
                    return (
                      <div
                        className={cn(
                          "flex flex-col justify-between p-5 rounded-[1.5rem] border transition-all duration-300 gap-4 min-h-[10rem]",
                          dayShifts.length > 0
                            ? "bg-white dark:bg-slate-900 border-primary-100 dark:border-primary-800/50 shadow-sm"
                            : "bg-slate-50/50 dark:bg-slate-850/30 border-slate-100 dark:border-slate-850"
                        )}
                        key={index}
                      >
                        <div className="flex items-center justify-between">
                          <span
                            className={cn(
                              "text-[10px] font-extrabold uppercase tracking-widest",
                              dayShifts.length > 0
                                ? "text-slate-750 dark:text-slate-200"
                                : "text-slate-400 dark:text-slate-500"
                            )}
                          >
                            {dayName}
                          </span>
                        </div>

                        <div className="flex-1 flex flex-col gap-1.5 justify-center min-h-[3rem]">
                          {dayShifts.length > 0 ? (
                            dayShifts.map((shiftId) => {
                              const shiftName = shifts.find((s) => s.id === shiftId)?.name ?? shiftId;
                              return (
                                <div
                                  className="flex items-center justify-between gap-1.5 px-2.5 py-1.5 bg-primary-50 dark:bg-primary-950 border border-primary-100 dark:border-primary-850 rounded-xl shadow-sm animate-in zoom-in-95 duration-350"
                                  key={shiftId}
                                >
                                  <div className="flex items-center gap-1.5 overflow-hidden">
                                    <Clock
                                      className="text-primary-500 dark:text-primary-400 flex-shrink-0"
                                      size={10}
                                    />
                                    <span className="text-[10px] font-bold text-primary-750 dark:text-primary-300 uppercase tracking-tight truncate">
                                      {shiftName.toUpperCase()}
                                    </span>
                                  </div>
                                  <Button aria-label={`Remove ${shiftName}`}
                                    className="p-0.5 rounded-md text-primary-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-950/50 transition-colors flex-shrink-0 ml-1"
                                    onClick={() => {
                                      removeShiftFromDay(index, shiftId);
                                    }}
                                    type="button"
                                    variant="destructive"
                                  >
                                    <X size={12} />
                                  </Button>
                                </div>
                              );
                            })
                          ) : (
                            <div className="flex items-center justify-center flex-1 border border-dashed border-slate-200 dark:border-slate-800 rounded-2xl bg-slate-50/20 dark:bg-slate-900/10 min-h-[3rem]">
                              <span className="text-[10px] font-bold text-slate-350 dark:text-slate-600 uppercase tracking-widest italic">
                                {t("status_off")}
                              </span>
                            </div>
                          )}
                        </div>

                        <Combobox
                          className="w-full !py-1.5 !px-3 rounded-xl text-[9px] font-bold uppercase tracking-tight outline-none border border-dashed border-primary-200 dark:border-primary-800 bg-transparent text-primary-500 dark:text-primary-400 cursor-pointer transition-all hover:bg-primary-50 dark:hover:bg-primary-950/50"
                          onChange={(val) => {
                            if (val) addShiftToDay(index, val);
                          }}
                          options={shifts
                            .filter((s) => !dayShifts.includes(s.id))
                            .map((s) => ({ label: s.name.toUpperCase(), value: s.id }))}
                          placeholder={`+ ${t("patterns.form.add_shift") || "ADD"}`}
                          value=""
                        />
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
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
