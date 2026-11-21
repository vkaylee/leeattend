import type { Employee } from "@/lib/api/types";

export function getActiveMonthYear(
  filterMode: string,
  startDate: string,
  specificMonth: { month: number; year: number },
): { month: number; year: number } {
  if (filterMode === "specific_month") {
    return specificMonth;
  }

  const [year, month] = startDate.split("-").map(Number);
  if (Number.isInteger(year) && Number.isInteger(month) && month >= 1 && month <= 12) {
    return { month, year };
  }

  return specificMonth;
}

interface MatrixEmployeeFilters {
  employeeStatus: string;
  searchQuery: string;
  selectedDepartmentId: string | null;
  selectedEmployerId: string | null;
}

interface LegacyEmployeeFields {
  department?: string;
  employee_code?: string;
}

function matchesMatrixSearch(employee: Employee, query: string): boolean {
  if (!query) {
    return true;
  }

  const legacyEmployee = employee as Employee & LegacyEmployeeFields;
  return [
    employee.full_name,
    employee.employee_id_number,
    employee.department_name ?? "",
    legacyEmployee.employee_code ?? "",
    legacyEmployee.department ?? "",
  ].some((value) => value.toLowerCase().includes(query));
}

function matchesEmployeeStatus(employee: Employee, employeeStatus: string): boolean {
  if (employeeStatus === "active") {
    return employee.active_employment_id !== null;
  }
  if (employeeStatus === "inactive") {
    return employee.active_employment_id === null;
  }
  return true;
}

export function filterMatrixEmployees(
  employees: Employee[],
  { employeeStatus, searchQuery, selectedDepartmentId, selectedEmployerId }: MatrixEmployeeFilters,
): Employee[] {
  const query = searchQuery.trim().toLowerCase();
  return employees.filter((employee) => (
    matchesMatrixSearch(employee, query)
    && (!selectedDepartmentId || employee.department_name === selectedDepartmentId)
    && (!selectedEmployerId || employee.employer_id === selectedEmployerId)
    && matchesEmployeeStatus(employee, employeeStatus)
  ));
}
