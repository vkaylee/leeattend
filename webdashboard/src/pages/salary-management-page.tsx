import { useQuery } from "@tanstack/react-query";
import { CalendarDays, ChevronLeft, ChevronRight, DollarSign } from "lucide-react";
import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate, useSearch } from "@tanstack/react-router";

import Button from "@/components/ui/button";
import { CONTROL_FOCUS_WITHIN } from "@/components/ui/control-styles";
import EmptyState from "@/components/ui/empty-state";
import { PageContainer } from "@/components/ui/page-container";
import PageHeader from "@/components/ui/page-header";
import { SearchInput } from "@/components/ui/search-input";
import { fetchSalaryManagement, type SalaryManagementRow } from "@/lib/api/salary-management";

const PAGE_SIZE = 25;

interface SalaryManagementSearch {
  as_of?: string;
  department_id?: string;
  employer_id?: string;
  page?: number;
  search?: string;
  status?: string;
  unconfigured?: boolean;
}

function SalarySkeleton() {
  return (
    <div aria-label="Loading salary records" className="overflow-hidden rounded-2xl border border-slate-200 bg-white dark:border-slate-800 dark:bg-slate-900" role="status">
      <div className="space-y-4 p-6">
        {Array.from({ length: 5 }, (_, index) => <div className="h-12 animate-pulse rounded-xl bg-slate-100 dark:bg-slate-800" key={index} />)}
      </div>
    </div>
  );
}

function SalaryRow({ row, setupSearch, t }: { row: SalaryManagementRow; setupSearch: Record<string, string | number | undefined>; t: (key: string, options?: { defaultValue?: string }) => string }) {
  const statusLabel = t("common.active", { defaultValue: "Active" });

  return (
    <tr className="border-b border-slate-100 last:border-0 dark:border-slate-800">
      <td className="px-5 py-4">
        <Link className="font-semibold text-primary-700 hover:underline dark:text-primary-300" params={{ employmentId: row.employment_id }} search={{ ...setupSearch, employee_id: row.employee_id } as never} to="/salary-management/$employmentId/edit">{row.employee_name}</Link>
        <div className="text-xs text-slate-500">{row.employee_id_number}</div>
      </td>
      <td className="px-5 py-4 text-sm text-slate-600 dark:text-slate-300">{row.employer_name}</td>
      <td className="px-5 py-4 text-sm text-slate-600 dark:text-slate-300">{row.department_name ?? "—"}</td>
      <td className="px-5 py-4">
        <div className="font-medium text-slate-800 dark:text-slate-100">{row.component_name}</div>
        <div className="text-xs text-slate-500">{row.component_code}</div>
      </td>
      <td className="px-5 py-4 text-right text-sm font-semibold text-slate-700 dark:text-slate-200">{row.amount}</td>
      <td className="px-5 py-4 text-right text-sm text-slate-600 dark:text-slate-300">{row.effective_date}</td>
      <td className="px-5 py-4 text-right">
        <span className="rounded-full bg-emerald-50 px-2 py-1 text-xs font-semibold text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300">{statusLabel}</span>
      </td>
    </tr>
  );
}

export default function SalaryManagementPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const search = useSearch({ strict: false }) as SalaryManagementSearch;
  const page = Math.max(1, search.page ?? 1);
  const query = useQuery({
    queryKey: ["salary-management", search],
    queryFn: () => fetchSalaryManagement({ ...search, limit: PAGE_SIZE, page }),
  });
  const rows = query.data?.data ?? [];
  const total = query.data?.total ?? 0;
  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));
  const updateSearch = (patch: Partial<SalaryManagementSearch>) => {
    void navigate({ search: (previous: Record<string, unknown>) => ({ ...previous, ...patch }) as never, replace: true });
  };
  const setupSearch = {
    as_of: search.as_of,
    department_id: search.department_id,
    employer_id: search.employer_id,
    page,
    search: search.search,
    status: search.status,
  };
  const searchLabel = t("salary_management.search", { defaultValue: "Search employee code or name" });
  const dateValue = search.as_of ?? new Date().toISOString().slice(0, 10);
  const resultLabel = useMemo(() => `${Math.min((page - 1) * PAGE_SIZE + 1, total)}–${Math.min(page * PAGE_SIZE, total)} / ${total}`, [page, total]);

  return (
    <PageContainer>
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <PageHeader icon={<DollarSign size={24} />} subtitle={t("salary_management.subtitle", { defaultValue: "Review salary and allowance configuration by effective date." })} title={t("salary_management.title", { defaultValue: "Salary & allowances" })} />
        <Link search={{ ...setupSearch, employee_id: undefined, employment_id: undefined } as never} to="/salary-management/setup">
          <Button className="shrink-0" icon={<DollarSign size={16} />} variant="primary">{t("salary_management.setup", { defaultValue: "Set up salary" })}</Button>
        </Link>
      </div>
      <section aria-label={t("salary_management.filters", { defaultValue: "Salary filters" })} className="flex flex-col gap-3 rounded-2xl border border-slate-200 bg-white p-4 dark:border-slate-800 dark:bg-slate-900 sm:flex-row sm:flex-wrap sm:items-center">
        <SearchInput
          aria-label={searchLabel}
          containerClassName="min-w-[240px] flex-1"
          onChange={(event) => updateSearch({ search: event.target.value || undefined, page: 1 })}
          onClear={() => updateSearch({ search: undefined, page: 1 })}
          placeholder={searchLabel}
          size="sm"
          value={search.search ?? ""}
        />
        <label className="flex items-center gap-2 rounded-xl border border-slate-200 px-3 text-sm transition-[border-color,box-shadow] duration-200 dark:border-slate-700 focus-within:border-primary-500 dark:focus-within:border-primary-400">
          <CalendarDays aria-hidden size={16} />
          <span className="sr-only">{t("salary_management.as_of", { defaultValue: "Effective on" })}</span>
          <input aria-label={t("salary_management.as_of", { defaultValue: "Effective on" })} className="appearance-none bg-transparent py-2 outline-none focus:outline-none focus:ring-0 focus-visible:!outline-none" data-no-focus-outline onChange={(event) => updateSearch({ as_of: event.target.value, page: 1 })} type="date" value={dateValue} />
        </label>
        <select aria-label={t("common.status", { defaultValue: "Status" })} className="rounded-xl border border-slate-200 bg-transparent px-3 py-2 text-sm outline-none dark:border-slate-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)]" onChange={(event) => updateSearch({ status: event.target.value || undefined, page: 1 })} value={search.status ?? "active"}>
          <option value="active">{t("common.active", { defaultValue: "Active" })}</option>
          <option value="inactive">{t("common.inactive", { defaultValue: "Inactive" })}</option>
          <option value="">{t("common.all", { defaultValue: "All statuses" })}</option>
        </select>
        <span className="px-2 text-xs text-slate-500 dark:text-slate-400">{t("salary_management.scope_note", { defaultValue: "Showing configured salary and allowance components" })}</span>
      </section>
      {query.isError ? <div aria-live="polite" className="rounded-2xl border border-rose-200 bg-rose-50 p-6 text-sm text-rose-700 dark:border-rose-900 dark:bg-rose-950/40 dark:text-rose-200"><p>{t("salary_management.error", { defaultValue: "Salary records could not be loaded." })}</p><Button className="mt-3" onClick={() => { void query.refetch(); }} variant="outline">{t("common.retry", { defaultValue: "Retry" })}</Button></div> : null}
      {query.isLoading ? <SalarySkeleton /> : rows.length === 0 ? <EmptyState description={t("salary_management.empty_description", { defaultValue: "No salary records match current filters." })} icon={<DollarSign size={40} />} title={t("salary_management.empty", { defaultValue: "No salary records" })} /> : <div className="overflow-x-auto rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900"><table className="w-full min-w-[1120px] text-left"><caption className="sr-only">{t("salary_management.title", { defaultValue: "Salary & allowances" })}</caption><thead className="bg-slate-50 text-xs uppercase tracking-wide text-slate-500 dark:bg-slate-800/60"><tr><th className="px-5 py-4">{t("salary_management.employee", { defaultValue: "Employee" })}</th><th className="px-5 py-4">{t("salary_management.employer", { defaultValue: "Legal entity" })}</th><th className="px-5 py-4">{t("common.department", { defaultValue: "Department" })}</th><th className="px-5 py-4">{t("salary_management.component", { defaultValue: "Component" })}</th><th className="px-5 py-4 text-right">{t("salary_management.amount", { defaultValue: "Amount" })}</th><th className="px-5 py-4 text-right">{t("common.effective_date", { defaultValue: "Effective date" })}</th><th className="px-5 py-4 text-right">{t("common.status", { defaultValue: "Status" })}</th></tr></thead><tbody>{rows.map((row) => <SalaryRow key={`${row.employment_id}:${row.component_id}`} row={row} setupSearch={setupSearch} t={t} />)}</tbody></table></div>}
      {total > 0 ? <nav aria-label={t("salary_management.pagination", { defaultValue: "Salary records pagination" })} className="flex flex-col items-center justify-between gap-3 text-sm text-slate-500 sm:flex-row"><span>{resultLabel}</span><div className="flex gap-2"><Button aria-label={t("common.previous", { defaultValue: "Previous page" })} disabled={page <= 1} onClick={() => updateSearch({ page: page - 1 })} variant="outline"><ChevronLeft size={16} /></Button><span className="px-2 py-2">{page} / {totalPages}</span><Button aria-label={t("common.next", { defaultValue: "Next page" })} disabled={page >= totalPages} onClick={() => updateSearch({ page: page + 1 })} variant="outline"><ChevronRight size={16} /></Button></div></nav> : null}
    </PageContainer>
  );
}
