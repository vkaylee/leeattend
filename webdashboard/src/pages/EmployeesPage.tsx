import React, { useState, useEffect, useMemo, useRef } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useInfiniteQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useSearchParams } from "react-router-dom";
import {
  UserCircle,
  Building2,
  UserPlus,
  Upload,
  History as HistoryIcon,
  Calendar as CalendarIcon,
} from "lucide-react";
import NavigableList from "@/components/common/NavigableList";
import type { NavigableListColumn } from "@/components/common/NavigableList";

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


export default function EmployeesPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const { formatDate } = useDateTimeFormatter();
  const [searchParams, setSearchParams] = useSearchParams();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);
  const [editingEmployee, setEditingEmployee] = useState<Employee | null>(null);
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const PAGE_SIZE = 20;
  const [prefillEnrollmentId, setPrefillEnrollmentId] = useState<string>("");

  // Handle enrollment_id query param from staging logs
  useEffect(() => {
    const enrollmentId = searchParams.get("enrollment_id");
    if (enrollmentId) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setPrefillEnrollmentId(enrollmentId);
      setIsModalOpen(true);
      setEditingEmployee(null);
      // Clear the query param after opening modal
      searchParams.delete("enrollment_id");
      setSearchParams(searchParams, { replace: true });
    }
  }, [searchParams, setSearchParams]);

  const [isTransferModalOpen, setIsTransferModalOpen] = useState(false);
  const [isHistoryModalOpen, setIsHistoryModalOpen] = useState(false);
  const [isRehireModalOpen, setIsRehireModalOpen] = useState(false);
  const [selectedDeptId, setSelectedDeptId] = useState<string>("");

  const debouncedSearch = useDebounce(searchTerm, 500);

  const {
    data: employeesPages,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery({
    queryKey: ["employees", wsId, debouncedSearch],
    queryFn: ({ pageParam }) => searchEmployees(wsId, debouncedSearch, pageParam as number, PAGE_SIZE),
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data?.length || 0), 0);
      if (totalLoaded >= (lastPage.total || 0)) return undefined;
      return allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId
  });

  const employees = useMemo(() => {
    if (!employeesPages) return [];
    return employeesPages.pages.flatMap((p) => p.data || []);
  }, [employeesPages]);

  const totalItems = employeesPages?.pages[0]?.total || 0;

  // Column definitions for NavigableList
  const employeeColumns: NavigableListColumn<Employee>[] = [
    {
      key: "name",
      header: t("employees.table.name"),
      cell: (emp) => (
        <div className="flex items-center gap-5">
          <div className={cn("w-12 h-12 rounded-2xl flex items-center justify-center font-bold text-sm shadow-inner border transition-all uppercase", selectedEmployee?.id === emp.id ? "bg-primary-600 text-white border-primary-500" : "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 group-hover:bg-white dark:hover:bg-slate-900 group-hover:text-primary-600 dark:group-hover:text-primary-400 group-hover:border-primary-100 dark:border-primary-800")}>{emp.full_name[0]}</div>
          <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none">{emp.full_name}</div>
        </div>
      ),
      cellClassName: "py-6",
    },
    {
      key: "id",
      header: t("employees.table.id"),
      cell: (emp) => <span className="font-mono font-bold text-[11px] text-slate-500 dark:text-slate-400 uppercase tracking-tighter">{emp.employee_id_number}</span>,
      cellClassName: "py-6",
    },
    {
      key: "status",
      header: t("employees.table.status"),
      headerClassName: "text-center",
      cell: (emp) => (
        <span className={cn("px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm", emp.is_active ? 'bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800' : 'bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 opacity-60')}>{emp.is_active ? t("employees.table.active").toUpperCase() : t("employees.table.terminated").toUpperCase()}</span>
      ),
      cellClassName: "py-6 text-center",
    },
  ];

  const { data: departments } = useQuery({ queryKey: ["departments", wsId], queryFn: () => fetchDepartments(wsId), enabled: !!wsId });
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
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="space-y-1">
          <h1 data-testid="employees-title" className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none">{t("employees.title")}</h1>
          <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1.5 opacity-70">{t("employees.subtitle")}</p>
        </div>
        <div className="flex items-center gap-4">
          <PermissionGate permission="employees:create">
            <button data-testid="import-employee-btn" onClick={() => setIsImportDialogOpen(true)} className="flex items-center gap-3 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 px-8 py-3 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800 transition-all shadow-sm active:scale-95"><Upload size={18} /> {t("employees.bulk_import_btn").toUpperCase()}</button>
          </PermissionGate>
          <PermissionGate permission="employees:create">
            <button data-testid="add-employee-btn" onClick={() => { setEditingEmployee(null); setIsModalOpen(true); }} className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95"><UserPlus size={18} /> {t("employees.add_btn").toUpperCase()}</button>
          </PermissionGate>
        </div>
      </div>

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
        className="animate-in slide-in-from-bottom-8 duration-700"
      />

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
        onTerminate={(emp) => {
          if (window.confirm(t("employees.details.terminate_confirm"))) {
            updateMutation.mutate({ id: emp.id, employee: { is_active: false, termination_date: format(new Date(), 'yyyy-MM-dd') } });
          }
        }}
        onRehire={() => setIsRehireModalOpen(true)}
        onTransfer={(deptId) => { setSelectedDeptId(deptId); setIsTransferModalOpen(true); }}
      />

      {/* Main Employee Form Modal */}
      <Modal isOpen={isModalOpen} onClose={() => setIsModalOpen(false)} title={editingEmployee ? t("employees.form.edit_title") : t("employees.form.add_title")} subtitle={t("employees.form.subtitle")} icon={<UserCircle size={24} />} maxWidth="lg">
        <form onSubmit={handleSubmit} className="space-y-8 pt-4 animate-in fade-in duration-500">
          <div className="space-y-3 px-1">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("employees.form.full_name")}</label>
            <input name="full_name" data-testid="employees-form-name-input" required defaultValue={editingEmployee?.full_name} className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm uppercase tracking-tight shadow-sm" />
          </div>
          <div className="space-y-3 px-1"><label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("employees.form.employee_id")}</label><input name="employee_id_number" data-testid="employees-form-id-input" required defaultValue={editingEmployee?.employee_id_number || prefillEnrollmentId} className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm tracking-widest uppercase shadow-sm" /></div>
          <div className="space-y-3 px-1"><label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("employees.form.hire_date")}</label><input type="date" name="hire_date" data-testid="employees-form-hire-date-input" required defaultValue={editingEmployee?.hire_date} className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm shadow-sm" /></div>
          <div className="flex items-center gap-4 py-2 px-2 group cursor-pointer">
            <input type="checkbox" name="is_active" id="is_active" data-testid="employees-form-active-checkbox" defaultChecked={editingEmployee ? editingEmployee.is_active : true} className="w-7 h-7 text-primary-600 dark:text-primary-400 rounded-xl border-slate-200 dark:border-slate-700 focus:ring-primary-500 transition-all cursor-pointer shadow-inner" />
            <label htmlFor="is_active" className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest cursor-pointer group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors">{t("employees.form.is_active")}</label>
          </div>
          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
            <button type="button" onClick={() => setIsModalOpen(false)} className="px-8 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95">{t("employees.form.cancel").toUpperCase()}</button>
            <button type="submit" data-testid="save-employee-btn" className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 dark:shadow-black/30 active:scale-95">{editingEmployee ? t("employees.form.update").toUpperCase() : t("employees.form.save").toUpperCase()}</button>
          </div>
        </form>
      </Modal>

      {/* Transfer Modal */}
      <Modal isOpen={isTransferModalOpen} onClose={() => { setIsTransferModalOpen(false); setSelectedDeptId(""); }} title={t("employees.transfer.title")} subtitle={t("employees.transfer.subtitle")} icon={<Building2 size={24} />} maxWidth="md">
        <form onSubmit={handleTransferSubmit} className="space-y-8 pt-4 animate-in fade-in duration-500">
          <div className="space-y-3 px-1"><label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("employees.transfer.new_dept")}</label><Combobox options={departments?.map(d => ({ value: d.id, label: d.name.toUpperCase(), icon: <Building2 size={14} /> })) || []} value={selectedDeptId} onChange={setSelectedDeptId} placeholder={t("employees.transfer.select_dept")} className="bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 h-14" /></div>
          <div className="space-y-3 px-1"><label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("employees.transfer.effective_date")}</label><input type="date" name="effective_date" required defaultValue={format(new Date(), 'yyyy-MM-dd')} className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm uppercase tracking-tight shadow-sm" /></div>
          <div className="space-y-3 px-1"><label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("employees.transfer.notes")}</label><textarea name="notes" className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-medium text-sm h-32 resize-none shadow-sm" placeholder={t("employees.transfer.notes_placeholder")} /></div>
          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
            <button type="button" onClick={() => setIsTransferModalOpen(false)} className="px-8 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95">{t("common.cancel").toUpperCase()}</button>
            <button type="submit" data-testid="submit-transfer-btn" className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 dark:shadow-black/30 active:scale-95">{t("employees.transfer.submit").toUpperCase()}</button>
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
                    {!period.termination_date && (<span className="px-3 py-1 bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 text-[9px] font-bold rounded-full uppercase tracking-widest border border-emerald-100 dark:border-emerald-800 shadow-sm">{t("employees.history.currently_working")}</span>)}
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
        <div className="mt-10 pt-8 border-t border-slate-50 dark:border-slate-800 flex justify-end"><button onClick={() => setIsHistoryModalOpen(false)} className="px-12 py-4 bg-slate-900 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 dark:shadow-black/30 active:scale-95">{t("common.close").toUpperCase()}</button></div>
      </Modal>

      {/* Rehire Modal */}
      <Modal isOpen={isRehireModalOpen} onClose={() => setIsRehireModalOpen(false)} title={t("employees.rehire.title")} subtitle={t("employees.rehire.subtitle")} icon={<UserPlus size={24} />} maxWidth="md">
        <div className="space-y-10 pt-4 animate-in fade-in duration-500">
          <div className="p-8 bg-slate-50 dark:bg-slate-800 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-inner text-center italic"><p className="text-sm font-bold text-slate-600 dark:text-slate-300 uppercase tracking-tight leading-relaxed">"{t("employees.rehire.confirm_msg")}"</p></div>
          <form onSubmit={handleRehireSubmit} className="space-y-8">
            <div className="space-y-3 px-1"><label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("employees.rehire.rehire_date")}</label><input type="date" name="hire_date" required defaultValue={format(new Date(), 'yyyy-MM-dd')} className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm uppercase tracking-tight shadow-sm" /></div>
            <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
              <button type="button" onClick={() => setIsRehireModalOpen(false)} className="px-8 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95">{t("common.cancel").toUpperCase()}</button>
              <button type="submit" data-testid="confirm-rehire-btn" disabled={rehireMutation.isPending} className="px-8 py-4 bg-primary-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50">{rehireMutation.isPending ? t("employees.rehire.processing").toUpperCase() : t("employees.rehire.confirm").toUpperCase()}</button>
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
        ]}
      />
    </div>
  );
}
