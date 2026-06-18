import { useBlocker, useNavigate, useParams } from "@tanstack/react-router";
import { ArrowLeft, Save } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import ConfirmDialog from "@/components/ui/confirm-dialog";
import { Typography } from "@/components/ui/typography";

import { AssignmentRuleForm } from "./assignment-rules/components/assignment-rule-form";
import { useAssignmentRuleForm } from "./assignment-rules/hooks/use-assignment-rule-form";

export default function AssignmentRuleEditorPage() {
  const { ruleId } = useParams({ strict: false });
  const navigate = useNavigate();

  const {
    t,
    wsId,
    isEditMode,
    rulesLoading,
    patterns,
    departments,
    employers,
    locations,
    name,
    setName,
    patternId,
    setPatternId,
    priority,
    setPriority,
    conditions,
    setConditions,
    startDate,
    setStartDate,
    endDate,
    setEndDate,
    isIndefinite,
    setIsIndefinite,
    handleSubmit,
    isPending,
    isDirty,
  } = useAssignmentRuleForm(ruleId);

  const blocker = useBlocker({
    shouldBlockFn: () => isDirty,
    withResolver: true,
  });

  const handleCancel = () => {
    void navigate({ to: "/assignment-rules" });
  };

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  if (isEditMode && rulesLoading) {
    return (
      <div className="p-12 space-y-8 animate-pulse max-w-4xl mx-auto">
        <div className="h-10 w-64 bg-slate-200 dark:bg-slate-800 rounded-full" />
        <div className="h-96 bg-slate-200 dark:bg-slate-800 rounded-[2.5rem]" />
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6 pb-20 animate-in fade-in duration-500">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-4">
          <Button
            className="p-3 bg-white dark:bg-slate-900 border border-slate-105 dark:border-slate-850 hover:bg-slate-50 dark:hover:bg-slate-800 rounded-2xl shadow-sm text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200 active:scale-95 transition-all"
            onClick={handleCancel}
            type="button"
            variant="ghost"
          >
            <ArrowLeft size={18} />
          </Button>
          <div>
            <Typography
              className="text-2xl font-bold tracking-tight text-slate-850 dark:text-slate-100 normal-case mb-0.5"
              variant="h3"
            >
              {isEditMode ? t("rules.edit_title", "Edit Rule") : t("rules.create_title", "Create New Rule")}
            </Typography>
            <Typography
              className="text-xs text-slate-500 dark:text-slate-400 font-medium normal-case tracking-normal"
              variant="small"
            >
              {t("rules.form_subtitle", "Define when this pattern should be applied")}
            </Typography>
          </div>
        </div>

        <div className="flex items-center gap-4">
          <Button onClick={handleCancel} size="lg" variant="outline">
            {t("common.cancel").toUpperCase()}
          </Button>
          <Button
            className="flex items-center gap-2.5 shadow-md shadow-primary-500/10"
            disabled={isPending}
            loading={isPending}
            onClick={() => {
              handleSubmit();
            }}
            size="lg"
            variant="secondary"
          >
            <Save size={16} />
            {isEditMode ? t("common.save_changes").toUpperCase() : t("common.create").toUpperCase()}
          </Button>
        </div>
      </div>

      {/* Form Card */}
      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] p-10 shadow-sm">
        <AssignmentRuleForm
          conditions={conditions}
          departments={departments}
          employers={employers}
          endDate={endDate}
          isIndefinite={isIndefinite}
          locations={locations}
          name={name}
          patternId={patternId}
          patterns={patterns}
          priority={priority}
          setConditions={setConditions}
          setEndDate={setEndDate}
          setIsIndefinite={setIsIndefinite}
          setName={setName}
          setPatternId={setPatternId}
          setPriority={setPriority}
          setStartDate={setStartDate}
          startDate={startDate}
        />
      </div>

      {/* Unsaved Changes Blocker */}
      <ConfirmDialog
        cancelLabel={t("patterns.form.unsaved_changes_cancel", "Stay")}
        confirmLabel={t("patterns.form.unsaved_changes_confirm", "Leave")}
        isOpen={blocker.status === "blocked"}
        message={t("patterns.form.unsaved_changes", "You have unsaved changes. Are you sure you want to leave this page?")}
        onClose={() => {
          blocker.reset?.();
        }}
        onConfirm={() => {
          blocker.proceed?.();
        }}
        title={t("common.warning", "Warning")}
        variant="warning"
      />
    </div>
  );
}
