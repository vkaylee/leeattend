/* eslint-disable max-lines-per-function, complexity */
import { Link, useNavigate } from "@tanstack/react-router";
import { ArrowLeft, Check, Layers, Sparkles } from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { useLeaveTypeManager } from "@/components/leave/hooks/use-leave-type-manager";
import { LeaveTypeForm } from "@/components/leave/leave-policy-components/leave-type-form";
import { RestoreTypeModal } from "@/components/leave/leave-policy-components/restore-type-modal";
import { TemplateGallery } from "@/components/leave/leave-policy-components/template-gallery";
import { LeaveTemplate } from "@/components/leave/leave-templates";
import { LeaveTypeCard } from "@/components/leave/leave-type-manager";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { LeaveType } from "@/lib/api/types";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

type Step = "template" | "form";

export function NewLeaveTypePage() {
  const { t } = useTranslation();
  const navigate = useNavigate();

  const [step, setStep] = useState<Step>("template");
  const [formData, setFormData] = useState({ code: "", name: "", is_paid: true });
  const [selectedTemplateName, setSelectedTemplateName] = useState<string | null>(null);

  const navigateBack = () => {
    void navigate({
      to: ROUTES.SETTINGS.LEAVE,
      search: { tab: "types" },
    });
  };

  const {
    leaveTypes,
    isTypesLoading,
    createLeaveType,
    restoreLeaveType,
    restoringType,
    setRestoringType,
    isCreating,
    isRestoring,
  } = useLeaveTypeManager({
    onCreateSuccess: navigateBack,
    onRestoreSuccess: navigateBack,
  });

  const existingCodes = useMemo(
    () => leaveTypes.map((lt) => lt.code.toLowerCase()),
    [leaveTypes]
  );

  const handleApplyTemplate = (template: LeaveTemplate) => {
    const updated = {
      code: template.code,
      name: template.name,
      is_paid: template.is_paid,
    };
    setFormData(updated);
    setSelectedTemplateName(template.name);
    setStep("form");
  };

  const handleSkipToManual = () => {
    setSelectedTemplateName(null);
    setStep("form");
  };

  const previewType: LeaveType = useMemo(
    () => ({
      id: "preview-id",
      code: formData.code.trim() || "CODE",
      name: formData.name.trim() || t("settings.leave.new_type.preview_placeholder_name", "New Leave Type"),
      is_paid: formData.is_paid,
      policy_config: {
        approval_required: true,
        requires_document: false,
        allow_negative_balance: false,
      },
    } as LeaveType),
    [formData, t]
  );

  return (
    <div className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500 w-full max-w-7xl mx-auto pb-12">
      {/* 1. Breadcrumbs & Return Link */}
      <nav aria-label={t("common.breadcrumb", "Breadcrumb")} className="flex items-center gap-2">
        <Link
          className="inline-flex items-center gap-1.5 text-xs font-semibold text-slate-500 hover:text-primary-600 dark:text-slate-400 dark:hover:text-primary-400 transition-colors py-1.5 px-3 -ml-2 rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800/60"
          search={{ tab: "types" }}
          to={ROUTES.SETTINGS.LEAVE}
        >
          <ArrowLeft className="w-4 h-4" />
          <span>{t("settings.leave.new_type.back_to_leave", "Back to Leave Settings")}</span>
        </Link>
      </nav>

      {/* 2. Page Header & Stepper */}
      <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-6 pb-6 border-b border-slate-200/80 dark:border-slate-800/80">
        <div>
          <div className="flex items-center gap-3">
            <div className="w-11 h-11 rounded-2xl bg-gradient-to-br from-primary-500/10 to-primary-600/20 dark:from-primary-950/70 dark:to-primary-900/40 border border-primary-200/70 dark:border-primary-800/60 flex items-center justify-center shrink-0 text-primary-600 dark:text-primary-400 shadow-xs">
              <Sparkles className="w-5 h-5" />
            </div>
            <div>
              <Typography className="text-2xl font-bold text-slate-900 dark:text-slate-100 tracking-tight" variant="h2">
                {t("settings.leave.new_type.title", "Create Leave Type")}
              </Typography>
              <Typography className="text-xs text-slate-500 dark:text-slate-400 mt-1 max-w-2xl leading-relaxed" variant="p">
                {step === "template"
                  ? t("settings.leave.gallery.subtitle", "Start faster with a professional setup")
                  : t("settings.leave.gallery.customize_subtitle", "Refining {{name}}", {
                      name: formData.name || t("settings.leave.gallery.customize", "Customize Leave Type"),
                    })}
              </Typography>
            </div>
          </div>
        </div>

        {/* Stepper Indicator */}
        <div className="flex items-center gap-2 bg-slate-100/80 dark:bg-slate-800/60 p-1.5 rounded-2xl border border-slate-200/60 dark:border-slate-700/60 shrink-0 self-start lg:self-auto">
          <Button
            aria-current={step === "template" ? "step" : undefined}
            className={cn(
              "px-4 py-2 rounded-xl text-xs font-semibold flex items-center gap-2.5 transition-all h-auto",
              step === "template"
                ? "bg-white dark:bg-slate-900 text-slate-900 dark:text-slate-100 shadow-2xs border border-slate-200/80 dark:border-slate-700/80"
                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-white/50 dark:hover:bg-slate-900/50"
            )}
            onClick={() => { setStep("template"); }}
            size="sm"
            type="button"
            variant="ghost"
          >
            <span
              className={cn(
                "w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold",
                step === "form" ? "bg-emerald-500 text-white" : "bg-primary-600 text-white"
              )}
            >
              {step === "form" ? <Check className="w-3 h-3 stroke-[3]" /> : "1"}
            </span>
            <span>{t("settings.leave.new_type.step_template", "1. Choose Template")}</span>
          </Button>

          <div className="w-4 h-px bg-slate-300 dark:bg-slate-700" />

          <Button
            aria-current={step === "form" ? "step" : undefined}
            className={cn(
              "px-4 py-2 rounded-xl text-xs font-semibold flex items-center gap-2.5 transition-all h-auto",
              step === "form"
                ? "bg-white dark:bg-slate-900 text-slate-900 dark:text-slate-100 shadow-2xs border border-slate-200/80 dark:border-slate-700/80"
                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-white/50 dark:hover:bg-slate-900/50"
            )}
            onClick={() => { setStep("form"); }}
            size="sm"
            type="button"
            variant="ghost"
          >
            <span
              className={cn(
                "w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold",
                step === "form"
                  ? "bg-primary-600 text-white"
                  : "bg-slate-200 dark:bg-slate-700 text-slate-600 dark:text-slate-400"
              )}
            >
              2
            </span>
            <span>{t("settings.leave.new_type.step_form", "2. Configure Details")}</span>
          </Button>
        </div>
      </div>

      {/* 3. Main Step Content */}
      {isTypesLoading ? (
        <div className="py-24 text-center text-slate-400 dark:text-slate-500 animate-pulse font-bold uppercase tracking-widest text-xs">
          {t("settings.leave.type_manager.loading", "Loading leave types...")}
        </div>
      ) : step === "template" ? (
        <TemplateGallery
          className="w-full"
          onSelectTemplate={handleApplyTemplate}
          onSkip={handleSkipToManual}
        />
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
          {/* Left Column: Form Configuration */}
          <div className="lg:col-span-7 space-y-6">
            <div className="bg-white dark:bg-slate-900 rounded-3xl border border-slate-200/80 dark:border-slate-800/80 p-6 sm:p-8 lg:p-9 shadow-2xs">
              {Boolean(selectedTemplateName) && (
                <div className="mb-6 flex items-center justify-between p-4 rounded-2xl bg-primary-50/70 dark:bg-primary-950/40 border border-primary-100 dark:border-primary-900/50">
                  <div className="flex items-center gap-2.5">
                    <Layers className="w-4 h-4 text-primary-600 dark:text-primary-400 shrink-0" />
                    <span className="text-xs font-semibold text-primary-900 dark:text-primary-200">
                      {t("settings.leave.new_type.selected_template", "Using Template")}:{" "}
                      <span className="font-bold">{selectedTemplateName}</span>
                    </span>
                  </div>
                  <Button
                    className="text-[11px] text-primary-700 dark:text-primary-300 hover:underline h-auto p-0 font-semibold"
                    onClick={() => { setStep("template"); }}
                    type="button"
                    variant="ghost"
                  >
                    {t("settings.leave.type_form.change_template", "Change Template")}
                  </Button>
                </div>
              )}

              <LeaveTypeForm
                existingCodes={existingCodes}
                initialData={formData}
                isSaving={isCreating}
                onBackToTemplates={() => { setStep("template"); }}
                onCancel={navigateBack}
                onChange={(data) => { setFormData(data); }}
                onSave={createLeaveType}
              />
            </div>
          </div>

          {/* Right Column: Live Card Preview & Enterprise Highlights */}
          <div className="lg:col-span-5 space-y-6 lg:sticky lg:top-6">
            <div className="bg-slate-50/70 dark:bg-slate-900/60 rounded-3xl border border-slate-200/80 dark:border-slate-800 p-6 sm:p-7 shadow-2xs space-y-5">
              <div className="flex items-center justify-between pb-3 border-b border-slate-200/60 dark:border-slate-800">
                <div className="flex items-center gap-2">
                  <span className="relative flex h-2 w-2">
                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
                    <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500" />
                  </span>
                  <Typography className="text-xs font-bold text-slate-800 dark:text-slate-200 uppercase tracking-widest" variant="h4">
                    {t("settings.leave.new_type.preview_title", "Live Organization Preview")}
                  </Typography>
                </div>
                <span className="text-[10px] font-semibold text-slate-400 dark:text-slate-500">
                  {t("settings.leave.new_type.preview_badge", "Realtime")}
                </span>
              </div>

              <div>
                <LeaveTypeCard type={previewType} />
              </div>

              <div className="pt-2 space-y-2.5">
                <Typography className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider" variant="p">
                  {t("settings.leave.new_type.features_title", "Included with this leave type:")}
                </Typography>
                <div className="grid grid-cols-1 gap-2.5 text-xs text-slate-600 dark:text-slate-400">
                  <div className="flex items-center gap-2.5">
                    <Check className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400 shrink-0" />
                    <span>{t("settings.leave.new_type.feature_audit", "Full immutable audit trail & ledger tracking")}</span>
                  </div>
                  <div className="flex items-center gap-2.5">
                    <Check className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400 shrink-0" />
                    <span>{t("settings.leave.new_type.feature_policy", "Custom accrual & rollover rules in Policies tab")}</span>
                  </div>
                  <div className="flex items-center gap-2.5">
                    <Check className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400 shrink-0" />
                    <span>{t("settings.leave.new_type.feature_roster", "Roster conflict detection & calendar badges")}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 4. Conflict / Restore Modal */}
      <RestoreTypeModal
        isRestoring={isRestoring}
        name={restoringType?.name ?? ""}
        onCancel={() => { setRestoringType(null); }}
        onConfirm={() => {
          if (restoringType) restoreLeaveType(restoringType.leave_type_id);
        }}
        open={restoringType !== null}
      />
    </div>
  );
}

export default NewLeaveTypePage;
