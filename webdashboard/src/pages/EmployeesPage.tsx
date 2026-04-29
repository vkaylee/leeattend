import React, { useState, useEffect, useMemo, useRef } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useInfiniteQuery, useMutation, useQueryClient, keepPreviousData } from "@tanstack/react-query";
import {
  UserCircle,
  Building2,
  UserPlus,
  Upload,
  History as HistoryIcon,
  Calendar as CalendarIcon,
  ArrowUpDown,
  ChevronUp,
  ChevronDown,
  ListFilter,
  Check,
  X,
} from "lucide-react";
import * as Popover from "@radix-ui/react-popover";
import NavigableList from "@/components/common/NavigableList";
import type { NavigableListColumn } from "@/components/common/NavigableList";
import Button from "@/components/ui/Button";
import StatusBadge from "@/components/ui/StatusBadge";
import FormField from "@/components/ui/FormField";
import PageHeader from "@/components/ui/PageHeader";

import {
  searchEmployees,
  createEmployee,
  updateEmployee,
  getCurrentWorkspaceId,
  Employee,
  fetchDepartments,
  fetchEmployeeDepartmentHistory,
  transferEmployee,
  fetchEmploymentHistory,
  rehireEmployee
} from "../lib/api";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useDebounce } from "@/hooks/use-debounce";
import { format } from "date-fns";
import { Combobox } from "@/components/ui/Combobox";
import Modal from "@/components/ui/Modal";
import { useToast } from "@/components/ui/Toast";
import { cn } from "@/lib/utils";
import FloatingRecordBadge from "@/components/common/FloatingRecordBadge";
import EmployeeDetailDrawer from "@/components/features/employees/EmployeeDetailDrawer";

import AttendanceImportDialog from "@/components/features/attendance/AttendanceImportDialog";
import PermissionGate from "@/components/common/PermissionGate";
import { useConfirm } from "@/hooks/use-confirm";


export default function EmployeesPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const { formatDate } = useDateTimeFormatter();
  const [searchParams] = useState(() => new URLSearchParams(window.location.search));
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);
  const [editingEmployee, setEditingEmployee] = useState<Employee | null>(null);
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [filterDeptIds, setFilterDeptIds] = useState<string[]>([]);
  const PAGE_SIZE = 20;
  const [prefillEnrollmentId, setPrefillEnrollmentId] = useState<string>("");
  const { confirm, ConfirmationDialog } = useConfirm();

  // Handle enrollment_id query param from staging logs (read once on mount)
  useEffect(() => {
    const enrollmentId = searchParams.get("enrollment_id");
    if (enrollmentId) {
      setPrefillEnrollmentId(enrollmentId);
      setIsModalOpen(true);
      setEditingEmployee(null);
      // Clear from URL without triggering re-render
      const url = new URL(window.location.href);
      url.searchParams.delete("enrollment_id");
      window.history.replaceState({}, "", url.toString());
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const [isTransferModalOpen, setIsTransferModalOpen] = useState(false);
  const [isHistoryModalOpen, setIsHistoryModalOpen] = useState(false);
  const [isRehireModalOpen, setIsRehireModalOpen] = useState(false);
  const [selectedDeptId, setSelectedDeptId] = useState<string>("");

  const debouncedSearch = useDebounce(searchTerm, 500);

  // Sorting state
  type SortKey = "name" | "id" | "department" | "status";
  type SortDir = "asc" | "desc";
  const [sortKey, setSortKey] = useState<SortKey | null>(null);
  const [sortDir, setSortDir] = useState<SortDir>("asc");

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(key);
      setSortDir("asc");
    }
  };

  const renderSortIcon = (col: SortKey) => {
    if (sortKey !== col) return <ArrowUpDown size={10} className="opacity-0 group-hover:opacity-50 transition-opacity ml-1" />;
    return sortDir === "asc" ? <ChevronUp size={10} className="text-primary-600 dark:text-primary-400 ml-1" /> : <ChevronDown size={10} className="text-primary-600 dark:text-primary-400 ml-1" />;
  };

  const {
    data: employeesPages,
    isLoading,
    isFetching,
    isPlaceholderData,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery({
    queryKey: ["employees", wsId, debouncedSearch, sortKey, sortDir, filterDeptIds],
    queryFn: ({ pageParam }) => {
      const apiSortKey = sortKey ? ({ name: "name", id: "employee_id_number", department: "department", status: "status" } as const)[sortKey] : undefined;
      return searchEmployees(wsId, debouncedSearch, pageParam as number, PAGE_SIZE, apiSortKey, sortDir, filterDeptIds.length > 0 ? filterDeptIds : undefined);
    },
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data?.length || 0), 0);
      if (totalLoaded >= (lastPage.total || 0)) return undefined;
      return allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId,
    placeholderData: keepPreviousData,
  });

  const employees = useMemo(() => {
    if (!employeesPages) return [];
    return employeesPages.pages.flatMap((p) => p.data || []);
  }, [employeesPages]);

  const totalItems = employeesPages?.pages[0]?.total || 0;

  const { data: departments } = useQuery({ queryKey: ["departments", wsId], queryFn: () => fetchDepartments(wsId), enabled: !!wsId });

  const toggleDeptFilter = (deptId: string) => {
    setFilterDeptIds(prev =>
      prev.includes(deptId) ? prev.filter(id => id !== deptId) : [...prev, deptId]
    );
  };

  // Column definitions for NavigableList
  const employeeColumns: NavigableListColumn<Employee>[] = [
    {
      key: "name",
      header: <span className="inline-flex items-center">{t("employees.table.name")}{renderSortIcon("name")}</span>,
      onHeaderClick: () => toggleSort("name"),
      cell: (emp) => (
        <div className="flex items-center gap-5">
          <div className={cn("w-12 h-12 rounded-2xl flex items-center justify-center font-bold text-sm shadow-inner border transition-all uppercase", selectedEmployee?.id === emp.id ? "bg-primary-600 text-white border-primary-500" : "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 group-hover:bg-white dark:hover:bg-slate-900 group-hover:text-primary-600 dark:group-hover:text-primary-400 group-hover:border-primary-100 dark:border-primary-800")}>{emp.full_name[0]}</div>
          <div>
            <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none">{emp.full_name}</div>
            <div className="font-mono font-bold text-[10px] text-slate-400 dark:text-slate-500 uppercase tracking-tighter mt-1.5">{emp.employee_id_number}</div>
          </div>
        </div>
      ),
      cellClassName: "py-6",
    },
    {
      key: "department",
      header: (
        <span className="inline-flex items-center gap-2">
          <span className="inline-flex items-center">
            {t("common.department")}{renderSortIcon("department")}
          </span>
          <Popover.Root>
            <Popover.Trigger asChild>
              <button
                type="button"
                data-testid="dept-filter-btn"
                onClick={(e) => e.stopPropagation()}
                className={cn(
                  "relative inline-flex items-center justify-center w-5 h-5 rounded transition-colors",
                  filterDeptIds.length > 0
                    ? "text-primary-600 dark:text-primary-400"
                    : "text-slate-400 dark:text-slate-500 opacity-60 hover:opacity-100 hover:text-primary-600 dark:hover:text-primary-400"
                )}
                title={t("employees.filter_department", "Filter by department")}
              >
                <ListFilter size={12} />
                {filterDeptIds.length > 0 && (
                  <span className="absolute -top-1 -right-1 min-w-[14px] h-[14px] px-0.5 flex items-center justify-center bg-primary-500 text-white text-[8px] font-bold rounded-full leading-none animate-in zoom-in">
                    {filterDeptIds.length}
                  </span>
                )}
              </button>
            </Popover.Trigger>
            <Popover.Portal>
              <Popover.Content
                className="z-[10000] w-56 overflow-hidden rounded-2xl border border-slate-100 dark:border-slate-700 bg-white dark:bg-slate-900 p-2 shadow-2xl dark:shadow-black/30 animate-in fade-in zoom-in-95 duration-200"
                sideOffset={8}
                align="start"
                onClick={(e) => e.stopPropagation()}
              >
                <div className="px-3 pt-2 pb-1.5">
                  <div className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em]">
                    {t("employees.filter_department", "Filter by department")}
                  </div>
                </div>
                <div className="max-h-60 overflow-y-auto custom-scrollbar">
                  {/* Clear filter button — only shown when filter is active */}
                  {filterDeptIds.length > 0 && (
                    <button
                      type="button"
                      onClick={() => setFilterDeptIds([])}
                      className="flex w-full items-center gap-3 rounded-xl px-3 py-2 text-[11px] font-bold uppercase tracking-wide text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/30 transition-colors mb-1"
                    >
                      <X size={12} className="shrink-0" />
                      {t("employees.clear_filter", "Xóa bộ lọc")}
                    </button>
                  )}

                  {departments && departments.length > 0 ? (
                    departments.map(d => {
                      const isChecked = filterDeptIds.includes(d.id);
                      return (
                        <button
                          key={d.id}
                          type="button"
                          data-testid={`dept-filter-option-${d.id}`}
                          onClick={() => toggleDeptFilter(d.id)}
                          className={cn(
                            "flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-[11px] font-bold uppercase tracking-wide transition-colors",
                            isChecked
                              ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400"
                              : "text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800"
                          )}
                        >
                          <div className={cn("w-4 h-4 rounded border flex items-center justify-center shrink-0 transition-colors",
                            isChecked
                              ? "bg-primary-600 border-primary-600 text-white"
                              : "border-slate-300 dark:border-slate-600"
                          )}>
                            {isChecked && <Check size={10} strokeWidth={3} />}
                          </div>
                          <Building2 size={12} className="opacity-50 shrink-0" />
                          <span className="truncate">{d.name}</span>
                        </button>
                      );
                    })
                  ) : (
                    <div className="px-3 py-6 text-center text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest opacity-60">
                      {t("employees.no_departments", "Chưa có phòng ban")}
                    </div>
                  )}
                </div>
              </Popover.Content>
            </Popover.Portal>
          </Popover.Root>
        </span>
      ),
      onHeaderClick: () => toggleSort("department"),
      cell: (emp) => emp.department_name ? (
        <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 rounded-xl text-[10px] font-bold uppercase tracking-widest border border-blue-100 dark:border-blue-800">
          <Building2 size={12} className="opacity-60" />{emp.department_name}
        </span>
      ) : <span className="text-slate-300 dark:text-slate-600">—</span>,
      cellClassName: "py-6",
    },
    {
      key: "status",
      header: <span className="inline-flex items-center">{t("employees.table.status")}{renderSortIcon("status")}</span>,
      onHeaderClick: () => toggleSort("status"),
      headerClassName: "text-center",
      cell: (emp) => (
        <StatusBadge variant={emp.is_active ? "success" : "neutral"}>{emp.is_active ? t("employees.table.active").toUpperCase() : t("employees.table.terminated").toUpperCase()}</StatusBadge>
      ),
      cellClassName: "py-6 text-center",
    },
  ];

  const { data: deptHistory } = useQuery({ queryKey: ["deptHistory", selectedEmployee?.id], queryFn: () => fetchEmployeeDepartmentHistory(wsId, selectedEmployee!.id), enabled: !!selectedEmployee });
  const { data: employmentHistory } = useQuery({ queryKey: ["employmentHistory", selectedEmployee?.id], queryFn: () => fetchEmploymentHistory(wsId, selectedEmployee!.id), enabled: !!selectedEmployee });

  const createMutation = useMutation({
    mutationFn: (data: Partial<Employee>) => createEmployee(wsId, data),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["employees", wsId] }); setIsModalOpen(false); toast.success(t("common.success")); },
  });

  const updateMutation = useMutation({
    mutationFn: (data: { id: string; employee: Partial<Employee> }) => updateEmployee(wsId, data.id, data.employee),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["employees", wsId] }); setIsModalOpen(false); setEditingEmployee(null); toast.success(t("common.success")); },
  });

  const transferMutation = useMutation({
    mutationFn: (data: any) => transferEmployee(wsId, selectedEmployee!.id, data),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["deptHistory", selectedEmployee?.id] }); setIsTransferModalOpen(false); setSelectedDeptId(""); toast.success(t("common.success")); },
  });

  const rehireMutation = useMutation({
    mutationFn: (data: { hire_date: string }) => rehireEmployee(wsId, selectedEmployee!.id, data),
    onSuccess: (updatedEmp) => { queryClient.invalidateQueries({ queryKey: ["employees", wsId] }); queryClient.invalidateQueries({ queryKey: ["employmentHistory", selectedEmployee?.id] }); setSelectedEmployee(updatedEmp); setIsRehireModalOpen(false); toast.success(t("common.success")); },
  });

  const toast = useToast();



  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault(); const formData = new FormData(e.currentTarget);
    const data = { full_name: formData.get("full_name") as string, employee_id_number: formData.get("employee_id_number") as string, hire_date: formData.get("hire_date") as string, is_active: formData.get("is_active") === "on" };
    if (editingEmployee) updateMutation.mutate({ id: editingEmployee.id, employee: data });
    else {
      createMutation.mutate(data);
      setPrefillEnrollmentId(""); // Clear prefill after creating
    }
  };

  const handleTransferSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault(); const formData = new FormData(e.currentTarget);
    transferMutation.mutate({ department_id: selectedDeptId, effective_date: formData.get("effective_date"), notes: formData.get("notes") });
  };

  const handleRehireSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault(); const formData = new FormData(e.currentTarget);
    rehireMutation.mutate({ hire_date: formData.get("hire_date") as string });
  };

  const searchInputRef = useRef<HTMLInputElement>(null);
  const focusSearch = () => setTimeout(() => searchInputRef.current?.focus(), 100);

  const handleDrawerClose = () => {
    setSelectedEmployee(null);
    focusSearch();
  };

  if (isLoading) return <div className="flex h-full items-center justify-center py-32"><div className="animate-spin rounded-full h-12 w-12 border-4 border-primary-600 border-t-transparent shadow-xl"></div></div>;

  return (
    <div className="space-y-10 pb-20 animate-in fade-in duration-700">
      <PageHeader
        title={t("employees.title")}
        subtitle={t("employees.subtitle")}
        data-testid="employees-title"
        actions={
          <>
            <PermissionGate permission="employees:create">
              <Button variant="outline" data-testid="import-employee-btn" onClick={() => setIsImportDialogOpen(true)} icon={<Upload size={18} />}>{t("employees.bulk_import_btn").toUpperCase()}</Button>
            </PermissionGate>
            <PermissionGate permission="employees:create">
              <Button data-testid="add-employee-btn" onClick={() => { setEditingEmployee(null); setIsModalOpen(true); }} icon={<UserPlus size={18} />}>{t("employees.add_btn").toUpperCase()}</Button>
            </PermissionGate>
          </>
        }
      />

      {/* Active filter chips */}
      {filterDeptIds.length > 0 && departments && (
        <div className="flex items-center gap-2 flex-wrap animate-in slide-in-from-top-4 duration-300" data-testid="active-filter-chips">
          <span className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] mr-1">
            {t("employees.filtering_by", "Đang lọc")}
          </span>
          {filterDeptIds.map(id => {
            const dept = departments.find(d => d.id === id);
            if (!dept) return null;
            return (
              <button
                key={id}
                type="button"
                data-testid={`dept-chip-${id}`}
                onClick={() => toggleDeptFilter(id)}
                className="inline-flex items-center gap-1.5 px-3 py-1 bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 rounded-full text-[10px] font-bold uppercase tracking-wide border border-primary-100 dark:border-primary-800 hover:bg-primary-100 dark:hover:bg-primary-900 transition-colors group"
              >
                <Building2 size={10} className="opacity-60" />
                {dept.name}
                <X size={10} className="opacity-40 group-hover:opacity-100 transition-opacity" />
              </button>
            );
          })}
          <button
            type="button"
            onClick={() => setFilterDeptIds([])}
            className="text-[10px] font-bold text-slate-400 dark:text-slate-500 hover:text-red-500 dark:hover:text-red-400 uppercase tracking-wide transition-colors ml-1"
          >
            {t("employees.clear_all", "Xóa tất cả")}
          </button>
        </div>
      )}

      <div className={cn("transition-opacity duration-300", isPlaceholderData && isFetching && "opacity-50")}>
      <NavigableList<Employee>
        columns={employeeColumns}
        items={employees}
        itemKey={(emp) => emp.id}
        onSelect={(emp) => setSelectedEmployee(emp)}
        selectedKey={selectedEmployee?.id}
        isLoading={false}
        emptyMessage={t("employees.table.no_data")}
        showChevron
        rowIdPrefix="employee-row"
        testIdPrefix="employees-row"
        resetDeps={[searchTerm]}
        searchValue={searchTerm}
        onSearchChange={setSearchTerm}
        searchPlaceholder={t("employees.search_placeholder")}
        searchInputRef={searchInputRef}
        searchTestId="employee-search"
        hasNextPage={hasNextPage}
        isFetchingNextPage={isFetchingNextPage}
        fetchNextPage={fetchNextPage}
        maxHeightClass=""
        className="animate-in slide-in-from-bottom-8 duration-700"
      />
      </div>

      {/* Employee Detail Drawer */}
      <EmployeeDetailDrawer
        key={selectedEmployee?.id || "closed"}
        isOpen={!!selectedEmployee}
        onClose={handleDrawerClose}
        employee={selectedEmployee}
        wsId={wsId}
        departments={departments}
        deptHistory={deptHistory}
        employmentHistory={employmentHistory}
        formatDate={formatDate}
        onEdit={(emp) => { setEditingEmployee(emp); setIsModalOpen(true); }}
        onTerminate={async (emp) => {
          const ok = await confirm({
            title: t("common.warning", { defaultValue: "Warning" }),
            message: t("employees.details.terminate_confirm"),
            variant: "danger"
          });
          if (!ok) return;
          updateMutation.mutate({ id: emp.id, employee: { is_active: false, termination_date: format(new Date(), 'yyyy-MM-dd') } });
        }}
        onRehire={() => setIsRehireModalOpen(true)}
        onTransfer={(deptId) => { setSelectedDeptId(deptId); setIsTransferModalOpen(true); }}
      />

      {/* Main Employee Form Modal */}
      <Modal isOpen={isModalOpen} onClose={() => setIsModalOpen(false)} title={editingEmployee ? t("employees.form.edit_title") : t("employees.form.add_title")} subtitle={t("employees.form.subtitle")} icon={<UserCircle size={24} />} maxWidth="lg">
        <form onSubmit={handleSubmit} className="space-y-8 pt-4 animate-in fade-in duration-500">
          <FormField label={t("employees.form.full_name")} name="full_name" data-testid="employees-form-name-input" required defaultValue={editingEmployee?.full_name} />
          <FormField label={t("employees.form.employee_id")} name="employee_id_number" data-testid="employees-form-id-input" required defaultValue={editingEmployee?.employee_id_number || prefillEnrollmentId} inputClassName="font-mono tracking-widest" />
          <FormField label={t("employees.form.hire_date")} name="hire_date" type="date" data-testid="employees-form-hire-date-input" required defaultValue={editingEmployee?.hire_date} />
          <div className="flex items-center gap-4 py-2 px-2 group cursor-pointer">
            <input type="checkbox" name="is_active" id="is_active" data-testid="employees-form-active-checkbox" defaultChecked={editingEmployee ? editingEmployee.is_active : true} className="w-7 h-7 text-primary-600 dark:text-primary-400 rounded-xl border-slate-200 dark:border-slate-700 focus:ring-primary-500 transition-all cursor-pointer shadow-inner" />
            <label htmlFor="is_active" className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest cursor-pointer group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors">{t("employees.form.is_active")}</label>
          </div>
          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
            <Button variant="outline" size="lg" onClick={() => setIsModalOpen(false)}>{t("employees.form.cancel").toUpperCase()}</Button>
            <Button variant="secondary" size="lg" type="submit" data-testid="save-employee-btn">{editingEmployee ? t("employees.form.update").toUpperCase() : t("employees.form.save").toUpperCase()}</Button>
          </div>
        </form>
      </Modal>

      {/* Transfer Modal */}
      <Modal isOpen={isTransferModalOpen} onClose={() => { setIsTransferModalOpen(false); setSelectedDeptId(""); }} title={t("employees.transfer.title")} subtitle={t("employees.transfer.subtitle")} icon={<Building2 size={24} />} maxWidth="md">
        <form onSubmit={handleTransferSubmit} className="space-y-8 pt-4 animate-in fade-in duration-500">
          <div className="space-y-3 px-1"><label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("employees.transfer.new_dept")}</label><Combobox options={departments?.map(d => ({ value: d.id, label: d.name.toUpperCase(), icon: <Building2 size={14} /> })) || []} value={selectedDeptId} onChange={setSelectedDeptId} placeholder={t("employees.transfer.select_dept")} className="bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 h-14" /></div>
          <FormField label={t("employees.transfer.effective_date")} name="effective_date" type="date" required defaultValue={format(new Date(), 'yyyy-MM-dd')} />
          <FormField label={t("employees.transfer.notes")} name="notes" type="textarea" placeholder={t("employees.transfer.notes_placeholder")} rows={4} />
          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
            <Button variant="outline" size="lg" onClick={() => setIsTransferModalOpen(false)}>{t("common.cancel").toUpperCase()}</Button>
            <Button variant="secondary" size="lg" type="submit" data-testid="submit-transfer-btn">{t("employees.transfer.submit").toUpperCase()}</Button>
          </div>
        </form>
      </Modal>

      {/* History Modal */}
      <Modal isOpen={isHistoryModalOpen} onClose={() => setIsHistoryModalOpen(false)} title={t("employees.history.title")} subtitle={t("employees.history.subtitle")} icon={<HistoryIcon size={24} />} maxWidth="lg">
        <div className="space-y-4 pt-4 animate-in fade-in duration-500">
          {employmentHistory?.map((period) => (
            <div key={period.id} className="p-8 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] shadow-inner group hover:bg-white dark:hover:bg-slate-900 transition-all relative overflow-hidden">
              <div className="flex justify-between items-center relative z-10">
                <div>
                  <div className="font-bold text-slate-700 dark:text-slate-200 text-lg uppercase tracking-tight flex items-center gap-4">
                    {period.is_rehire ? t("employees.history.rehire_period") : t("employees.history.first_period")}
                    {!period.termination_date && (<StatusBadge variant="success">{t("employees.history.currently_working")}</StatusBadge>)}
                  </div>
                  <div className="text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.25em] mt-3 flex items-center gap-3 opacity-70">
                    <CalendarIcon size={14} className="text-primary-400" /> {t("employees.history.period_range", { start: formatDate(period.hire_date), end: period.termination_date ? formatDate(period.termination_date) : t("employees.history.present").toUpperCase() })}
                  </div>
                </div>
              </div>
              <CalendarIcon className="absolute -right-8 -bottom-8 w-32 h-32 text-slate-700 dark:text-slate-200 opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
            </div>
          ))}
        </div>
        <div className="mt-10 pt-8 border-t border-slate-50 dark:border-slate-800 flex justify-end"><Button variant="secondary" size="lg" onClick={() => setIsHistoryModalOpen(false)}>{t("common.close").toUpperCase()}</Button></div>
      </Modal>

      {/* Rehire Modal */}
      <Modal isOpen={isRehireModalOpen} onClose={() => setIsRehireModalOpen(false)} title={t("employees.rehire.title")} subtitle={t("employees.rehire.subtitle")} icon={<UserPlus size={24} />} maxWidth="md">
        <div className="space-y-10 pt-4 animate-in fade-in duration-500">
          <div className="p-8 bg-slate-50 dark:bg-slate-800 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-inner text-center italic"><p className="text-sm font-bold text-slate-600 dark:text-slate-300 uppercase tracking-tight leading-relaxed">"{t("employees.rehire.confirm_msg")}"</p></div>
          <form onSubmit={handleRehireSubmit} className="space-y-8">
            <FormField label={t("employees.rehire.rehire_date")} name="hire_date" type="date" required defaultValue={format(new Date(), 'yyyy-MM-dd')} />
            <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
              <Button variant="outline" size="lg" onClick={() => setIsRehireModalOpen(false)}>{t("common.cancel").toUpperCase()}</Button>
              <Button size="lg" type="submit" data-testid="confirm-rehire-btn" loading={rehireMutation.isPending}>{rehireMutation.isPending ? t("employees.rehire.processing").toUpperCase() : t("employees.rehire.confirm").toUpperCase()}</Button>
            </div>
          </form>
        </div>
      </Modal>

      <FloatingRecordBadge loaded={employees.length} total={totalItems} label={t("common.pagination.records")} />

      <AttendanceImportDialog
        isOpen={isImportDialogOpen}
        onClose={() => setIsImportDialogOpen(false)}
        title={t("employees.bulk_import_title")}
        endpoint="/employees/upload"
        targetType="employee"
        customRequirements={[
          t("employees.import_req.col_a"),
          t("employees.import_req.col_b"),
          t("employees.import_req.col_c"),
          t("employees.import_req.col_d"),
        ]}
      />
      <ConfirmationDialog />
    </div>
  );
}
