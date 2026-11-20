import type { Employee } from "@/lib/api";

export type EmployeeModalState =
  | { type: "edit"; employee: Employee }
  | { type: "transfer"; employee: Employee; departmentId?: string }
  | { type: "terminate"; employee: Employee }
  | { type: "rehire"; employee: Employee }
  | { type: "import" }
  | { type: "import-termination" }
  | null;
