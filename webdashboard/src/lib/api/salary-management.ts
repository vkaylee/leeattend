import { fetch } from "./client";

export interface SalaryManagementRow {
  configured_component_count: number;
  department_id?: string | null;
  department_name?: string | null;
  employee_id: string;
  employee_id_number: string;
  employee_name: string;
  employer_id: string;
  employer_name: string;
  employment_end_date?: string | null;
  employment_id: string;
  employment_start_date: string;
  employment_status: string;
  employment_type: string;
  job_title?: string | null;
  latest_effective_date?: string | null;
  salary_status: "current" | "future" | "unconfigured";
}

export interface SalaryManagementSearch {
  as_of?: string;
  department_id?: string;
  employer_id?: string;
  limit?: number;
  page?: number;
  search?: string;
  status?: string;
  unconfigured?: boolean;
}

export interface SalaryManagementResponse {
  as_of: string;
  data: SalaryManagementRow[];
  limit: number;
  page: number;
  total: number;
}

export async function fetchSalaryManagement(
  params: SalaryManagementSearch,
): Promise<SalaryManagementResponse> {
  const { data } = await fetch<SalaryManagementResponse>(
    "GET",
    "/api/v1/salary-management",
    { params: { query: { ...params } } },
  );
  return data;
}
