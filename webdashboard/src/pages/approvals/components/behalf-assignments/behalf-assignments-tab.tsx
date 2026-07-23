import { useQuery } from "@tanstack/react-query";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import type { BehalfAssignment, Employee } from "@/lib/api/types";

import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { useBehalfAssignments } from "@/hooks/use-behalf-assignments";
import { fetchDepartments } from "@/lib/api";
import { logError } from "@/lib/logger";

import type { AssignmentType, BehalfAssignmentInput } from "./types";

import { AddAssignmentForm } from "./add-assignment-form";
import { AssignmentItem } from "./assignment-item";
import { BehalfAssignmentsEmptyState, BehalfAssignmentsHeader } from "./behalf-assignments-header";

export function BehalfAssignmentsTab({
  wsId,
  assignmentType = "ot",
}: {
  wsId: string;
  assignmentType?: AssignmentType;
}) {
  const { t } = useTranslation();
  const toast = useToast();

  const {
    assignments,
    isLoading: assignmentsLoading,
    createAssignment,
    isCreating,
    deleteAssignment,
    isDeleting,
  } = useBehalfAssignments(assignmentType);

  const { data: departments = [], isLoading: deptsLoading } = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId),
    enabled: !!wsId,
  });

  const [isOpen, setIsOpen] = useState(false);
  const [assigneeId, setAssigneeId] = useState("");
  const [selectedAssigneeEmployee, setSelectedAssigneeEmployee] = useState<Employee | null>(null);
  const [targetType, setTargetType] = useState<"employee" | "department">("employee");
  const [targetEmployeeId, setTargetEmployeeId] = useState("");
  const [targetDepartmentId, setTargetDepartmentId] = useState("");

  const validateForm = (): boolean => {
    if (!assigneeId) {
      toast.error(t("behalf_assignments.toasts.select_assignee"));
      return false;
    }

    if (targetType === "employee" && !targetEmployeeId) {
      toast.error(t("behalf_assignments.toasts.select_target_employee"));
      return false;
    }

    if (targetType === "department" && !targetDepartmentId) {
      toast.error(t("behalf_assignments.toasts.select_target_department"));
      return false;
    }

    return true;
  };

  const handleCreate = async (e: React.SyntheticEvent) => {
    e.preventDefault();
    if (!validateForm()) return;

    if (!selectedAssigneeEmployee?.user_id) {
      toast.error(t("behalf_assignments.toasts.no_linked_user"));
      return;
    }

    try {
      const payload: BehalfAssignmentInput = {
        assignee_user_id: selectedAssigneeEmployee.user_id,
        target_department_id: targetType === "department" ? targetDepartmentId : null,
        target_employee_id: targetType === "employee" ? targetEmployeeId : null,
      };
      await createAssignment(payload);
      toast.success(t("behalf_assignments.toasts.create_success"));
      setIsOpen(false);
      setAssigneeId("");
      setSelectedAssigneeEmployee(null);
      setTargetEmployeeId("");
      setTargetDepartmentId("");
    } catch (err: unknown) {
      logError(err, { component: "BehalfAssignmentsTab" });
      const msg = err instanceof Error ? err.message : "Failed to create assignment";
      toast.error(msg);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteAssignment(id);
      toast.success(t("behalf_assignments.toasts.delete_success"));
    } catch (err: unknown) {
      logError(err, { component: "BehalfAssignmentsTab" });
      const msg = err instanceof Error ? err.message : "Failed to delete assignment";
      toast.error(msg);
    }
  };

  const isLoading = assignmentsLoading || deptsLoading;

  if (isLoading && assignments.length === 0) {
    return (
      <div className="p-32 flex flex-col items-center gap-6 animate-pulse">
        <div className="w-12 h-12 border-4 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
        <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em]">
          {t("common.loading")}
        </span>
      </div>
    );
  }

  const departmentOptions = departments.map((dept) => ({
    description: dept.description,
    label: dept.name,
    value: dept.id,
  }));

  return (
    <div className="p-8 space-y-6">
      <BehalfAssignmentsHeader
        assignmentType={assignmentType}
        onAddClick={() => {
          setIsOpen(true);
        }}
      />

      {assignments.length === 0 ? (
        <BehalfAssignmentsEmptyState assignmentType={assignmentType} />
      ) : (
        <div className="grid gap-4 animate-in fade-in duration-500">
          {assignments.map((assignment: BehalfAssignment) => (
            <AssignmentItem
              assignment={assignment}
              assignmentType={assignmentType}
              departments={departments}
              isDeleting={isDeleting}
              key={assignment.id}
              onDelete={handleDelete}
            />
          ))}
        </div>
      )}

      <Modal
        isOpen={isOpen}
        onClose={() => {
          setIsOpen(false);
        }}
        title={t("behalf_assignments.modal.add_title")}
      >
        <AddAssignmentForm
          assigneeId={assigneeId}
          assignmentType={assignmentType}
          departmentOptions={departmentOptions}
          departments={departments}
          isCreating={isCreating}
          onCancel={() => {
            setIsOpen(false);
          }}
          onSelectAssigneeEmployee={setSelectedAssigneeEmployee}
          onSubmit={handleCreate}
          setAssigneeId={setAssigneeId}
          setTargetDepartmentId={setTargetDepartmentId}
          setTargetEmployeeId={setTargetEmployeeId}
          setTargetType={setTargetType}
          targetDepartmentId={targetDepartmentId}
          targetEmployeeId={targetEmployeeId}
          targetType={targetType}
          wsId={wsId}
        />
      </Modal>
    </div>
  );
}
