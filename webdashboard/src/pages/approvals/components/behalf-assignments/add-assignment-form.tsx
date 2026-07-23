import { Loader2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { EmployeeSearch } from "@/components/common/employee-search";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { Typography } from "@/components/ui/typography";

import type { AddAssignmentFormProps } from "./types";

// eslint-disable-next-line complexity, max-lines-per-function
export function AddAssignmentForm({
  assigneeId,
  assignmentType,
  departmentOptions,
  departments,
  isCreating,
  onCancel,
  onSubmit,
  setAssigneeId,
  onSelectAssigneeEmployee,
  setTargetDepartmentId,
  setTargetEmployeeId,
  setTargetType,
  targetDepartmentId,
  targetEmployeeId,
  targetType,
  wsId,
}: AddAssignmentFormProps) {
  const { t } = useTranslation();

  const submitterLabel = t(
    assignmentType === "ot"
      ? "behalf_assignments.form.assignee_label_ot"
      : assignmentType === "overtime_request"
        ? "behalf_assignments.form.assignee_label_overtime_request"
        : "behalf_assignments.form.assignee_label_other"
  );

  const submitterHelpText = t(
    assignmentType === "ot"
      ? "behalf_assignments.form.assignee_help_ot"
      : assignmentType === "overtime_request"
        ? "behalf_assignments.form.assignee_help_overtime_request"
        : assignmentType === "attendance_adjustment"
          ? "behalf_assignments.form.assignee_help_adjustment"
          : "behalf_assignments.form.assignee_help_leave"
  );

  return (
    <form className="space-y-6 animate-in fade-in zoom-in-95 duration-250" onSubmit={onSubmit}>
      <div className="space-y-2">
        <label
          className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest"
          htmlFor="submitter-select"
        >
          {submitterLabel}
        </label>
        <EmployeeSearch
          clearOnEscape
          departments={departments}
          emptyQueryBehavior="prompt-to-type"
          id="submitter-select"
          onChange={(val) => {
            const id = typeof val === "string" ? val : val[0] ?? "";
            setAssigneeId(id);
            if (!id) {
              onSelectAssigneeEmployee(null);
            }
          }}
          onSelectEmployee={onSelectAssigneeEmployee}
          permission={
            assignmentType === "ot"
              ? "overtime:claim_on_behalf"
              : assignmentType === "leave"
                ? "leave:requests:submit_on_behalf"
                : assignmentType === "attendance_adjustment"
                  ? "attendance:adjustments:submit_on_behalf"
                  : "overtime:request:create_on_behalf"
          }
          placeholder={t("behalf_assignments.form.assignee_placeholder")}
          value={assigneeId}
          workspaceId={wsId}
        />
        <Typography className="text-[10px] text-slate-400 mt-1" variant="p">
          {submitterHelpText}
        </Typography>
      </div>

      <div className="space-y-3">
        <span className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
          {t("behalf_assignments.form.target_type_label")}
        </span>
        <div className="flex gap-4">
          <label className="flex items-center gap-2 cursor-pointer text-sm text-slate-700 dark:text-slate-300 font-bold">
            <input
              checked={targetType === "employee"}
              className="text-primary-600 focus:ring-primary-500 border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900"
              onChange={() => {
                setTargetType("employee");
              }}
              type="radio"
            />
            {t("behalf_assignments.form.target_type_employee")}
          </label>
          <label className="flex items-center gap-2 cursor-pointer text-sm text-slate-700 dark:text-slate-300 font-bold">
            <input
              checked={targetType === "department"}
              className="text-primary-600 focus:ring-primary-500 border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900"
              onChange={() => {
                setTargetType("department");
              }}
              type="radio"
            />
            {t("behalf_assignments.form.target_type_department")}
          </label>
        </div>
      </div>

      {targetType === "employee" ? (
        <div className="space-y-2">
          <label
            className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest"
            htmlFor="target-employee-select"
          >
            {t("behalf_assignments.form.target_employee_label")}
          </label>
          <EmployeeSearch
            clearOnEscape
            departments={departments}
            emptyQueryBehavior="prompt-to-type"
            id="target-employee-select"
            onChange={(val) => { setTargetEmployeeId(typeof val === "string" ? val : val[0] ?? ""); }}
            placeholder={t("behalf_assignments.form.target_employee_placeholder")}
            value={targetEmployeeId}
            workspaceId={wsId}
          />
        </div>
      ) : (
        <div className="space-y-2">
          <label
            className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest"
            htmlFor="target-department-select"
          >
            {t("behalf_assignments.form.target_department_label")}
          </label>
          <Combobox
            emptyText={t("behalf_assignments.form.no_departments")}
            id="target-department-select"
            onChange={setTargetDepartmentId}
            options={departmentOptions}
            placeholder={t("behalf_assignments.form.target_department_placeholder")}
            value={targetDepartmentId}
          />
        </div>
      )}

      <div className="flex justify-end gap-3 pt-4 border-t border-slate-50 dark:border-slate-800">
        <Button
          className="normal-case tracking-normal h-10 px-6 rounded-xl font-bold text-sm bg-slate-100 hover:bg-slate-200 dark:bg-slate-850 dark:hover:bg-slate-800 text-slate-700 dark:text-slate-300 shadow-none border-none"
          onClick={onCancel}
          type="button"
          variant="ghost"
        >
          {t("common.cancel")}
        </Button>
        <Button
          className="normal-case tracking-normal gap-2 h-10 px-6 bg-primary-600 hover:bg-primary-700 text-white rounded-xl font-bold text-sm shadow-md transition-all duration-300"
          disabled={isCreating}
          type="submit"
          variant="primary"
        >
          {isCreating ? <Loader2 className="w-4 h-4 animate-spin" /> : null}
          {t("behalf_assignments.form.submit_btn")}
        </Button>
      </div>
    </form>
  );
}
