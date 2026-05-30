/* eslint-disable @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access */
import { useNavigate, useParams } from "@tanstack/react-router";
import { ArrowLeft, Save, Info, Zap, Clock } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import ConfirmDialog from "@/components/ui/confirm-dialog";
import { Typography } from "@/components/ui/typography";
import { ShiftTimelinePreview } from "@/components/shifts/shift-timeline-preview";
import { useShiftEditor } from "./hooks/use-shift-editor";

export default function ShiftEditorPage() {
  const { t } = useTranslation();
  const { shiftId } = useParams({ strict: false });
  const navigate = useNavigate();

  const {
    wsId,
    isEditMode,
    isLoadingShifts,
    shiftName,
    setShiftName,
    shiftSymbol,
    setShiftSymbol,
    shiftStart,
    setShiftStart,
    shiftEnd,
    setShiftEnd,
    shiftBreak,
    setShiftBreak,
    shiftGrace,
    setShiftGrace,
    breakStart,
    setBreakStart,
    breakEnd,
    setBreakEnd,
    isFixedBreak,
    setIsFixedBreak,
    isAutoDeduct,
    setIsAutoDeduct,
    isBreakOvertime,
    setIsBreakOvertime,
    allowEarlyOt,
    setAllowEarlyOt,
    maxEarlyOtMinutes,
    setMaxEarlyOtMinutes,
    isCorrection,
    setIsCorrection,
    correctionEffectiveDate,
    setCorrectionEffectiveDate,
    handleSubmit,
    isSaving,
    isSaveDisabled,
    blocker,
  } = useShiftEditor(shiftId);

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  if (isEditMode && isLoadingShifts) {
    return (
      <div className="p-12 space-y-8 animate-pulse max-w-7xl mx-auto">
        <div className="h-10 w-64 bg-slate-200 dark:bg-slate-800 rounded-full" />
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-10">
          <div className="lg:col-span-2 h-[30rem] bg-slate-200 dark:bg-slate-800 rounded-[2.5rem]" />
          <div className="h-96 bg-slate-200 dark:bg-slate-800 rounded-[2.5rem]" />
        </div>
      </div>
    );
  }

  return (
    <form className="max-w-7xl mx-auto space-y-10 pb-20 animate-in fade-in duration-500" onSubmit={handleSubmit}>
      {/* Page Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
        <div className="flex items-center gap-4">
          <Button
            className="p-3 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 hover:bg-slate-50 dark:hover:bg-slate-850 rounded-2xl shadow-sm text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200 active:scale-95 transition-all"
            onClick={() => {
              void navigate({ to: "/shifts", search: { tab: "directory" } });
            }}
            type="button"
            variant="ghost"
          >
            <ArrowLeft size={18} />
          </Button>
          <div>
            <Typography className="text-3xl font-extrabold text-slate-700 dark:text-slate-250 tracking-tight uppercase" variant="h1">
              {isEditMode ? t("shifts.modal.edit_subtitle", "Chỉnh sửa ca làm việc") : t("shifts.modal.setup_title", "Cài đặt ca làm việc")}
            </Typography>
            <Typography className="text-slate-450 dark:text-slate-500 text-xs font-bold uppercase tracking-wider mt-1" variant="p">
              {isEditMode ? t("shifts.modal.edit_description", "Cập nhật các thông số của ca") : t("shifts.modal.setup_subtitle", "Thiết lập các thông số cơ bản cho ca làm việc mới")}
            </Typography>
          </div>
        </div>
        <div className="flex items-center gap-4">
          <Button
            data-testid="shift-cancel-btn"
            onClick={() => {
              void navigate({ to: "/shifts", search: { tab: "directory" } });
            }}
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
            {(isEditMode ? t("shifts.modal.update_config_btn", "Cập nhật") : t("shifts.modal.save_shift_btn", "Lưu ca")).toUpperCase()}
          </Button>
        </div>
      </div>

      {/* Two-Column Form Layout */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-10">
        
        {/* Left Column: Form config inputs (takes 2 cols on lg) */}
        <div className="lg:col-span-2 space-y-8">
          
          {/* Card 1: Basic Shift Config */}
          <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] p-10 shadow-sm space-y-6">
            <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] border-b border-slate-50 dark:border-slate-800/80 pb-4" variant="h3">
              {t("shifts.modal.basic_info", "Thông tin cơ bản").toUpperCase()}
            </Typography>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                  {t("shifts.table.shift_name")} <span className="text-red-500">*</span>
                </label>
                <input
                  className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all outline-none"
                  onChange={(e) => setShiftName(e.target.value)}
                  placeholder={t("shifts.modal.name_placeholder")}
                  required
                  type="text"
                  value={shiftName}
                />
              </div>
              
              <div className="space-y-2">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                  {t("shifts.modal.report_symbol", "Ký hiệu báo cáo")}
                  <span className="ml-1 normal-case font-normal text-slate-400">(tối đa 10 ký tự)</span>
                </label>
                <input
                  className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all outline-none"
                  maxLength={10}
                  onChange={(e) => setShiftSymbol(e.target.value.slice(0, 10))}
                  placeholder={t("shifts.modal.report_symbol_placeholder", "VD: S8, T8, FX...")}
                  type="text"
                  value={shiftSymbol}
                />
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-2">
              <div className="space-y-2">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                  {t("shifts.modal.start_time")} <span className="text-red-500">*</span>
                </label>
                <input
                  className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all outline-none"
                  onChange={(e) => setShiftStart(e.target.value)}
                  required
                  type="time"
                  value={shiftStart}
                />
              </div>
              <div className="space-y-2">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                  {t("shifts.modal.end_time")} <span className="text-red-500">*</span>
                </label>
                <input
                  className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all outline-none"
                  onChange={(e) => setShiftEnd(e.target.value)}
                  required
                  type="time"
                  value={shiftEnd}
                />
              </div>
            </div>
          </div>

          {/* Card 2: Grace period & Early OT */}
          <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] p-10 shadow-sm space-y-6">
            <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] border-b border-slate-50 dark:border-slate-800/80 pb-4" variant="h3">
              {t("shifts.modal.ot_grace_config", "Chính sách đi muộn & Tăng ca").toUpperCase()}
            </Typography>

            {/* Grace Period */}
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                  {t("shifts.grace_period")}
                </label>
                <div className="group relative">
                  <Info className="text-slate-300 dark:text-slate-650 cursor-help" size={12} />
                  <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 w-56 p-3 bg-slate-800 text-white text-[10px] rounded-xl opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50 normal-case leading-normal font-normal">
                    {t("shifts.grace_period_desc")}
                  </div>
                </div>
              </div>
              <input
                className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all outline-none"
                min="0"
                onChange={(e) => setShiftGrace(parseInt(e.target.value) || 0)}
                type="number"
                value={shiftGrace}
              />
            </div>

            {/* Early OT Config */}
            <div className="p-6 bg-slate-50 dark:bg-slate-800/40 border border-slate-100 dark:border-slate-800 rounded-3xl space-y-4">
              <div className="flex items-center justify-between">
                <div className="space-y-1 pr-4">
                  <label className="text-xs font-bold text-slate-700 dark:text-slate-200 block">
                    {t("shifts.modal.allow_early_ot", "Cho phép tính tăng ca sớm")}
                  </label>
                  <span className="text-[10px] text-slate-500 dark:text-slate-400 block font-normal leading-normal">
                    {t("shifts.modal.allow_early_ot_help", "Tính thời gian làm việc trước ca là tăng ca sớm (Early OT)")}
                  </span>
                </div>
                <div
                  aria-checked={allowEarlyOt}
                  aria-label={t("shifts.modal.allow_early_ot", "Cho phép tính tăng ca sớm")}
                  className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 cursor-pointer ${allowEarlyOt ? 'bg-primary-600' : 'bg-slate-200 dark:bg-slate-700'}`}
                  onClick={() => setAllowEarlyOt(!allowEarlyOt)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      setAllowEarlyOt(!allowEarlyOt);
                    }
                  }}
                  role="switch"
                  tabIndex={0}
                >
                  <span className={`inline-block h-4 w-4 transform rounded-full bg-white dark:bg-slate-900 transition-transform ${allowEarlyOt ? 'translate-x-6' : 'translate-x-1'}`} />
                </div>
              </div>

              {allowEarlyOt && (
                <div className="space-y-2 pt-3 border-t border-slate-200/50 dark:border-slate-700/50 animate-in fade-in slide-in-from-top-2 duration-300">
                  <label htmlFor="max-early-ot" className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest block">
                    {t("shifts.modal.max_early_ot", "Giới hạn tăng ca sớm tối đa (phút)")} <span className="text-red-500">*</span>
                  </label>
                  <input
                    id="max-early-ot"
                    className="w-full bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all outline-none"
                    min="1"
                    onChange={(e) => setMaxEarlyOtMinutes(parseInt(e.target.value) || 0)}
                    type="number"
                    value={maxEarlyOtMinutes}
                  />
                  {maxEarlyOtMinutes <= 0 && (
                    <span className="text-[10px] text-red-500 block font-medium">
                      {t("shifts.modal.max_early_ot_required", "Vui lòng nhập số phút giới hạn lớn hơn 0.")}
                    </span>
                  )}
                </div>
              )}
            </div>
          </div>

          {/* Card 3: Correction Config (Only in Edit Mode) */}
          {isEditMode && (
            <div className="bg-amber-50 dark:bg-amber-950/20 border border-amber-100 dark:border-amber-900/60 rounded-[2.5rem] p-10 shadow-sm space-y-6">
              <div className="flex items-center gap-3">
                <div className="bg-amber-100 dark:bg-amber-900 p-2.5 rounded-2xl text-amber-600 dark:text-amber-300">
                  <Zap size={18} />
                </div>
                <div>
                  <Typography className="text-xs font-bold text-amber-900 dark:text-amber-400 uppercase tracking-tight" variant="h4">
                    {t("shifts.modal.correction_label", "Cập nhật hồi tố (Retroactive Correction)")}
                  </Typography>
                  <Typography className="text-[10px] font-medium text-amber-700 dark:text-amber-300/80 leading-normal" variant="p">
                    {t("shifts.modal.correction_desc", "Quyết định xem các thay đổi này có áp dụng cho dữ liệu quá khứ hay không.")}
                  </Typography>
                </div>
              </div>

              <div className="flex items-center gap-3 bg-white dark:bg-slate-900/50 p-4 rounded-2xl border border-amber-100/50 dark:border-amber-900/40">
                <input
                  checked={isCorrection}
                  className="w-4 h-4 rounded-md border-amber-300 text-amber-600 dark:text-amber-400 focus:ring-amber-500 cursor-pointer"
                  id="editor_is_correction"
                  onChange={(e) => setIsCorrection(e.target.checked)}
                  type="checkbox"
                />
                <label className="text-[10px] font-bold text-amber-800 dark:text-amber-300 uppercase tracking-widest cursor-pointer select-none" htmlFor="editor_is_correction">
                  {t("shifts.modal.correction_warning", "CẬP NHẬT TRỰC TIẾP LÊN CA GỐC (ÁP DỤNG HỒI TỐ)")}
                </label>
              </div>

              {!isCorrection && (
                <div className="space-y-2 pt-2 animate-in fade-in slide-in-from-top-2 duration-300">
                  <label className="text-[10px] font-bold text-amber-750 dark:text-amber-400 uppercase tracking-widest block">
                    {t("shifts.modal.effective_date_label", "Ngày hiệu lực mới")}
                  </label>
                  <input
                    className="w-full bg-white dark:bg-slate-900 border border-amber-250 dark:border-amber-900 rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-amber-500 transition-all outline-none"
                    onChange={(e) => setCorrectionEffectiveDate(e.target.value)}
                    type="date"
                    value={correctionEffectiveDate}
                  />
                  <Typography className="text-[9px] font-bold text-amber-600 dark:text-amber-350 leading-tight italic px-1" variant="p">
                    {t("shifts.modal.effective_date_desc", "Các thay đổi sẽ chỉ có hiệu lực từ ngày được chọn trở đi.")}
                  </Typography>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Right Column: Break Configuration & Visual Timeline (1 col on lg) */}
        <div className="space-y-8">
          
          {/* Break Configuration Card */}
          <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] p-8 shadow-sm space-y-6">
            <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] border-b border-slate-50 dark:border-slate-800/80 pb-4" variant="h3">
              {t("shifts.modal.break_config", "Thiết lập giờ nghỉ")}
            </Typography>

            {/* Fixed Break Toggle */}
            <div className="flex items-center justify-between p-4 bg-slate-50 dark:bg-slate-800/40 rounded-2xl border border-slate-100 dark:border-slate-800">
              <span className="text-xs font-bold text-slate-650 dark:text-slate-350">{t("shifts.types.fixed", "Giờ nghỉ cố định")}</span>
              <div
                aria-checked={isFixedBreak}
                aria-label={t("shifts.types.fixed")}
                className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 cursor-pointer ${isFixedBreak ? 'bg-primary-600' : 'bg-slate-200 dark:bg-slate-700'}`}
                onClick={() => setIsFixedBreak(!isFixedBreak)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    setIsFixedBreak(!isFixedBreak);
                  }
                }}
                role="switch"
                tabIndex={0}
              >
                <span className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white dark:bg-slate-900 transition-transform ${isFixedBreak ? 'translate-x-5' : 'translate-x-1'}`} />
              </div>
            </div>

            {/* Break Times / Duration Inputs */}
            {isFixedBreak ? (
              <div className="grid grid-cols-2 gap-4 animate-in fade-in duration-350">
                <div className="space-y-2">
                  <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase">{t("shifts.modal.break_start")}</label>
                  <input
                    className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-4 py-3 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all outline-none"
                    onChange={(e) => setBreakStart(e.target.value)}
                    type="time"
                    value={breakStart}
                  />
                </div>
                <div className="space-y-2">
                  <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase">{t("shifts.modal.break_end")}</label>
                  <input
                    className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-4 py-3 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all outline-none"
                    onChange={(e) => setBreakEnd(e.target.value)}
                    type="time"
                    value={breakEnd}
                  />
                </div>
              </div>
            ) : (
              <div className="space-y-2 animate-in fade-in duration-350">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase block">
                  {t("shifts.modal.break_duration")} ({t("common.minutes")})
                </label>
                <input
                  className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-4 py-3 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all outline-none"
                  onChange={(e) => setShiftBreak(parseInt(e.target.value) || 0)}
                  type="number"
                  value={shiftBreak}
                />
              </div>
            )}

            {/* Break Policy Radios */}
            <div className="space-y-3 pt-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest block">
                {t("shifts.modal.break_policy_label")}
              </label>
              
              <div className="space-y-2">
                
                {/* Option 1: Auto Deduct */}
                <div
                  className="flex items-start gap-3 p-4 bg-slate-50 dark:bg-slate-800/40 rounded-2xl border border-slate-100 dark:border-slate-800/80 cursor-pointer hover:bg-slate-100/50 dark:hover:bg-slate-800/30 transition-all"
                  onClick={() => {
                    setIsAutoDeduct(true);
                    setIsBreakOvertime(false);
                  }}
                >
                  <input
                    checked={isAutoDeduct && !isBreakOvertime}
                    className="mt-0.5 w-4 h-4 text-primary-600 border-slate-350 dark:border-slate-700 focus:ring-primary-500 cursor-pointer"
                    id="break_policy_auto_deduct"
                    name="break_policy"
                    onChange={() => {}}
                    type="radio"
                  />
                  <div className="space-y-1">
                    <label className="text-xs font-bold text-slate-750 dark:text-slate-200 cursor-pointer block" htmlFor="break_policy_auto_deduct">
                      {t("shifts.modal.break_policy_auto_deduct")}
                    </label>
                    <span className="text-[10px] text-slate-500 dark:text-slate-400 block font-normal leading-normal">
                      {t("shifts.modal.break_policy_auto_deduct_help")}
                    </span>
                  </div>
                </div>

                {/* Option 2: No Deduct */}
                <div
                  className="flex items-start gap-3 p-4 bg-slate-50 dark:bg-slate-800/40 rounded-2xl border border-slate-100 dark:border-slate-800/80 cursor-pointer hover:bg-slate-100/50 dark:hover:bg-slate-800/30 transition-all"
                  onClick={() => {
                    setIsAutoDeduct(false);
                    setIsBreakOvertime(false);
                  }}
                >
                  <input
                    checked={!isAutoDeduct && !isBreakOvertime}
                    className="mt-0.5 w-4 h-4 text-primary-600 border-slate-355 dark:border-slate-700 focus:ring-primary-500 cursor-pointer"
                    id="break_policy_no_deduct"
                    name="break_policy"
                    onChange={() => {}}
                    type="radio"
                  />
                  <div className="space-y-1">
                    <label className="text-xs font-bold text-slate-755 dark:text-slate-200 cursor-pointer block" htmlFor="break_policy_no_deduct">
                      {t("shifts.modal.break_policy_no_deduct")}
                    </label>
                    <span className="text-[10px] text-slate-500 dark:text-slate-400 block font-normal leading-normal">
                      {t("shifts.modal.break_policy_no_deduct_help")}
                    </span>
                  </div>
                </div>

                {/* Option 3: Break Overtime */}
                <div
                  className={`flex items-start gap-3 p-4 rounded-2xl border transition-all ${
                    !isFixedBreak
                      ? "bg-slate-100/50 dark:bg-slate-900/20 border-slate-200/50 dark:border-slate-800/30 opacity-60 cursor-not-allowed"
                      : "bg-slate-50 dark:bg-slate-800/40 border-slate-100 dark:border-slate-800/80 cursor-pointer hover:bg-slate-100/50 dark:hover:bg-slate-800/30"
                  }`}
                  onClick={() => {
                    if (isFixedBreak) {
                      setIsAutoDeduct(false);
                      setIsBreakOvertime(true);
                    }
                  }}
                >
                  <input
                    checked={isBreakOvertime}
                    className={`mt-0.5 w-4 h-4 text-primary-600 border-slate-355 dark:border-slate-700 focus:ring-primary-500 ${
                      !isFixedBreak ? "cursor-not-allowed" : "cursor-pointer"
                    }`}
                    disabled={!isFixedBreak}
                    id="break_policy_overtime"
                    name="break_policy"
                    onChange={() => {}}
                    type="radio"
                  />
                  <div className="space-y-1">
                    <label className={`text-xs font-bold block ${!isFixedBreak ? "text-slate-400 dark:text-slate-500 cursor-not-allowed" : "text-slate-700 dark:text-slate-200 cursor-pointer"}`} htmlFor="break_policy_overtime">
                      {t("shifts.modal.break_policy_overtime")}
                    </label>
                    <span className="text-[10px] text-slate-500 dark:text-slate-400 block font-normal leading-normal">
                      {!isFixedBreak ? (
                        <span className="text-amber-600 dark:text-amber-500 font-medium">
                          ⚠️ {t("shifts.modal.break_policy_overtime_disabled_hint")}
                        </span>
                      ) : (
                        t("shifts.modal.break_policy_overtime_help")
                      )}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Visual Shift Timeline Preview */}
          <ShiftTimelinePreview
            allowEarlyOt={allowEarlyOt}
            breakEnd={breakEnd}
            breakStart={breakStart}
            isAutoDeduct={isAutoDeduct}
            isBreakOvertime={isBreakOvertime}
            isFixedBreak={isFixedBreak}
            maxEarlyOtMinutes={maxEarlyOtMinutes}
            shiftBreak={shiftBreak}
            shiftEnd={shiftEnd}
            shiftGrace={shiftGrace}
            shiftName={shiftName}
            shiftStart={shiftStart}
          />
        </div>
      </div>

      {/* Confirm Navigation Dialog */}
      <ConfirmDialog
        cancelLabel={t("patterns.form.unsaved_changes_cancel", "Hủy bỏ")}
        confirmLabel={t("patterns.form.unsaved_changes_confirm", "Rời đi")}
        isOpen={blocker.status === "blocked"}
        message={t("patterns.form.unsaved_changes", "Bạn có thay đổi chưa lưu. Bạn có chắc chắn muốn rời khỏi trang này không?")}
        onClose={() => blocker.reset?.()}
        onConfirm={() => blocker.proceed?.()}
        title={t("common.warning", "Cảnh báo")}
        variant="warning"
      />
    </form>
  );
}
