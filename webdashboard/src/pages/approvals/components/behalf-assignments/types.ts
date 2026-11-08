import type { BehalfAssignment, Department, OrganizationMember } from "@/lib/api/types";

export type AssignmentType = "ot" | "leave" | "attendance_adjustment" | "overtime_request";

export interface AddAssignmentFormProps { assigneeId: string;
assigneeOptions: { description?: string | null; label: string; value: string }[];
assignmentType: AssignmentType;
departmentOptions: { description?: string | null; label: string; value: string }[];
departments: Department[];
isCreating: boolean;
isLoadingAssignees: boolean;
onCancel: () => void;
onSubmit: (e: React.SyntheticEvent) => void;
setAssigneeId: (id: string) => void;
setTargetDepartmentId: (id: string) => void;
setTargetEmployeeId: (id: string) => void;
setTargetType: (type: "employee" | "department") => void;
targetDepartmentId: string;
targetEmployeeId: string;
targetType: "employee" | "department";  }

export interface AssignmentItemProps {
  assignees: OrganizationMember[];
  assignmentType: AssignmentType;
  departments: {
    description?: string | null;
    id: string;
    name: string;
  }[];
  isDeleting: boolean;
  onDelete: (id: string) => Promise<void>;
}

export interface BehalfAssignmentInput {
  assignee_user_id: string;
  target_department_id?: string | null;
  target_employee_id?: string | null;
}

export interface BehalfAssignmentsEmptyStateProps {
  assignmentType: AssignmentType;
}

export interface BehalfAssignmentsHeaderProps {
  assignmentType: AssignmentType;
  onAddClick: () => void;
}

export interface TargetScopeDisplayProps {
  targetDept: { description?: string | null; name: string } | null;
  targetEmp: { employee_id_number?: string | null; full_name: string } | null;
}
