import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Trash2,
  UserCircle,
  Building2,
  Calendar,
  Clock,
  Briefcase,
  Search,
  Plus,
  UserPlus,
  ChevronRight,
  X,
  Edit2,
  ArrowRightLeft,
  Banknote,
  History as HistoryIcon,
  Check,
  Calendar as CalendarIcon,
  Upload
} from "lucide-react";

import { 
  fetchEmployees, 
  createEmployee, 
  updateEmployee, 
  deleteEmployee, 
  getCurrentWorkspaceId,
  Employee,
  fetchDepartments,
  fetchEmployeeDepartmentHistory,
  transferEmployee,
  fetchEmploymentHistory,
  rehireEmployee
} from "../lib/api";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { format } from "date-fns";
import { Combobox } from "@/components/ui/Combobox";
import Modal from "@/components/ui/Modal";
import { EmployeeSearch } from "@/components/common/EmployeeSearch";
import EmployeeSalaryConfig from "@/components/payroll/EmployeeSalaryConfig";
import { useToast } from "@/components/Toast";
import { cn } from "@/lib/utils";

import AttendanceImportDialog from "@/components/AttendanceImportDialog";
import WorkingConditionConfig from "@/components/employees/WorkingConditionConfig";

type DetailTabType = "profile" | "salary" | "history" | "policies";

export default function EmployeesPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const { formatDate } = useDateTimeFormatter();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);
  const [editingEmployee, setEditingEmployee] = useState<Employee | null>(null);
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [activeDetailTab, setActiveDetailTab] = useState<DetailTabType>("profile");
  
  const [isTransferModalOpen, setIsTransferModalOpen] = useState(false);
  const [isHistoryModalOpen, setIsHistoryModalOpen] = useState(false);
  const [isRehireModalOpen, setIsRehireModalOpen] = useState(false);
  const [selectedDeptId, setSelectedDeptId] = useState<string>("");

  const { data: employees = [], isLoading } = useQuery({ queryKey: ["employees", wsId], queryFn: () => fetchEmployees(wsId), enabled: !!wsId });
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

  const filteredEmployees = employees.filter(emp => 
    emp.full_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    emp.employee_id_number.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault(); const formData = new FormData(e.currentTarget);
    const data = { full_name: formData.get("full_name") as string, employee_id_number: formData.get("employee_id_number") as string, hire_date: formData.get("hire_date") as string, is_active: formData.get("is_active") === "on" };
    if (editingEmployee) updateMutation.mutate({ id: editingEmployee.id, employee: data });
    else createMutation.mutate(data);
  };

  const handleTransferSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault(); const formData = new FormData(e.currentTarget);
    transferMutation.mutate({ department_id: selectedDeptId, effective_date: formData.get("effective_date"), notes: formData.get("notes") });
  };

  const handleRehireSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault(); const formData = new FormData(e.currentTarget);
    rehireMutation.mutate({ hire_date: formData.get("hire_date") as string });
  };

  if (isLoading) return <div className="flex h-full items-center justify-center py-32"><div className="animate-spin rounded-full h-12 w-12 border-4 border-primary-600 border-t-transparent shadow-xl"></div></div>;

  return (
    <div className="flex h-[calc(100vh-10rem)] gap-8 overflow-hidden animate-in fade-in duration-700">
      <div className={cn("flex flex-col gap-8 overflow-hidden transition-all duration-500", selectedEmployee ? 'flex-1 max-w-[45%]' : 'w-full')}>
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
          <div className="space-y-1">
            <h1 data-testid="employees-title" className="text-4xl font-black tracking-tighter text-slate-900 uppercase leading-none">{t("employees.title")}</h1>
            <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest mt-1.5 opacity-70">{t("employees.subtitle")}</p>
          </div>
          <div className="flex items-center gap-4">
            <button data-testid="import-employee-btn" onClick={() => setIsImportDialogOpen(true)} className="flex items-center gap-3 bg-white border border-slate-200 text-slate-700 px-8 py-3 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-slate-50 transition-all shadow-sm active:scale-95"><Upload size={18} /> {t("employees.bulk_import_btn").toUpperCase()}</button>
            <button data-testid="add-employee-btn" onClick={() => { setEditingEmployee(null); setIsModalOpen(true); }} className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95"><UserPlus size={18} /> {t("employees.add_btn").toUpperCase()}</button>
          </div>
        </div>

        <div className="relative shrink-0 group px-1">
          <Search className="absolute left-6 top-1/2 -translate-y-1/2 text-slate-300 group-focus-within:text-primary-500 transition-colors" size={20} />
          <input data-testid="employee-search" type="text" placeholder={t("employees.search_placeholder")} className="w-full pl-14 pr-8 py-4.5 bg-white border border-slate-100 rounded-[1.75rem] shadow-xl shadow-slate-900/5 focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:bg-white transition-all font-black text-sm uppercase tracking-tight" value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} />
        </div>

        <div className="bg-white border border-slate-100 rounded-[2.5rem] flex-1 overflow-auto shadow-sm p-2">
          <table className="w-full text-left border-collapse">
            <thead className="bg-slate-50/50 sticky top-0 backdrop-blur-md z-10 border-b border-slate-100">
              <tr>
                <th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em]">{t("employees.table.name")}</th>
                <th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em]">{t("employees.table.id")}</th>
                {!selectedEmployee && (<th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] text-center">{t("employees.table.status")}</th>)}
                <th className="px-8 py-5 border-b border-slate-100"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {filteredEmployees.length === 0 ? (
                <tr><td colSpan={5} className="px-8 py-32 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("employees.table.no_data")}</td></tr>
              ) : filteredEmployees.map((emp) => (
                <tr key={emp.id} onClick={() => setSelectedEmployee(emp)} className={cn("cursor-pointer transition-all group relative", selectedEmployee?.id === emp.id ? "bg-primary-50/50" : "hover:bg-slate-50/50")}>
                  <td className="px-8 py-6">
                    <div className="flex items-center gap-5">
                      <div className={cn("w-12 h-12 rounded-2xl flex items-center justify-center font-black text-sm shadow-inner border transition-all uppercase", selectedEmployee?.id === emp.id ? "bg-primary-600 text-white border-primary-500" : "bg-slate-100 text-slate-400 border-slate-100 group-hover:bg-white group-hover:text-primary-600 group-hover:border-primary-100")}>{emp.full_name[0]}</div>
                      <div className="font-black text-slate-900 text-sm tracking-tight uppercase leading-none">{emp.full_name}</div>
                    </div>
                  </td>
                  <td className="px-8 py-6 font-mono font-bold text-[11px] text-slate-400 uppercase tracking-tighter">{emp.employee_id_number}</td>
                  {!selectedEmployee && (
                    <td className="px-8 py-6 text-center">
                      <span className={cn("px-3 py-1 rounded-full text-[9px] font-black uppercase tracking-widest border shadow-sm", emp.is_active ? 'bg-emerald-50 text-emerald-600 border-emerald-100' : 'bg-slate-50 text-slate-400 border-slate-100 opacity-60')}>{emp.is_active ? t("employees.table.active").toUpperCase() : t("employees.table.terminated").toUpperCase()}</span>
                    </td>
                  )}
                  <td className="px-8 py-6 text-right">
                    <ChevronRight className={cn("w-5 h-5 transition-all duration-300 ml-auto", selectedEmployee?.id === emp.id ? "translate-x-1 text-primary-600" : "text-slate-200 group-hover:text-primary-400")} />
                  </td>
                  {selectedEmployee?.id === emp.id && <div className="absolute left-0 top-0 bottom-0 w-1.5 bg-primary-600 rounded-r-full shadow-lg shadow-primary-500/50" />}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {selectedEmployee && (
        <div className="flex-1 bg-white border border-slate-100 rounded-[3rem] flex flex-col shadow-premium-2xl overflow-hidden animate-in slide-in-from-right duration-700">
          <div className="p-10 border-b border-slate-50 bg-slate-50/30">
            <div className="flex justify-between items-start mb-8">
              <div className="flex items-center gap-6">
                <div className="w-20 h-20 bg-primary-600 text-white rounded-[2rem] flex items-center justify-center font-black text-3xl shadow-xl shadow-primary-100 border-4 border-white rotate-3 group hover:rotate-0 transition-transform duration-500 uppercase">{selectedEmployee.full_name[0]}</div>
                <div>
                  <h2 className="text-3xl font-black text-slate-900 tracking-tighter uppercase leading-none mb-2">{selectedEmployee.full_name}</h2>
                  <div className="flex items-center gap-3">
                    <span className="text-primary-600 font-mono text-xs font-black tracking-widest uppercase bg-white px-3 py-1 rounded-full border border-primary-100 shadow-sm">{selectedEmployee.employee_id_number}</span>
                    <div className="w-1.5 h-1.5 bg-slate-200 rounded-full" />
                    <span className={cn("text-[9px] font-black uppercase tracking-widest px-2 py-0.5 rounded-full border shadow-sm", selectedEmployee.is_active ? 'bg-emerald-50 text-emerald-600 border-emerald-100' : 'bg-slate-50 text-slate-400 border-slate-100 opacity-60')}>{selectedEmployee.is_active ? t("employees.table.active").toUpperCase() : t("employees.table.terminated").toUpperCase()}</span>
                  </div>
                </div>
              </div>
              <button onClick={() => setSelectedEmployee(null)} className="p-3 bg-white hover:bg-slate-50 rounded-[1.25rem] transition-all shadow-md active:scale-90 border border-slate-100 text-slate-300 hover:text-rose-600"><X size={24} /></button>
            </div>

            <div className="flex bg-white/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50 shadow-inner">
              {[ 
                { id: "profile", label: t("common.profile") }, 
                { id: "salary", label: t("employees.details.salary_benefits") }, 
                { id: "policies", label: t("employees.details.special_policies") },
                { id: "history", label: t("employees.details.work_history") } 
              ].map(tab => (
                <button key={tab.id} onClick={() => setActiveDetailTab(tab.id as DetailTabType)} className={cn("px-8 py-2.5 rounded-[18px] text-[10px] font-black uppercase tracking-widest transition-all", activeDetailTab === tab.id ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}>{tab.label.toUpperCase()}</button>
              ))}
            </div>
          </div>

          <div className="flex-1 overflow-y-auto p-10 space-y-10 animate-in fade-in duration-500 custom-scrollbar scroll-smooth">
            {activeDetailTab === "profile" && (
              // ... existing profile content ...
              <div className="space-y-10 animate-in slide-in-from-bottom-4 duration-500">
                <div className="grid grid-cols-2 gap-6">
                  <button onClick={() => { setEditingEmployee(selectedEmployee); setIsModalOpen(true); }} className="flex items-center justify-center gap-3 py-4 bg-white border border-slate-200 text-slate-700 font-black rounded-2xl hover:bg-slate-50 hover:border-slate-300 transition-all text-xs uppercase tracking-widest shadow-sm active:scale-95"><Edit2 size={18} /> {t("employees.details.edit_profile").toUpperCase()}</button>
                  {selectedEmployee.is_active ? (
                    <button onClick={() => { if (window.confirm(t("employees.details.terminate_confirm"))) { updateMutation.mutate({ id: selectedEmployee.id, employee: { is_active: false, termination_date: format(new Date(), 'yyyy-MM-dd') } }); } }} className="flex items-center justify-center gap-3 py-4 bg-rose-50 text-rose-600 font-black rounded-2xl hover:bg-rose-100 transition-all text-xs uppercase tracking-widest active:scale-95 shadow-sm shadow-rose-900/5"><Trash2 size={18} /> {t("employees.details.terminate").toUpperCase()}</button>
                  ) : (
                    <button onClick={() => setIsRehireModalOpen(true)} className="flex items-center justify-center gap-3 py-4 bg-primary-600 text-white font-black rounded-2xl hover:bg-primary-700 transition-all text-xs uppercase tracking-widest shadow-xl shadow-primary-100 active:scale-95"><UserPlus size={18} /> {t("employees.details.rehire").toUpperCase()}</button>
                  )}
                </div>

                <section className="space-y-5">
                  <div className="flex justify-between items-center px-2">
                    <span className="text-[10px] font-black text-slate-400 uppercase tracking-[0.3em] flex items-center gap-2 opacity-70"><Building2 size={14} className="text-primary-500" /> {t("employees.details.current_dept").toUpperCase()}</span>
                    <button onClick={() => { setSelectedDeptId(deptHistory?.[0]?.department_id || ""); setIsTransferModalOpen(true); }} className="text-[10px] font-black text-primary-600 uppercase tracking-widest hover:underline active:scale-90">{t("employees.details.transfer").toUpperCase()}</button>
                  </div>
                  <div className="bg-slate-50/50 border border-slate-100 p-8 rounded-[2.5rem] group hover:bg-white hover:shadow-xl hover:shadow-primary-100/10 transition-all shadow-inner relative overflow-hidden">
                    {deptHistory && deptHistory.length > 0 ? (
                      <div>
                        <div className="font-black text-slate-900 text-2xl tracking-tighter uppercase leading-none mb-3">{departments?.find(d => d.id === deptHistory[0].department_id)?.name}</div>
                        <div className="flex items-center gap-2 text-[10px] text-slate-400 font-black uppercase tracking-widest opacity-70"><Calendar size={12} className="text-primary-400" /> {t("employees.transfer.effective_date")}: {formatDate(deptHistory[0].effective_date)}</div>
                      </div>
                    ) : (<div className="text-xs text-slate-400 font-bold uppercase tracking-widest italic opacity-60">{t("employees.details.not_assigned_dept")}</div>)}
                    <Building2 className="absolute -right-8 -bottom-8 w-32 h-32 text-slate-900 opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
                  </div>
                </section>

                <section className="space-y-5">
                  <span className="text-[10px] font-black text-slate-400 uppercase tracking-[0.3em] flex items-center gap-2 px-2 opacity-70"><Calendar size={14} className="text-emerald-500" /> {t("employees.table.hire_date").toUpperCase()}</span>
                  <div className="bg-slate-50/50 border border-slate-100 p-8 rounded-[2.5rem] font-black text-slate-800 text-2xl tracking-tighter shadow-inner uppercase font-mono">{formatDate(selectedEmployee.hire_date)}</div>
                </section>
              </div>
            )}

            {activeDetailTab === "salary" && (
              <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
                <EmployeeSalaryConfig wsId={wsId} employeeId={selectedEmployee.id} />
              </div>
            )}

            {activeDetailTab === "policies" && (
              <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
                <WorkingConditionConfig wsId={wsId} employeeId={selectedEmployee.id} />
              </div>
            )}

            {activeDetailTab === "history" && (
              <div className="space-y-12 animate-in fade-in slide-in-from-bottom-4 duration-500 px-2 pt-4">
                <div className="relative pl-12 space-y-12 before:absolute before:left-[11px] before:top-4 before:bottom-4 before:w-px before:bg-slate-100">
                  {employmentHistory?.map((period, idx) => (
                    <React.Fragment key={period.id}>
                      {period.termination_date && (
                        <div className="relative animate-in slide-in-from-left duration-500">
                          <div className="absolute -left-[3.15rem] top-1 w-6 h-6 rounded-full bg-rose-500 border-4 border-white shadow-xl ring-4 ring-rose-50 z-10"></div>
                          <div className="bg-slate-50/50 border border-slate-100 p-6 rounded-[2rem] shadow-inner group hover:bg-white transition-all">
                            <div className="text-sm font-black text-slate-800 uppercase tracking-widest mb-1.5">{t("employees.table.terminated")}</div>
                            <div className="text-[10px] text-rose-500 font-black uppercase tracking-[0.2em] font-mono">{formatDate(period.termination_date)}</div>
                          </div>
                        </div>
                      )}
                      <div className="relative animate-in slide-in-from-left duration-500">
                        <div className={cn("absolute -left-[3.15rem] top-1 w-6 h-6 rounded-full border-4 border-white shadow-xl ring-4 z-10", idx === 0 && !period.termination_date ? 'bg-emerald-500 ring-emerald-50 animate-pulse' : 'bg-primary-600 ring-primary-50')}></div>
                        <div className="bg-white border border-slate-100 p-6 rounded-[2rem] shadow-sm hover:shadow-xl hover:shadow-primary-100/10 transition-all group overflow-hidden">
                          <div className="text-sm font-black text-slate-900 uppercase tracking-widest mb-1.5">{period.is_rehire ? t("employees.details.returned") : t("employees.details.joined")}</div>
                          <div className={cn("text-[10px] font-black uppercase tracking-[0.2em] font-mono", idx === 0 && !period.termination_date ? "text-emerald-600" : "text-slate-400")}>{formatDate(period.hire_date)}</div>
                          <CalendarIcon className="absolute -right-4 -bottom-4 w-16 h-16 text-slate-100 opacity-20 -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
                        </div>
                      </div>
                    </React.Fragment>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Main Employee Form Modal */}
      <Modal isOpen={isModalOpen} onClose={() => setIsModalOpen(false)} title={editingEmployee ? t("employees.form.edit_title") : t("employees.form.add_title")} subtitle={t("employees.form.subtitle")} icon={<UserCircle size={24} />} maxWidth="lg">
        <form onSubmit={handleSubmit} className="space-y-8 pt-4 animate-in fade-in duration-500">
          <div className="space-y-3 px-1">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("employees.form.full_name")}</label>
            <input name="full_name" required defaultValue={editingEmployee?.full_name} className="w-full px-6 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight shadow-sm" />
          </div>
          <div className="space-y-3 px-1"><label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("employees.form.employee_id")}</label><input name="employee_id_number" required defaultValue={editingEmployee?.employee_id_number} className="w-full px-6 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-mono font-black text-sm tracking-widest uppercase shadow-sm" /></div>
          <div className="space-y-3 px-1"><label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("employees.form.hire_date")}</label><input type="date" name="hire_date" required defaultValue={editingEmployee?.hire_date} className="w-full px-6 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm shadow-sm" /></div>
          <div className="flex items-center gap-4 py-2 px-2 group cursor-pointer">
            <input type="checkbox" name="is_active" id="is_active" defaultChecked={editingEmployee ? editingEmployee.is_active : true} className="w-7 h-7 text-primary-600 rounded-xl border-slate-200 focus:ring-primary-500 transition-all cursor-pointer shadow-inner" />
            <label htmlFor="is_active" className="text-sm font-black text-slate-700 uppercase tracking-widest cursor-pointer group-hover:text-primary-600 transition-colors">{t("employees.form.is_active")}</label>
          </div>
          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50">
            <button type="button" onClick={() => setIsModalOpen(false)} className="px-8 py-4 bg-white border border-slate-200 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all active:scale-95">{t("employees.form.cancel").toUpperCase()}</button>
            <button type="submit" data-testid="save-employee-btn" className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all shadow-xl shadow-slate-200 active:scale-95">{editingEmployee ? t("employees.form.update").toUpperCase() : t("employees.form.save").toUpperCase()}</button>
          </div>
        </form>
      </Modal>

      {/* Transfer Modal */}
      <Modal isOpen={isTransferModalOpen} onClose={() => { setIsTransferModalOpen(false); setSelectedDeptId(""); }} title={t("employees.transfer.title")} subtitle={t("employees.transfer.subtitle")} icon={<Building2 size={24} />} maxWidth="md">
        <form onSubmit={handleTransferSubmit} className="space-y-8 pt-4 animate-in fade-in duration-500">
          <div className="space-y-3 px-1"><label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("employees.transfer.new_dept")}</label><Combobox options={departments?.map(d => ({ value: d.id, label: d.name.toUpperCase(), icon: <Building2 size={14} /> })) || []} value={selectedDeptId} onChange={setSelectedDeptId} placeholder={t("employees.transfer.select_dept")} className="bg-slate-50 border border-slate-100 h-14" /></div>
          <div className="space-y-3 px-1"><label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("employees.transfer.effective_date")}</label><input type="date" name="effective_date" required defaultValue={format(new Date(), 'yyyy-MM-dd')} className="w-full px-6 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight shadow-sm" /></div>
          <div className="space-y-3 px-1"><label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("employees.transfer.notes")}</label><textarea name="notes" className="w-full px-6 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-medium text-sm h-32 resize-none shadow-sm" placeholder={t("employees.transfer.notes_placeholder")} /></div>
          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50">
            <button type="button" onClick={() => setIsTransferModalOpen(false)} className="px-8 py-4 bg-white border border-slate-200 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all active:scale-95">{t("common.cancel").toUpperCase()}</button>
            <button type="submit" data-testid="submit-transfer-btn" className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all shadow-xl shadow-slate-200 active:scale-95">{t("employees.transfer.submit").toUpperCase()}</button>
          </div>
        </form>
      </Modal>

      {/* History Modal */}
      <Modal isOpen={isHistoryModalOpen} onClose={() => setIsHistoryModalOpen(false)} title={t("employees.history.title")} subtitle={t("employees.history.subtitle")} icon={<HistoryIcon size={24} />} maxWidth="lg">
        <div className="space-y-4 pt-4 animate-in fade-in duration-500">
          {employmentHistory?.map((period) => (
            <div key={period.id} className="p-8 bg-slate-50 border border-slate-100 rounded-[2.5rem] shadow-inner group hover:bg-white transition-all relative overflow-hidden">
              <div className="flex justify-between items-center relative z-10">
                <div>
                  <div className="font-black text-slate-900 text-lg uppercase tracking-tight flex items-center gap-4">
                    {period.is_rehire ? t("employees.history.rehire_period") : t("employees.history.first_period")}
                    {!period.termination_date && (<span className="px-3 py-1 bg-emerald-50 text-emerald-600 text-[9px] font-black rounded-full uppercase tracking-widest border border-emerald-100 shadow-sm">{t("employees.history.currently_working")}</span>)}
                  </div>
                  <div className="text-[10px] text-slate-400 font-black uppercase tracking-[0.25em] mt-3 flex items-center gap-3 opacity-70">
                    <CalendarIcon size={14} className="text-primary-400" /> {t("employees.history.period_range", { start: formatDate(period.hire_date), end: period.termination_date ? formatDate(period.termination_date) : t("employees.history.present").toUpperCase() })}
                  </div>
                </div>
              </div>
              <CalendarIcon className="absolute -right-8 -bottom-8 w-32 h-32 text-slate-900 opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
            </div>
          ))}
        </div>
        <div className="mt-10 pt-8 border-t border-slate-50 flex justify-end"><button onClick={() => setIsHistoryModalOpen(false)} className="px-12 py-4 bg-slate-900 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all shadow-xl shadow-slate-200 active:scale-95">{t("common.close").toUpperCase()}</button></div>
      </Modal>

      {/* Rehire Modal */}
      <Modal isOpen={isRehireModalOpen} onClose={() => setIsRehireModalOpen(false)} title={t("employees.rehire.title")} subtitle={t("employees.rehire.subtitle")} icon={<UserPlus size={24} />} maxWidth="md">
        <div className="space-y-10 pt-4 animate-in fade-in duration-500">
          <div className="p-8 bg-slate-50 rounded-[2.5rem] border border-slate-100 shadow-inner text-center italic"><p className="text-sm font-black text-slate-600 uppercase tracking-tight leading-relaxed">"{t("employees.rehire.confirm_msg")}"</p></div>
          <form onSubmit={handleRehireSubmit} className="space-y-8">
            <div className="space-y-3 px-1"><label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("employees.rehire.rehire_date")}</label><input type="date" name="hire_date" required defaultValue={format(new Date(), 'yyyy-MM-dd')} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight shadow-sm" /></div>
            <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50">
              <button type="button" onClick={() => setIsRehireModalOpen(false)} className="px-8 py-4 bg-white border border-slate-200 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all active:scale-95">{t("common.cancel").toUpperCase()}</button>
              <button type="submit" data-testid="confirm-rehire-btn" disabled={rehireMutation.isPending} className="px-8 py-4 bg-primary-600 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95 disabled:opacity-50">{rehireMutation.isPending ? t("employees.rehire.processing").toUpperCase() : t("employees.rehire.confirm").toUpperCase()}</button>
            </div>
          </form>
        </div>
      </Modal>

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
