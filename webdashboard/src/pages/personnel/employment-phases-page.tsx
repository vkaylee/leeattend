import type { ColumnDef } from "@tanstack/react-table";

import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate, useSearch } from "@tanstack/react-router";
import { Milestone, Plus, Search, UserRound } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import type { Employee, Employment } from "@/lib/api/types";

import PermissionGate from "@/components/common/permission-gate";
import AssignMilestoneModal from "@/components/features/employees/assign-milestone-modal";
import UpdatePhaseStatusModal from "@/components/features/employees/update-phase-status-modal";
import EmploymentPhaseManager from "@/components/people/employment-phase-manager";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { PageContainer } from "@/components/ui/page-container";
import PageHeader from "@/components/ui/page-header";
import ResponsiveTable from "@/components/ui/responsive-table";
import StatusBadge from "@/components/ui/status-badge";
import { fetchEmployeeContracts } from "@/lib/api/contracts";
import { fetchEmploymentPhaseAssignments, fetchEmploymentPhases, type EmploymentPhaseAssignment } from "@/lib/api/employee-phases";
import { fetchEmployees, fetchEmployeeEmployments } from "@/lib/api/employees";

function displayDate(value: string) {
  return new Date(value).toLocaleDateString();
}

export default function EmploymentPhasesPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const search = useSearch({ strict: false });
  const queryClient = useQueryClient();
  const [selected, setSelected] = useState<EmploymentPhaseAssignment | null>(null);
  const [assignOpen, setAssignOpen] = useState(false);
  const [statusOpen, setStatusOpen] = useState(false);
  const [queryText, setQueryText] = useState(search.q ?? "");
  const [selectedEmploymentId, setSelectedEmploymentId] = useState("");

  const { data: employeesResponse } = useQuery({
    queryKey: ["employment-phase-employee-picker"],
    queryFn: () => fetchEmployees(1, 300),
  });
  const employees = useMemo(() => employeesResponse?.data ?? [], [employeesResponse?.data]);
  const selectedEmployeeId = search.employeeId ?? "";
  const { data: employments = [] } = useQuery({
    enabled: Boolean(selectedEmployeeId),
    queryKey: ["employment-phase-employments", selectedEmployeeId],
    queryFn: () => fetchEmployeeEmployments(selectedEmployeeId),
  });
  const { data: contracts = [] } = useQuery({
    enabled: Boolean(selectedEmployeeId),
    queryKey: ["employment-phase-contracts", selectedEmployeeId],
    queryFn: () => fetchEmployeeContracts(selectedEmployeeId),
  });
  const preferred = employments.find((employment) => employment.status === "active") ?? employments[0];
  const activeEmploymentId = selectedEmployeeId
    ? (employments.some((employment) => employment.id === selectedEmploymentId) ? selectedEmploymentId : preferred?.id ?? "")
    : "";
  const selectedContracts = contracts.filter((contract) => !activeEmploymentId || contract.employment_id === activeEmploymentId);

  const page = search.page ?? 1;
  const limit = search.limit ?? 50;
  const status = search.status ?? "";
  const { data, isLoading, isError, refetch } = useQuery({
    queryKey: ["employment-phase-assignments", page, limit, search.q, status, selectedEmployeeId],
    queryFn: () => fetchEmploymentPhaseAssignments({ employeeId: selectedEmployeeId || undefined, limit, page, q: search.q, status }),
  });
  const { data: definitions = [] } = useQuery({ queryKey: ["employment-phases"], queryFn: fetchEmploymentPhases });

  const columns = useMemo<ColumnDef<EmploymentPhaseAssignment>[]>(() => [
    { accessorKey: "employee_name", header: t("common.employee", { defaultValue: "Employee" }) },
    { accessorKey: "employee_code", header: t("common.employee_code", { defaultValue: "Code" }) },
    { accessorKey: "phase_name", header: t("common.phase", { defaultValue: "Phase" }) },
    { accessorKey: "department_name", header: t("common.department", { defaultValue: "Department" }), cell: ({ row }) => row.original.department_name ?? "—" },
    { accessorKey: "start_date", header: t("common.startDate", { defaultValue: "Start" }), cell: ({ row }) => displayDate(row.original.start_date) },
    { accessorKey: "end_date", header: t("common.endDate", { defaultValue: "End" }), cell: ({ row }) => displayDate(row.original.end_date) },
    { accessorKey: "status", header: t("common.status", { defaultValue: "Status" }), cell: ({ row }) => <StatusBadge variant={row.original.status === "active" ? "success" : "neutral"}>{row.original.status}</StatusBadge> },
  ], [t]);

  const updateSearch = (patch: Record<string, unknown>) => {
    void navigate({ search: (previous: Record<string, unknown>) => ({ ...previous, ...patch }) as never, replace: true });
  };

  const employeeOptions = useMemo(() => employees.map((employee: Employee) => ({
    value: employee.id,
    label: employee.full_name,
    description: employee.employee_id_number,
    icon: <UserRound className="h-4 w-4" />,
  })), [employees]);

  const employmentOptions = useMemo(() => employments.map((employment: Employment) => ({
    value: employment.id,
    label: `${employment.status} · ${employment.start_date}`,
  })), [employments]);

  const pageCount = Math.max(1, Math.ceil((data?.total ?? 0) / limit));
  const rows = data?.data ?? [];

  return (
    <PageContainer>
      <PageHeader
        actions={<PermissionGate permission="employment_phases:manage"><Button disabled={!selectedEmployeeId} icon={<Plus size={16} />} onClick={() => { setSelected(null); setAssignOpen(true); }} variant="primary">{t("employees.phases.add_milestone", { defaultValue: "Add phase" })}</Button></PermissionGate>}
        icon={<Milestone size={24} />}
        subtitle={t("common.employment_phases_description", { defaultValue: "Manage employee phases across the organization." })}
        title={t("common.employment_phases", { defaultValue: "Employment Phases" })}
      />

      <div className="flex flex-col gap-3 rounded-2xl border border-slate-200 bg-white p-4 dark:border-slate-800 dark:bg-slate-900 sm:flex-row sm:flex-wrap">
        <div className="min-w-[240px] flex-1">
          <Combobox
            aria-label={t("common.employee", { defaultValue: "Employee" })}
            onChange={(employeeId) => { updateSearch({ employeeId: employeeId || undefined, page: 1 }); }}
            options={employeeOptions}
            placeholder={t("common.select_employee", { defaultValue: "Select employee" })}
            value={selectedEmployeeId}
          />
        </div>
        {selectedEmployeeId ? <div className="min-w-[220px] flex-1">
          <Combobox
            aria-label={t("common.employment", { defaultValue: "Employment" })}
            onChange={setSelectedEmploymentId}
            options={employmentOptions}
            placeholder={t("common.select_employment", { defaultValue: "Select employment" })}
            value={activeEmploymentId}
          />
        </div> : null}
        <label className="flex min-w-0 flex-1 items-center gap-2 rounded-xl border border-slate-200 px-3 dark:border-slate-700">
          <Search aria-hidden size={16} />
          <input aria-label={t("common.search", { defaultValue: "Search" })} className="min-w-0 flex-1 bg-transparent py-2 text-sm outline-none" onChange={(event) => { setQueryText(event.target.value); }} onKeyDown={(event) => { if (event.key === "Enter") updateSearch({ q: queryText || undefined, page: 1 }); }} placeholder={t("common.search", { defaultValue: "Search" })} value={queryText} />
        </label>
        <select aria-label={t("common.status", { defaultValue: "Status" })} className="rounded-xl border border-slate-200 bg-transparent px-3 text-sm dark:border-slate-700" onChange={(event) => { updateSearch({ status: event.target.value || undefined, page: 1 }); }} value={status}>
          <option value="">{t("common.all", { defaultValue: "All" })}</option>
          <option value="active">{t("common.active", { defaultValue: "Active" })}</option>
          <option value="completed">{t("common.completed", { defaultValue: "Completed" })}</option>
          <option value="terminated">{t("common.terminated", { defaultValue: "Terminated" })}</option>
        </select>
      </div>

      {isError ? <div className="rounded-2xl border border-rose-200 bg-rose-50 p-6 text-sm text-rose-700"><p>{t("common.error", { defaultValue: "Unable to load data." })}</p><Button className="mt-3" onClick={() => { void refetch(); }} variant="outline">{t("common.retry", { defaultValue: "Retry" })}</Button></div> : null}
      <ResponsiveTable
        columns={columns}
        data={rows}
        emptyState={{ icon: <Milestone className="h-10 w-10" />, title: t("employees.phases.no_milestones", { defaultValue: "No phases found" }) }}
        getCardKey={(row) => row.id}
        getRowId={(row) => row.id}
        isLoading={isLoading}
        onRowClick={(row) => { setSelected(row); setStatusOpen(true); }}
        renderCard={(row) => <Button aria-label={`${row.employee_name} ${row.phase_name}`} className="w-full rounded-2xl border border-slate-200 bg-white p-4 text-left normal-case tracking-normal dark:border-slate-800 dark:bg-slate-900" onClick={() => { setSelected(row); setStatusOpen(true); }} variant="outline"><div className="flex w-full justify-between gap-3"><strong>{row.employee_name}</strong><StatusBadge variant={row.status === "active" ? "success" : "neutral"}>{row.status}</StatusBadge></div><div className="mt-2 w-full text-sm text-slate-500">{row.phase_name} · {displayDate(row.start_date)} – {displayDate(row.end_date)}</div></Button>}
      />
      <div className="flex items-center justify-between text-sm"><span>{data?.total ?? 0} {t("common.results", { defaultValue: "results" })}</span><div className="flex gap-2"><Button disabled={page <= 1} onClick={() => { updateSearch({ page: page - 1 }); }} variant="outline">{t("common.previous", { defaultValue: "Previous" })}</Button><span className="px-2 py-2">{page} / {pageCount}</span><Button disabled={page >= pageCount} onClick={() => { updateSearch({ page: page + 1 }); }} variant="outline">{t("common.next", { defaultValue: "Next" })}</Button></div></div>

      <EmploymentPhaseManager />

      {assignOpen ? <AssignMilestoneModal contracts={selectedContracts} definitions={definitions} employeeId={selected?.employee_id ?? selectedEmployeeId} employmentId={selected?.employment_id ?? selectedEmploymentId} isOpen={assignOpen} onClose={() => { setAssignOpen(false); void queryClient.invalidateQueries({ queryKey: ["employment-phase-assignments"] }); }} /> : null}
      {statusOpen && selected ? <UpdatePhaseStatusModal employeeId={selected.employee_id} employmentId={selected.employment_id} isOpen={statusOpen} onClose={() => { setStatusOpen(false); setSelected(null); }} selectedPhase={selected} /> : null}
    </PageContainer>
  );
}
