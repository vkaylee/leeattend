import { useQuery } from "@tanstack/react-query";
import { ArrowRight, Loader2, Plus, ShieldAlert, Trash2 } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { EmployeeSearch } from "@/components/common/employee-search";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useBehalfAssignments } from "@/hooks/use-behalf-assignments";
import { useAllEmployees } from "@/hooks/use-employees";
import { useLeaveBehalfAssignments } from "@/hooks/use-leave-behalf-assignments";
import { useAttendanceBehalfAssignments } from "@/hooks/use-attendance-behalf-assignments";
import { fetchDepartments } from "@/lib/api";
import { BehalfAssignment } from "@/lib/api/types";

interface AddAssignmentFormProps {
  assigneeId: string;
  assignmentType: "ot" | "leave" | "attendance_adjustment";
  departmentOptions: { description?: string | null; label: string; value: string }[];
  isCreating: boolean;
  onCancel: () => void;
  onSubmit: (e: React.SyntheticEvent) => void;
  setAssigneeId: (id: string) => void;
  setTargetDepartmentId: (id: string) => void;
  setTargetEmployeeId: (id: string) => void;
  setTargetType: (type: "employee" | "department") => void;
  targetDepartmentId: string;
  targetEmployeeId: string;
  targetType: "employee" | "department";
  wsId: string;
}

interface AssignmentItemProps {
  assignment: BehalfAssignment;
  assignmentType: "ot" | "leave" | "attendance_adjustment";
  departments: {
    description?: string | null;
    id: string;
    name: string;
  }[];
  employees: {
    employee_id_number?: string | null;
    full_name: string;
    id: string;
  }[];
  isDeleting: boolean;
  onDelete: (id: string) => Promise<void>;
}

interface BehalfAssignmentInput {
  assignee_employee_id: string;
  target_department_id?: string | null;
  target_employee_id?: string | null;
}

interface BehalfAssignmentsEmptyStateProps {
  assignmentType: "ot" | "leave" | "attendance_adjustment";
}

interface BehalfAssignmentsHeaderProps {
  assignmentType: "ot" | "leave" | "attendance_adjustment";
  onAddClick: () => void;
}

interface TargetScopeDisplayProps {
  targetDept: { description?: string | null; name: string } | null;
  targetEmp: { employee_id_number?: string | null; full_name: string } | null;
}

function TargetScopeDisplay({ targetEmp, targetDept }: TargetScopeDisplayProps) {
  if (targetEmp) {
    return (
      <div className="mt-0.5">
        <span className="inline-flex items-center px-2 py-0.5 rounded-lg text-xs font-semibold bg-indigo-50 dark:bg-indigo-950/40 text-indigo-700 dark:text-indigo-400 mr-2">
          Employee
        </span>
        <span className="font-bold text-slate-700 dark:text-slate-300 text-sm">
          {targetEmp.full_name}
        </span>
        {targetEmp.employee_id_number ? (
          <span className="text-[10px] text-slate-400 font-mono ml-2">
            ({targetEmp.employee_id_number})
          </span>
        ) : null}
      </div>
    );
  }

  if (targetDept) {
    return (
      <div className="mt-0.5">
        <span className="inline-flex items-center px-2 py-0.5 rounded-lg text-xs font-semibold bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-indigo-400 mr-2">
          Department
        </span>
        <span className="font-bold text-slate-700 dark:text-slate-300 text-sm">
          {targetDept.name}
        </span>
        {targetDept.description ? (
          <span className="text-[10px] text-slate-400 ml-2">
            ({targetDept.description})
          </span>
        ) : null}
      </div>
    );
  }

  return <span className="text-slate-400 text-sm mt-0.5 italic">None</span>;
}

function AssignmentItem({
  assignment,
  assignmentType,
  departments,
  employees,
  isDeleting,
  onDelete,
}: AssignmentItemProps) {
  const assignee = employees.find((e) => e.id === assignment.assignee_employee_id);
  const targetEmp = employees.find((e) => e.id === assignment.target_employee_id) ?? null;
  const targetDept = departments.find((d) => d.id === assignment.target_department_id) ?? null;

  return (
    <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-100 dark:border-slate-800 p-5 flex items-center justify-between shadow-sm hover:shadow-md transition-all duration-200">
      <div className="flex flex-col md:flex-row md:items-center gap-4 md:gap-8 flex-1">
        <div className="flex flex-col min-w-[200px]">
          <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
            {assignmentType === "ot" ? "Authorized Claimant" : "Authorized Requester"}
          </span>
          <span className="font-bold text-slate-800 dark:text-slate-200 text-sm mt-0.5">
            {assignee?.full_name ?? "Unknown Employee"}
          </span>
          {assignee?.employee_id_number ? (
            <span className="text-[10px] text-slate-400 font-mono mt-0.5">
              {assignee.employee_id_number}
            </span>
          ) : null}
        </div>

        <div className="flex items-center gap-2 text-slate-300 dark:text-slate-600 self-center hidden md:flex">
          <ArrowRight className="w-5 h-5" />
        </div>

        <div className="flex flex-col">
          <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
            {assignmentType === "ot" ? "Claim Target Scope" : "Request Target Scope"}
          </span>
          <TargetScopeDisplay targetDept={targetDept} targetEmp={targetEmp} />
        </div>
      </div>

      <Button
        className="p-2 text-slate-400 hover:text-rose-600 dark:text-slate-600 dark:hover:text-rose-400 rounded-xl hover:bg-rose-50 dark:hover:bg-rose-950/20 transition-all border-none shadow-none h-10 w-10 flex items-center justify-center shrink-0"
        disabled={isDeleting}
        onClick={() => {
          void onDelete(assignment.id);
        }}
        variant="ghost"
      >
        <Trash2 className="w-4 h-4" />
      </Button>
    </div>
  );
}

function AddAssignmentForm({
  assigneeId,
  assignmentType,
  departmentOptions,
  isCreating,
  onCancel,
  onSubmit,
  setAssigneeId,
  setTargetDepartmentId,
  setTargetEmployeeId,
  setTargetType,
  targetDepartmentId,
  targetEmployeeId,
  targetType,
  wsId,
}: AddAssignmentFormProps) {
  return (
    <form className="space-y-6 animate-in fade-in zoom-in-95 duration-250" onSubmit={onSubmit}>
      <div className="space-y-2">
        <label
          className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest"
          htmlFor="submitter-select"
        >
          {assignmentType === "ot"
            ? "Authorized Claimant (Employee)"
            : "Authorized Requester (Employee)"}
        </label>
        <EmployeeSearch
          clearOnEscape
          emptyQueryBehavior="prompt-to-type"
          id="submitter-select"
          onChange={(val) => { setAssigneeId(typeof val === "string" ? val : val[0] ?? ""); }}
          placeholder="Search and select submitter..."
          value={assigneeId}
          workspaceId={wsId}
        />
        <Typography className="text-[10px] text-slate-400 mt-1" variant="p">
          {assignmentType === "ot"
            ? "Select the workspace employee who is allowed to claim on behalf."
            : "Select the workspace employee who is allowed to submit leave requests on behalf."}
        </Typography>
      </div>

      <div className="space-y-3">
        <span className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
          Target Scope Type
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
            Employee
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
            Department
          </label>
        </div>
      </div>

      {targetType === "employee" ? (
        <div className="space-y-2">
          <label
            className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest"
            htmlFor="target-employee-select"
          >
            Target Employee
          </label>
          <EmployeeSearch
            clearOnEscape
            emptyQueryBehavior="prompt-to-type"
            id="target-employee-select"
            onChange={(val) => { setTargetEmployeeId(typeof val === "string" ? val : val[0] ?? ""); }}
            placeholder="Search and select target employee..."
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
            Target Department
          </label>
          <Combobox
            emptyText="No departments found"
            id="target-department-select"
            onChange={setTargetDepartmentId}
            options={departmentOptions}
            placeholder="Search and select target department..."
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
          Cancel
        </Button>
        <Button
          className="normal-case tracking-normal gap-2 h-10 px-6 bg-primary-600 hover:bg-primary-700 text-white rounded-xl font-bold text-sm shadow-md transition-all duration-300"
          disabled={isCreating}
          type="submit"
          variant="primary"
        >
          {isCreating ? <Loader2 className="w-4 h-4 animate-spin" /> : null}
          Create Assignment
        </Button>
      </div>
    </form>
  );
}

function BehalfAssignmentsHeader({
  assignmentType,
  onAddClick,
}: BehalfAssignmentsHeaderProps) {
  return (
    <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 border-b border-slate-50 dark:border-slate-800 pb-6">
      <div>
        <Typography className="text-lg font-bold text-slate-800 dark:text-slate-200" variant="h3">
          {assignmentType === "ot"
            ? "Behalf Claim Assignments"
            : "Behalf Leave Request Assignments"}
        </Typography>
        <Typography className="text-xs text-slate-500 mt-1" variant="p">
          {assignmentType === "ot"
            ? "Configure which employees are authorized to claim overtime on behalf of specific employees or entire departments."
            : "Configure which employees are authorized to request leave on behalf of specific employees or entire departments."}
        </Typography>
      </div>
      <Button
        className="normal-case tracking-normal gap-2 h-10 px-6 bg-primary-600 hover:bg-primary-700 text-white rounded-xl font-bold text-sm shadow-md transition-all duration-300 transform active:scale-95"
        onClick={onAddClick}
        variant="primary"
      >
        <Plus className="w-4 h-4" />
        Add Assignment
      </Button>
    </div>
  );
}

function BehalfAssignmentsEmptyState({
  assignmentType,
}: BehalfAssignmentsEmptyStateProps) {
  return (
    <div className="py-24 text-center flex flex-col items-center border-4 border-dashed border-slate-50 dark:border-slate-800 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30">
      <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-xl border border-slate-100 dark:border-slate-800 mb-6">
        <ShieldAlert className="text-slate-300 dark:text-slate-600" size={32} />
      </div>
      <Typography className="text-slate-700 dark:text-slate-100 font-extrabold text-xl tracking-tight" variant="h3">
        {assignmentType === "ot"
          ? "No Behalf Assignments Configured"
          : assignmentType === "leave"
            ? "No Behalf Leave Assignments Configured"
            : "No Behalf Adjustment Assignments Configured"}
      </Typography>
      <Typography className="text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-widest mt-2 opacity-50 max-w-sm" variant="p">
        {assignmentType === "ot"
          ? "Users with Claim on Behalf permission will fall back to their managed departments. Assign specific employees or departments to scope their claim access."
          : assignmentType === "leave"
            ? "Users with Submit on Behalf permission will fall back to their managed departments. Assign specific employees or departments to scope their leave request access."
            : "Users with Submit on Behalf permission will fall back to their managed departments. Assign specific employees or departments to scope their adjustment access."}
      </Typography>
    </div>
  );
}

export function BehalfAssignmentsTab({
  wsId,
  assignmentType = "ot",
}: {
  wsId: string;
  assignmentType?: "ot" | "leave" | "attendance_adjustment";
}) {
  const { t } = useTranslation();
  const toast = useToast();

  const otHooks = useBehalfAssignments();
  const leaveHooks = useLeaveBehalfAssignments();
  const adjustmentHooks = useAttendanceBehalfAssignments();

  const hooksByType = { ot: otHooks, leave: leaveHooks, attendance_adjustment: adjustmentHooks };

  const {
    assignments,
    isLoading: assignmentsLoading,
    createAssignment,
    isCreating,
    deleteAssignment,
    isDeleting,
  } = hooksByType[assignmentType];

  const { employees: allEmployees, isLoading: allEmployeesLoading } = useAllEmployees(wsId);
  const employeesLoading = allEmployeesLoading;

  const { data: departments = [], isLoading: deptsLoading } = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId),
    enabled: !!wsId,
  });

  const [isOpen, setIsOpen] = useState(false);
  const [assigneeId, setAssigneeId] = useState("");
  const [targetType, setTargetType] = useState<"employee" | "department">("employee");
  const [targetEmployeeId, setTargetEmployeeId] = useState("");
  const [targetDepartmentId, setTargetDepartmentId] = useState("");

  const validateForm = (): boolean => {
    if (!assigneeId) {
      toast.error("Please select an assignee employee");
      return false;
    }

    if (targetType === "employee" && !targetEmployeeId) {
      toast.error("Please select a target employee");
      return false;
    }

    if (targetType === "department" && !targetDepartmentId) {
      toast.error("Please select a target department");
      return false;
    }

    return true;
  };

  const handleCreate = async (e: React.SyntheticEvent) => {
    e.preventDefault();
    if (!validateForm()) return;

    try {
      const payload: BehalfAssignmentInput = {
        assignee_employee_id: assigneeId,
        target_department_id: targetType === "department" ? targetDepartmentId : null,
        target_employee_id: targetType === "employee" ? targetEmployeeId : null,
      };
      await createAssignment(payload);
      toast.success("Behalf assignment created successfully");
      setIsOpen(false);
      setAssigneeId("");
      setTargetEmployeeId("");
      setTargetDepartmentId("");
    } catch (err: unknown) {
      console.error(err);
      const msg = err instanceof Error ? err.message : "Failed to create assignment";
      toast.error(msg);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteAssignment(id);
      toast.success("Assignment deleted successfully");
    } catch (err: unknown) {
      console.error(err);
      const msg = err instanceof Error ? err.message : "Failed to delete assignment";
      toast.error(msg);
    }
  };

  const isLoading = assignmentsLoading || employeesLoading || deptsLoading;

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
          {assignments.map((assignment) => (
            <AssignmentItem
              assignment={assignment}
              assignmentType={assignmentType}
              departments={departments}
              employees={allEmployees}
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
        title="Add Behalf Assignment"
      >
        <AddAssignmentForm
          assigneeId={assigneeId}
          assignmentType={assignmentType}
          departmentOptions={departmentOptions}
          isCreating={isCreating}
          onCancel={() => {
            setIsOpen(false);
          }}
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
