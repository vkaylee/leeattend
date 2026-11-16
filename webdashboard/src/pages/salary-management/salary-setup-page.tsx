import { useQuery } from "@tanstack/react-query";
import { Link, useNavigate, useParams, useSearch } from "@tanstack/react-router";
import { ArrowLeft, DollarSign } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { Employee, Employment } from "@/lib/api/types";

import { EmployeeSearch } from "@/components/common/employee-search";
import EmployeeSalaryConfig from "@/components/payroll/employee-salary-config";
import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import { PageContainer } from "@/components/ui/page-container";
import { PageHeader } from "@/components/ui/page-header";
import { Typography } from "@/components/ui/typography";
import { fetchEmployee, fetchEmployeeEmployments } from "@/lib/api/employees";
import { createQueryKey } from "@/lib/query-keys";


function SetupSkeleton() {
  return (
    <div aria-label="Loading salary setup" className="space-y-6 animate-pulse" role="status">
      <div className="h-8 w-64 rounded-xl bg-slate-200 dark:bg-slate-800" />
      <div className="h-32 rounded-2xl bg-slate-200 dark:bg-slate-800" />
      <div className="h-96 rounded-2xl bg-slate-200 dark:bg-slate-800" />
    </div>
  );
}

function EmploymentSummary({ employee, employment }: { employee: Employee; employment: Employment }) {
  return (
    <section aria-labelledby="salary-setup-context" className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-800 dark:bg-slate-900 sm:p-8">
      <div className="flex flex-col gap-5 sm:flex-row sm:items-center">
        <div aria-hidden className="flex size-16 shrink-0 items-center justify-center rounded-2xl bg-primary-600 text-2xl font-black uppercase text-white shadow-lg shadow-primary-500/20">
          {employee.full_name.slice(0, 1)}
        </div>
        <div className="min-w-0">
          <Typography className="text-2xl font-black tracking-tight text-slate-900 dark:text-slate-100" id="salary-setup-context" variant="h2">
            {employee.full_name}
          </Typography>
          <div className="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-sm text-slate-500 dark:text-slate-400">
            <span className="font-mono font-semibold text-primary-600 dark:text-primary-400">{employee.employee_id_number}</span>
            <span>{employment.job_title ?? employee.job_title ?? "—"}</span>
            <span>{employment.start_date} — {employment.end_date ?? "—"}</span>
          </div>
        </div>
      </div>
    </section>
  );
}

export default function SalarySetupPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const params = useParams({ strict: false });
  const search = useSearch({ strict: false });
  const employeeId = search.employee_id;
  const routeEmploymentId = params.employmentId;
  const backSearch = {
    as_of: search.as_of,
    department_id: search.department_id,
    employer_id: search.employer_id,
    page: search.page,
    search: search.search,
    status: search.status,
  };

  const employeeQuery = useQuery({
    enabled: Boolean(employeeId),
    queryKey: createQueryKey("salary-setup-employee", employeeId),
    queryFn: () => fetchEmployee(employeeId ?? ""),
  });
  const employmentsQuery = useQuery({
    enabled: Boolean(employeeId),
    queryKey: createQueryKey("salary-setup-employments", employeeId),
    queryFn: () => fetchEmployeeEmployments(employeeId ?? ""),
  });

  if (!employeeId) {
    return (
      <PageContainer>
        <PageHeader icon={<DollarSign size={24} />} subtitle={t("payroll.setup.select_subtitle", "Chọn nhân sự để bắt đầu gán lương.")} title={t("payroll.setup.title", "Thiết lập lương")} />
        <section aria-labelledby="salary-setup-select" className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-800 dark:bg-slate-900 sm:p-8">
          <Typography className="mb-4 text-lg font-bold text-slate-900 dark:text-slate-100" id="salary-setup-select" variant="h2">{t("payroll.setup.select_employee", "Chọn nhân sự")}</Typography>
          <EmployeeSearch
            data-testid="salary-setup-employee-search"
            onChange={(value) => {
              const id = Array.isArray(value) ? value[0] : value;
              if (id) void navigate({ to: "/salary-management/setup", search: { ...backSearch, employee_id: id } as never });
            }}
            placeholder={t("payroll.setup.search_employee", "Tìm theo tên hoặc mã nhân viên")}
          />
        </section>
      </PageContainer>
    );
  }

  if (employeeQuery.isLoading || employmentsQuery.isLoading) return <PageContainer><SetupSkeleton /></PageContainer>;
  if (employeeQuery.isError || employmentsQuery.isError || !employeeQuery.data) {
    return (
      <PageContainer>
        <EmptyState
          action={<Link to="/salary-management"><Button icon={<ArrowLeft size={16} />} variant="outline">{t("common.back_to_list", "Quay lại danh sách")}</Button></Link>}
          description={t("payroll.setup.employee_error", "Không thể tải thông tin nhân sự.")}
          icon={<DollarSign size={40} />}
          title={t("payroll.setup.not_found", "Không tìm thấy nhân sự")}
        />
      </PageContainer>
    );
  }

  const employee = employeeQuery.data;
  const employment = routeEmploymentId
    ? (employmentsQuery.data ?? []).find((item) => item.id === routeEmploymentId)
    : search.employment_id
      ? (employmentsQuery.data ?? []).find((item) => item.id === search.employment_id)
      : (employmentsQuery.data ?? []).find((item) => item.status === "active") ?? employmentsQuery.data?.[0];
  if (!employment) {
    return (
      <PageContainer>
        <EmptyState
          action={<Link to="/salary-management"><Button icon={<ArrowLeft size={16} />} variant="outline">{t("common.back_to_list", "Quay lại danh sách")}</Button></Link>}
          description={t("payroll.setup.no_employment_description", "Nhân sự này chưa có hợp đồng lao động để gán khoản lương.")}
          icon={<DollarSign size={40} />}
          title={t("payroll.setup.no_employment", "Chưa có quan hệ lao động")}
        />
      </PageContainer>
    );
  }

  const goBack = () => { void navigate({ to: "/salary-management", search: backSearch as never }); };
  return (
    <PageContainer>
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <PageHeader icon={<DollarSign size={24} />} subtitle={t("payroll.setup.subtitle", "Gán lương cơ bản và phụ cấp cho đúng quan hệ lao động.")} title={t("payroll.setup.title", "Thiết lập lương")} />
        <Button className="shrink-0" icon={<ArrowLeft size={16} />} onClick={goBack} variant="outline">{t("common.back_to_list", "Quay lại danh sách")}</Button>
      </div>
      <EmploymentSummary employee={employee} employment={employment} />
      <EmployeeSalaryConfig employeeId={employee.id} employmentId={employment.id} />
    </PageContainer>
  );
}
