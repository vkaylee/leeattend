/* eslint-disable max-lines-per-function */
import { useQueries } from "@tanstack/react-query";
import { useNavigate, useParams } from "@tanstack/react-router";
import { ArrowLeft, History, Save } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { ShiftRevisionHistoryDialog } from "@/components/shifts/shift-revision-history-dialog";
import { ShiftTimelinePreview } from "@/components/shifts/shift-timeline-preview";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import ConfirmDialog from "@/components/ui/confirm-dialog";
import { Typography } from "@/components/ui/typography";
import { fetchLocations, fetchEmployers } from "@/lib/api";

import { CorrectionConfigCard } from "./components/correction-config-card";
import { IntervalEditorCard } from "./components/interval-editor-card";
import { useShiftEditor } from "./hooks/use-shift-editor";

  // eslint-disable-next-line complexity
export default function ShiftEditorPage() {
  const { t } = useTranslation();
  const { shiftId } = useParams({ strict: false });
  const navigate = useNavigate();

  const {
    isEditMode,
    isLoadingShifts,
    shiftName,
    setShiftName,
    shiftSymbol,
    setShiftSymbol,
    locationId,
    setLocationId,
    employerId,
    setEmployerId,
    handleSubmit,
    isSaveDisabled,
    blocker,
    intervalDrafts,
    addInterval,
    removeInterval,
    updateIntervalDraft,
    isCorrection,
    setIsCorrection,
    correctionEffectiveDate,
    setCorrectionEffectiveDate,
    changeReason,
    setChangeReason,
  } = useShiftEditor(shiftId);

  // Parallel fetch — locations and employers are independent
  const [{ data: locations }, { data: employers }] = useQueries({
    queries: [
      {
        queryKey: ["locations"],
        queryFn: () => fetchLocations(),
      },
      {
        queryKey: ["employers"],
        queryFn: () => fetchEmployers(),
      },
    ],
  });

  const [historyOpen, setHistoryOpen] = useState(false);

  

  if (isEditMode && isLoadingShifts) {
    return (
      <div className="p-12 space-y-8 animate-pulse max-w-7xl mx-auto">
        <div className="h-10 w-64 bg-slate-200 dark:bg-slate-800 rounded-full" />
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-10">
          <div className="lg:col-span-2 h-[30rem] bg-slate-200 dark:bg-slate-800 rounded-2xl sm:rounded-[2.5rem]" />
          <div className="h-96 bg-slate-200 dark:bg-slate-800 rounded-2xl sm:rounded-[2.5rem]" />
        </div>
      </div>
    );
  }

  const navBack = () => {
    void navigate({ to: "/shifts", search: { tab: "directory", view: "week" } });
  };

  return (
    <form
      className="max-w-7xl mx-auto space-y-10 pb-20 animate-in fade-in duration-500"
      onSubmit={handleSubmit}
    >
      {/* Page Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
        <div className="flex items-center gap-4">
          <Button
            className="p-3 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 hover:bg-slate-50 dark:hover:bg-slate-850 rounded-2xl shadow-sm text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200 active:scale-95 transition-all"
            onClick={navBack}
            type="button"
            variant="ghost"
          >
            <ArrowLeft size={18} />
          </Button>
          <div>
            <Typography
              className="text-3xl font-extrabold text-slate-700 dark:text-slate-250 tracking-tight uppercase"
              variant="h1"
            >
              {isEditMode
                ? t("shifts.modal.edit_subtitle", "Chỉnh sửa ca làm việc")
                : t("shifts.modal.setup_title", "Cài đặt ca làm việc")}
            </Typography>
            <Typography
              className="text-slate-450 dark:text-slate-500 text-xs font-bold uppercase tracking-wider mt-1"
              variant="p"
            >
              {isEditMode
                ? t("shifts.modal.edit_description", "Cập nhật các thông số của ca")
                : t("shifts.modal.setup_subtitle", "Thiết lập các thông số cơ bản cho ca làm việc mới")}
            </Typography>
          </div>
        </div>
        <div className="flex items-center gap-4">
          {isEditMode && shiftId ? (
            <Button
              data-testid="shift-history-btn"
              onClick={() => { setHistoryOpen(true); }}
              size="lg"
              type="button"
              variant="outline"
            >
              <History size={16} />
              {t("shifts.revisions.title", "Version History")}
            </Button>
          ) : null}
          <Button
            data-testid="shift-cancel-btn"
            onClick={navBack}
            size="lg"
            type="button"
            variant="outline"
          >
            {t("common.cancel").toUpperCase()}
          </Button>
          <Button
            className="flex items-center gap-2.5 shadow-md shadow-primary-500/10"
            data-testid="shift-save-btn"
            disabled={isSaveDisabled}
            size="lg"
            type="submit"
            variant="primary"
          >
            <Save size={16} />
            {(isEditMode
              ? t("shifts.modal.update_config_btn", "Cập nhật")
              : t("shifts.modal.save_shift_btn", "Lưu ca")
            ).toUpperCase()}
          </Button>
        </div>
      </div>

      {/* Two-Column Form Layout */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-10">
        {/* Left Column */}
        <div className="lg:col-span-2 space-y-8">
          {/* Card 1: Shift Identity */}
          <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] p-6 sm:p-10 shadow-sm space-y-6">
            <Typography
              className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] border-b border-slate-50 dark:border-slate-800/80 pb-4"
              variant="h3"
            >
              {t("shifts.modal.basic_info", "Thông tin cơ bản").toUpperCase()}
            </Typography>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <label
                  className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest"
                  htmlFor="shift_name_input"
                >
                  {t("shifts.table.shift_name")} <span className="text-red-500">*</span>
                </label>
                <input
                  className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all outline-none"
                  id="shift_name_input"
                  onChange={(e) => { setShiftName(e.target.value); }}
                  placeholder={t("shifts.modal.name_placeholder")}
                  required
                  type="text"
                  value={shiftName}
                />
              </div>

              <div className="space-y-2">
                <label
                  className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest"
                  htmlFor="shift_symbol_input"
                >
                  {t("shifts.modal.report_symbol", "Ký hiệu báo cáo")}
                  <span className="ml-1 normal-case font-normal text-slate-400">
                    (tối đa 10 ký tự)
                  </span>
                </label>
                <input
                  className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all outline-none"
                  id="shift_symbol_input"
                  maxLength={10}
                  onChange={(e) => { setShiftSymbol(e.target.value.slice(0, 10)); }}
                  placeholder={t("shifts.modal.report_symbol_placeholder", "VD: S8, T8, FX...")}
                  type="text"
                  value={shiftSymbol}
                />
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-2">
              <div className="space-y-2">
                <label
                  className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest"
                  htmlFor="shift_location_input"
                >
                  {t("shifts.modal.location", "Chi nhánh / Địa điểm Scope")}
                </label>
                <Combobox
                  clearOnEscape
                  id="shift_location_input"
                  onChange={setLocationId}
                  options={[
                    { value: "", label: t("shifts.modal.global_scope", "Toàn cục / Dùng chung") },
                    ...(locations ?? []).map((loc) => ({
                      value: loc.id,
                      label: loc.name,
                      description: loc.timezone,
                    })),
                  ]}
                  placeholder={t("shifts.modal.select_location", "Chọn địa điểm làm việc")}
                  value={locationId ?? ""}
                />
              </div>

              <div className="space-y-2">
                <label
                  className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest"
                  htmlFor="shift_employer_input"
                >
                  {t("shifts.modal.employer", "Pháp nhân / Công ty Scope")}
                </label>
                <Combobox
                  clearOnEscape
                  id="shift_employer_input"
                  onChange={setEmployerId}
                  options={[
                    { value: "", label: t("shifts.modal.global_scope", "Toàn cục / Dùng chung") },
                    ...(employers ?? []).map((emp) => ({
                      value: emp.id,
                      label: emp.name,
                      description: emp.tax_code ? `MST: ${emp.tax_code}` : undefined,
                    })),
                  ]}
                  placeholder={t("shifts.modal.select_employer", "Chọn pháp nhân trực thuộc")}
                  value={employerId ?? ""}
                />
              </div>
            </div>
          </div>

          {/* Card 2: Intervals */}
          <IntervalEditorCard
            intervalDrafts={intervalDrafts}
            onAddInterval={addInterval}
            onRemoveInterval={removeInterval}
            onUpdateInterval={updateIntervalDraft}
          />

          {/* Card 3: Correction Config (Only in Edit Mode) */}
          {isEditMode ? (
            <CorrectionConfigCard
              changeReason={changeReason}
              correctionEffectiveDate={correctionEffectiveDate}
              isCorrection={isCorrection}
              onChangeReasonChange={setChangeReason}
              onCorrectionChange={setIsCorrection}
              onEffectiveDateChange={setCorrectionEffectiveDate}
            />
          ) : null}
        </div>

        {/* Right Column: Visual Timeline Preview */}
        <div className="space-y-8">
          <ShiftTimelinePreview intervals={intervalDrafts} />
        </div>
      </div>

      {/* Confirm Navigation Dialog */}
      <ConfirmDialog
        cancelLabel={t("patterns.form.unsaved_changes_cancel", "Hủy bỏ")}
        confirmLabel={t("patterns.form.unsaved_changes_confirm", "Rời đi")}
        isOpen={blocker.status === "blocked"}
        message={t(
          "patterns.form.unsaved_changes",
          "Bạn có thay đổi chưa lưu. Bạn có chắc chắn muốn rời khỏi trang này không?",
        )}
        onClose={() => blocker.reset?.()}
        onConfirm={() => blocker.proceed?.()}
        title={t("common.warning", "Cảnh báo")}
        variant="warning"
      />

      {/* Version History Dialog */}
      {shiftId ? (
        <ShiftRevisionHistoryDialog isOpen={historyOpen}
        onClose={() => { setHistoryOpen(false); }}
        shiftId={shiftId}
        shiftName={shiftName}  />
      ) : null}
    </form>
  );
}
