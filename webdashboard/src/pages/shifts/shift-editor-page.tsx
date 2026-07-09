
/* eslint-disable max-lines-per-function, complexity */
import { useQuery } from "@tanstack/react-query";
import { useNavigate, useParams } from "@tanstack/react-router";
import { ArrowLeft, Save, Info, Zap, Clock } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { ShiftTimelinePreview } from "@/components/shifts/shift-timeline-preview";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import ConfirmDialog from "@/components/ui/confirm-dialog";
import { Typography } from "@/components/ui/typography";
import { fetchLocations, fetchEmployers } from "@/lib/api";

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
  } = useShiftEditor(shiftId);

  const { data: locations } = useQuery({
    queryKey: ["locations", wsId],
    queryFn: () => fetchLocations(wsId),
    enabled: !!wsId,
  });

  const { data: employers } = useQuery({
    queryKey: ["employers", wsId],
    queryFn: () => fetchEmployers(wsId),
    enabled: !!wsId,
  });

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

        {/* Left Column: Shift identity + Intervals (takes 2 cols on lg) */}
        <div className="lg:col-span-2 space-y-8">

          {/* Card 1: Shift Identity */}
          <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] p-10 shadow-sm space-y-6">
            <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] border-b border-slate-50 dark:border-slate-800/80 pb-4" variant="h3">
              {t("shifts.modal.basic_info", "Thông tin cơ bản").toUpperCase()}
            </Typography>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" htmlFor="shift_name_input">
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
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" htmlFor="shift_symbol_input">
                  {t("shifts.modal.report_symbol", "Ký hiệu báo cáo")}
                  <span className="ml-1 normal-case font-normal text-slate-400">(tối đa 10 ký tự)</span>
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
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" htmlFor="shift_location_input">
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
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" htmlFor="shift_employer_input">
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

          {/* Card 2: Intervals (each with full time/break/policy config) */}
          <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] p-10 shadow-sm space-y-6">
            <div className="flex items-center justify-between border-b border-slate-50 dark:border-slate-800/80 pb-4">
              <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]" variant="h3">
                {t("work_segments.intervals.title", "Khung giờ trong ca").toUpperCase()}
              </Typography>
              <Button
                className="text-xs"
                onClick={addInterval}
                size="sm"
                type="button"
                variant="outline"
              >
                + {t("work_segments.intervals.add_interval", "Thêm khung giờ")}
              </Button>
            </div>

            {intervalDrafts.length === 0 ? (
              <div className="text-center py-8 text-slate-400 dark:text-slate-500 text-sm italic">
                <Info className="mx-auto mb-2 opacity-40" size={24} />
                {t("work_segments.intervals.no_intervals", "Chưa có khung giờ. Hệ thống dùng giờ bắt đầu/kết thúc của ca.")}
              </div>
            ) : (
              <div className="space-y-6">
                {intervalDrafts.map((iv, idx) => (
                  <div className="border border-slate-100 dark:border-slate-800 rounded-2xl p-5 space-y-4 relative" key={iv.key}>
                    <div className="flex items-center justify-between">
                      <span className="text-xs font-bold text-slate-500 dark:text-slate-400">
                        {t("work_segments.intervals.interval", "Khung giờ")} {idx + 1}
                      </span>
                      {intervalDrafts.length > 1 ? (
                        <Button
                          className="text-xs text-red-500 hover:text-red-700"
                          onClick={() => { removeInterval(iv.key); }}
                          size="sm"
                          type="button"
                          variant="ghost"
                        >
                          ✕ {t("work_segments.intervals.remove_interval", "Xóa")}
                        </Button>
                      ) : null}
                    </div>

                    {/* Interval name */}
                    <div>
                      <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
                        {t("work_segments.intervals.interval_name", "Tên khung giờ")}
                      </label>
                      <input
                        className="w-full mt-1 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
                        onChange={(e) => { updateIntervalDraft(iv.key, { intervalName: e.target.value }); }}
                        placeholder={idx === 0 ? "VD: Ca sáng, Ca chính..." : "VD: Ca chiều, Tăng ca..."}
                        type="text"
                        value={iv.intervalName}
                      />
                    </div>

                    {/* Time range */}
                    <div className="grid grid-cols-2 gap-3">
                      <div>
                        <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
                          {t("work_segments.intervals.start_time", "Bắt đầu")}
                        </label>
                        <input
                          className="w-full mt-1 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
                          onChange={(e) => { updateIntervalDraft(iv.key, { startTime: e.target.value, isNightShift: e.target.value > iv.endTime }); }}
                          type="time"
                          value={iv.startTime}
                        />
                      </div>
                      <div>
                        <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
                          {t("work_segments.intervals.end_time", "Kết thúc")}
                        </label>
                        <input
                          className="w-full mt-1 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
                          onChange={(e) => { updateIntervalDraft(iv.key, { endTime: e.target.value, isNightShift: iv.startTime > e.target.value }); }}
                          type="time"
                          value={iv.endTime}
                        />
                      </div>
                    </div>

                    {/* Night shift indicator (auto-derived) */}
                    {iv.startTime > iv.endTime ? (
                      <div className="text-[10px] text-indigo-500 dark:text-indigo-400 bg-indigo-50 dark:bg-indigo-950/20 px-3 py-1.5 rounded-lg flex items-center gap-2">
                        <Clock size={12} />
                        {t("work_segments.night_shift.label", "Ca đêm")} — {t("work_segments.night_shift.description", "Giờ làm qua nửa đêm")}
                      </div>
                    ) : null}

                    {/* Break Config */}
                    <div className="border-t border-slate-100 dark:border-slate-800 pt-3 space-y-3">
                      <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
                        {t("shifts.modal.break_config", "Thiết lập giờ nghỉ")}
                      </span>

                      {/* Fixed Break Toggle */}
                      <div className="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-800/40 rounded-xl">
                        <span className="text-xs font-bold text-slate-600 dark:text-slate-350">{t("shifts.types.fixed", "Giờ nghỉ cố định")}</span>
                        <div
                          aria-checked={iv.isFixedBreak}
                          className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 cursor-pointer ${iv.isFixedBreak ? 'bg-primary-600' : 'bg-slate-200 dark:bg-slate-700'}`}
                          onClick={() => {
                            const next = !iv.isFixedBreak;
                            updateIntervalDraft(iv.key, {
                              isFixedBreak: next,
                              isBreakOvertime: next ? iv.isBreakOvertime : false,
                              isAutoDeduct: next ? iv.isAutoDeduct : true,
                            });
                          }}
                          onKeyDown={(e) => {
                            if (e.key === "Enter" || e.key === " ") { e.preventDefault(); }
                          }}
                          role="switch"
                          tabIndex={0}
                        >
                          <span className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white dark:bg-slate-900 transition-transform ${iv.isFixedBreak ? 'translate-x-5' : 'translate-x-1'}`} />
                        </div>
                      </div>

                      {iv.isFixedBreak ? (
                        <div className="grid grid-cols-2 gap-3 animate-in fade-in duration-350">
                          <div>
                            <label className="text-[10px] font-bold text-slate-400 uppercase">{t("shifts.modal.break_start")}</label>
                            <input
                              className="w-full mt-1 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
                              onChange={(e) => { updateIntervalDraft(iv.key, { breakStart: e.target.value }); }}
                              type="time"
                              value={iv.breakStart}
                            />
                          </div>
                          <div>
                            <label className="text-[10px] font-bold text-slate-400 uppercase">{t("shifts.modal.break_end")}</label>
                            <input
                              className="w-full mt-1 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
                              onChange={(e) => { updateIntervalDraft(iv.key, { breakEnd: e.target.value }); }}
                              type="time"
                              value={iv.breakEnd}
                            />
                          </div>
                        </div>
                      ) : (
                        <div className="animate-in fade-in duration-350">
                          <label className="text-[10px] font-bold text-slate-400 uppercase block">{t("shifts.modal.break_duration")} ({t("common.minutes")})</label>
                          <input
                            className="w-full mt-1 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
                            onChange={(e) => { updateIntervalDraft(iv.key, { breakDuration: parseInt(e.target.value) || 0 }); }}
                            type="number"
                            value={iv.breakDuration}
                          />
                        </div>
                      )}

                      {/* Break Policy */}
                      <div className="space-y-2">
                        <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider block">{t("shifts.modal.break_policy_label")}</span>
                        {[
                          { val: { isAutoDeduct: true, isBreakOvertime: false }, label: t("shifts.modal.break_policy_auto_deduct"), help: t("shifts.modal.break_policy_auto_deduct_help") },
                          { val: { isAutoDeduct: false, isBreakOvertime: false }, label: t("shifts.modal.break_policy_no_deduct"), help: t("shifts.modal.break_policy_no_deduct_help") },
                          { val: { isAutoDeduct: false, isBreakOvertime: true }, label: t("shifts.modal.break_policy_overtime"), help: iv.isFixedBreak ? t("shifts.modal.break_policy_overtime_help") : t("shifts.modal.break_policy_overtime_disabled_hint"), disabled: !iv.isFixedBreak },
                        ].map((opt, oi) => (
                          <label
                            aria-label={`${opt.label} ${opt.help}`}
                            className={`flex items-start gap-3 p-3 rounded-xl border transition-all ${opt.disabled ? "bg-slate-100/50 dark:bg-slate-900/20 border-slate-200/50 opacity-60 cursor-not-allowed" : "bg-slate-50 dark:bg-slate-800/40 border-slate-100 dark:border-slate-800/80 cursor-pointer hover:bg-slate-100/50"}`}
                            htmlFor={`break_policy_${iv.key}_${oi}`}
                            key={oi}
                          >
                            <input
                              checked={iv.isAutoDeduct === opt.val.isAutoDeduct && iv.isBreakOvertime === opt.val.isBreakOvertime}
                              className="mt-0.5 w-4 h-4 text-primary-600 cursor-pointer"
                              disabled={opt.disabled}
                              id={`break_policy_${iv.key}_${oi}`}
                              name={`break_policy_${iv.key}`}
                              onChange={() => { updateIntervalDraft(iv.key, opt.val); }}
                              type="radio"
                            />
                            <div>
                              <span className="text-xs font-bold text-slate-700 dark:text-slate-200 block">{opt.label}</span>
                              <span className="text-[10px] text-slate-500 dark:text-slate-400 block">{opt.help}</span>
                            </div>
                          </label>
                        ))}
                      </div>
                    </div>

                    {/* Grace period & Early OT */}
                    <div className="border-t border-slate-100 dark:border-slate-800 pt-3 space-y-3">
                      <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
                        {t("shifts.modal.ot_grace_config", "Chính sách đi muộn & Tăng ca")}
                      </span>

                      <div>
                        <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">{t("shifts.grace_period")}</label>
                        <input
                          className="w-full mt-1 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
                          min="0"
                          onChange={(e) => { updateIntervalDraft(iv.key, { gracePeriodMinutes: parseInt(e.target.value) || 0 }); }}
                          type="number"
                          value={iv.gracePeriodMinutes}
                        />
                      </div>

                      <div className="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-800/40 rounded-xl">
                        <div>
                          <span className="text-xs font-bold text-slate-600 dark:text-slate-200 block">{t("shifts.modal.allow_early_ot", "Cho phép tính tăng ca sớm")}</span>
                          <span className="text-[10px] text-slate-500 dark:text-slate-400">{t("shifts.modal.allow_early_ot_help", "Tính thời gian làm việc trước ca là tăng ca sớm")}</span>
                        </div>
                        <div
                          aria-checked={iv.allowEarlyOt}
                          className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 cursor-pointer ${iv.allowEarlyOt ? 'bg-primary-600' : 'bg-slate-200 dark:bg-slate-700'}`}
                          onClick={() => { updateIntervalDraft(iv.key, { allowEarlyOt: !iv.allowEarlyOt }); }}
                          onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); updateIntervalDraft(iv.key, { allowEarlyOt: !iv.allowEarlyOt }); } }}
                          role="switch"
                          tabIndex={0}
                        >
                          <span className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white dark:bg-slate-900 transition-transform ${iv.allowEarlyOt ? 'translate-x-5' : 'translate-x-1'}`} />
                        </div>
                      </div>

                      {iv.allowEarlyOt ? (
                        <div>
                          <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">{t("shifts.modal.max_early_ot", "Giới hạn tăng ca sớm tối đa (phút)")}</label>
                          <input
                            className="w-full mt-1 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
                            min="1"
                            onChange={(e) => { updateIntervalDraft(iv.key, { maxEarlyOtMinutes: parseInt(e.target.value) || null }); }}
                            type="number"
                            value={iv.maxEarlyOtMinutes ?? ""}
                          />
                        </div>
                      ) : null}
                    </div>

                    {/* OT Aggregation Mode */}
                    <div className="border-t border-slate-100 dark:border-slate-800 pt-3 space-y-2">
                      <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
                        {t("work_segments.ot_aggregation_mode.label", "Cách tính tăng ca")}
                      </span>
                      {[
                        { val: "aggregate" as const, label: t("work_segments.ot_aggregation_mode.aggregate", "Gộp (Tổng cả ngày)") },
                        { val: "per_segment" as const, label: t("work_segments.ot_aggregation_mode.per_segment", "Theo từng phiên") },
                      ].map(opt => (
                        <label className="flex items-start gap-3 p-3 rounded-xl border cursor-pointer hover:bg-slate-50 dark:hover:bg-slate-800/40 transition-colors" key={opt.val}>
                          <input
                            checked={iv.otAggregationMode === opt.val}
                            className="mt-0.5"
                            name={`otMode_${iv.key}`}
                            onChange={() => { updateIntervalDraft(iv.key, { otAggregationMode: opt.val }); }}
                            type="radio"
                          />
                          <span className="text-sm font-bold text-slate-700 dark:text-slate-250">{opt.label}</span>
                        </label>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Card 3: Correction Config (Only in Edit Mode) */}
          {isEditMode ? (
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
                  onChange={(e) => { setIsCorrection(e.target.checked); }}
                  type="checkbox"
                />
                <label className="text-[10px] font-bold text-amber-800 dark:text-amber-300 uppercase tracking-widest cursor-pointer select-none" htmlFor="editor_is_correction">
                  {t("shifts.modal.correction_warning", "CẬP NHẬT TRỰC TIẾP LÊN CA GỐC (ÁP DỤNG HỒI TỐ)")}
                </label>
              </div>

              {!isCorrection ? (
                <div className="space-y-2 pt-2 animate-in fade-in slide-in-from-top-2 duration-300">
                  <label className="text-[10px] font-bold text-amber-750 dark:text-amber-400 uppercase tracking-widest block" htmlFor="correction_effective_date_input">
                    {t("shifts.modal.effective_date_label", "Ngày hiệu lực mới")}
                  </label>
                  <input
                    className="w-full bg-white dark:bg-slate-900 border border-amber-250 dark:border-amber-900 rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-amber-500 transition-all outline-none"
                    id="correction_effective_date_input"
                    onChange={(e) => { setCorrectionEffectiveDate(e.target.value); }}
                    type="date"
                    value={correctionEffectiveDate}
                  />
                  <Typography className="text-[9px] font-bold text-amber-600 dark:text-amber-350 leading-tight italic px-1" variant="p">
                    {t("shifts.modal.effective_date_desc", "Các thay đổi sẽ chỉ có hiệu lực từ ngày được chọn trở đi.")}
                  </Typography>
                </div>
              ) : null}
            </div>
          ) : null}
        </div>

        {/* Right Column: Visual Timeline Preview */}
        <div className="space-y-8">
          <ShiftTimelinePreview
            intervals={intervalDrafts}
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
