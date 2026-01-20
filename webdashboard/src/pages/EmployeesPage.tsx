import React, { useState } from "react";
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

type DetailTabType = "profile" | "salary" | "history";

export default function EmployeesPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const { formatDate } = useDateTimeFormatter();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingEmployee, setEditingEmployee] = useState<Employee | null>(null);
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [activeDetailTab, setActiveDetailTab] = useState<DetailTabType>("profile");
  
  // States for sub-modals
  const [isTransferModalOpen, setIsTransferModalOpen] = useState(false);
  const [isHistoryModalOpen, setIsHistoryModalOpen] = useState(false);
  const [isRehireModalOpen, setIsRehireModalOpen] = useState(false);

  // Combobox states
  const [selectedDeptId, setSelectedDeptId] = useState<string>("");

  const { data: employees = [], isLoading } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId),
    enabled: !!wsId,
  });

  const { data: departments } = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId),
    enabled: !!wsId,
  });

  const { data: deptHistory } = useQuery({
    queryKey: ["deptHistory", selectedEmployee?.id],
    queryFn: () => fetchEmployeeDepartmentHistory(wsId, selectedEmployee!.id),
    enabled: !!selectedEmployee,
  });

  const { data: employmentHistory } = useQuery({
    queryKey: ["employmentHistory", selectedEmployee?.id],
    queryFn: () => fetchEmploymentHistory(wsId, selectedEmployee!.id),
    enabled: !!selectedEmployee,
  });

  const createMutation = useMutation({
    mutationFn: (data: Partial<Employee>) => createEmployee(wsId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      setIsModalOpen(false);
      toast.success(t("common.success"));
    },
  });

  const updateMutation = useMutation({
    mutationFn: (data: { id: string; employee: Partial<Employee> }) => 
      updateEmployee(wsId, data.id, data.employee),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      setIsModalOpen(false);
      setEditingEmployee(null);
      toast.success(t("common.success"));
    },
  });

  const transferMutation = useMutation({
    mutationFn: (data: any) => transferEmployee(wsId, selectedEmployee!.id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["deptHistory", selectedEmployee?.id] });
      setIsTransferModalOpen(false);
      setSelectedDeptId("");
      toast.success(t("common.success"));
    },
  });

  const rehireMutation = useMutation({
    mutationFn: (data: { hire_date: string }) => rehireEmployee(wsId, selectedEmployee!.id, data),
    onSuccess: (updatedEmp) => {
      queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      queryClient.invalidateQueries({ queryKey: ["employmentHistory", selectedEmployee?.id] });
      setSelectedEmployee(updatedEmp);
      setIsRehireModalOpen(false);
      toast.success(t("common.success"));
    },
  });

  const toast = useToast();

  const filteredEmployees = employees.filter(emp => 
    `${emp.first_name} ${emp.last_name}`.toLowerCase().includes(searchTerm.toLowerCase()) ||
    emp.employee_id_number.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const data = {
      first_name: formData.get("first_name") as string,
      last_name: formData.get("last_name") as string,
      employee_id_number: formData.get("employee_id_number") as string,
      hire_date: formData.get("hire_date") as string,
      is_active: formData.get("is_active") === "on",
    };

    if (editingEmployee) {
      updateMutation.mutate({ id: editingEmployee.id, employee: data });
    } else {
      createMutation.mutate(data);
    }
  };

  const handleTransferSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    transferMutation.mutate({
      department_id: selectedDeptId,
      effective_date: formData.get("effective_date"),
      notes: formData.get("notes"),
    });
  };

  const handleRehireSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    rehireMutation.mutate({
      hire_date: formData.get("hire_date") as string,
    });
  };

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center py-20">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
      </div>
    );
  }

  return (
    <div className="flex h-[calc(100vh-10rem)] gap-8 overflow-hidden animate-in fade-in duration-700">
      {/* Left List */}
      <div className={`flex-1 flex flex-col gap-6 overflow-hidden transition-all duration-500 ${selectedEmployee ? 'max-w-[50%]' : 'max-w-full'}`}>
        <div className="flex justify-between items-center shrink-0">
          <div>
            <h1 className="text-3xl font-bold tracking-tight text-slate-900">{t("employees.title")}</h1>
            <p className="text-slate-500 mt-1">{t("employees.subtitle")}</p>
          </div>
          <button 
            onClick={() => { setEditingEmployee(null); setIsModalOpen(true); }}
            className="flex items-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 active:scale-95 font-black text-[10px] uppercase tracking-widest"
          >
            <UserPlus className="w-5 h-5" />
            {t("employees.add_btn")}
          </button>
        </div>

        <div className="relative shrink-0 group">
          <Search className="absolute left-5 top-1/2 -translate-y-1/2 text-slate-400 w-5 h-5 group-focus-within:text-primary-500 transition-colors" />
          <input 
            type="text" 
            placeholder={t("employees.search_placeholder")}
            className="w-full pl-14 pr-6 py-4 bg-white border border-slate-200 rounded-[1.5rem] shadow-sm focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 transition-all font-bold text-sm"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>

        <div className="bg-white border border-slate-200 rounded-[2rem] flex-1 overflow-auto shadow-sm">
          <table className="w-full text-left border-collapse">
            <thead className="bg-slate-50/50 sticky top-0 backdrop-blur-md z-10 border-b border-slate-100">
              <tr>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("employees.table.name")}</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("employees.table.id")}</th>
                {!selectedEmployee && (
                  <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("employees.table.status")}</th>
                )}
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {filteredEmployees.length === 0 ? (
                <tr><td colSpan={5} className="px-8 py-20 text-center text-slate-400 font-bold italic">{t("employees.table.no_data")}</td></tr>
              ) : filteredEmployees.map((emp) => (
                <tr 
                  key={emp.id} 
                  onClick={() => setSelectedEmployee(emp)}
                  className={`cursor-pointer transition-all group ${selectedEmployee?.id === emp.id ? 'bg-primary-50/50' : 'hover:bg-slate-50/50'}`}
                >
                  <td className="px-8 py-5">
                    <div className="flex items-center gap-4">
                      <div className="w-10 h-10 bg-slate-100 rounded-xl flex items-center justify-center text-slate-400 group-hover:bg-primary-100 group-hover:text-primary-600 transition-all duration-300 font-black">
                        {emp.last_name[0]}
                      </div>
                      <div className="font-black text-slate-900 text-sm">{emp.first_name} {emp.last_name}</div>
                    </div>
                  </td>
                  <td className="px-8 py-5 font-mono text-xs font-bold text-slate-500">{emp.employee_id_number}</td>
                  {!selectedEmployee && (
                    <td className="px-8 py-5">
                      <span className={`px-3 py-1 rounded-full text-[9px] font-black uppercase tracking-tighter border ${emp.is_active ? 'bg-emerald-50 text-emerald-600 border-emerald-100' : 'bg-slate-50 text-slate-400 border-slate-100'}`}>
                        {emp.is_active ? t("employees.table.active") : t("employees.table.terminated")}
                      </span>
                    </td>
                  )}
                  <td className="px-8 py-5 text-right">
                    <ChevronRight className={`w-5 h-5 text-slate-300 transition-transform ${selectedEmployee?.id === emp.id ? 'translate-x-1 text-primary-500' : 'group-hover:text-primary-400'}`} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Right Detail Panel */}
      {selectedEmployee && (
        <div className="w-[50%] bg-white border border-slate-200 rounded-[2.5rem] flex flex-col shadow-2xl shadow-slate-200/50 overflow-hidden animate-in slide-in-from-right duration-500">
          <div className="p-8 border-b border-slate-50 bg-slate-50/30">
            <div className="flex justify-between items-start mb-6">
              <div className="flex items-center gap-5">
                <div className="w-16 h-16 bg-primary-600 text-white rounded-[1.5rem] flex items-center justify-center font-black text-2xl shadow-lg shadow-primary-100">
                  {selectedEmployee.last_name[0]}
                </div>
                <div>
                  <h2 className="text-2xl font-black text-slate-900 tracking-tight leading-tight">{selectedEmployee.first_name} {selectedEmployee.last_name}</h2>
                  <p className="text-primary-600 font-mono text-xs font-black mt-1 tracking-widest uppercase">{selectedEmployee.employee_id_number}</p>
                </div>
              </div>
              <button onClick={() => setSelectedEmployee(null)} className="p-2.5 hover:bg-white rounded-xl transition-all shadow-sm active:scale-90 border border-slate-100">
                <X className="w-5 h-5 text-slate-400" />
              </button>
            </div>

            {/* Tabs inside Detail Panel */}
            <div className="flex items-center gap-1 bg-white/50 p-1 rounded-2xl w-fit border border-slate-200/50">
              <button 
                onClick={() => setActiveDetailTab("profile")}
                className={`px-4 py-2 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all ${
                  activeDetailTab === 'profile' ? 'bg-white text-primary-600 shadow-sm border border-slate-100' : 'text-slate-500 hover:text-slate-700'
                }`}
              >
                {t("common.profile")}
              </button>
              <button 
                onClick={() => setActiveDetailTab("salary")}
                className={`px-4 py-2 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all ${
                  activeDetailTab === 'salary' ? 'bg-white text-primary-600 shadow-sm border border-slate-100' : 'text-slate-500 hover:text-slate-700'
                }`}
              >
                {t("employees.details.salary_benefits")}
              </button>
              <button 
                onClick={() => setActiveDetailTab("history")}
                className={`px-4 py-2 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all ${
                  activeDetailTab === 'history' ? 'bg-white text-primary-600 shadow-sm border border-slate-100' : 'text-slate-500 hover:text-slate-700'
                }`}
              >
                {t("employees.details.work_history")}
              </button>
            </div>
          </div>

          <div className="flex-1 overflow-y-auto p-8 space-y-8 animate-in fade-in duration-500">
            {activeDetailTab === "profile" && (
              <div className="space-y-8 animate-in slide-in-from-bottom-2">
                {/* Actions Grid */}
                <div className="grid grid-cols-2 gap-4">
                  <button 
                    onClick={() => { setEditingEmployee(selectedEmployee); setIsModalOpen(true); }}
                    className="flex items-center justify-center gap-2 py-3 bg-white border border-slate-200 text-slate-700 font-bold rounded-2xl hover:bg-slate-50 transition-all text-xs shadow-sm"
                  >
                    <Edit2 className="w-4 h-4" />
                    {t("employees.details.edit_profile")}
                  </button>
                  {selectedEmployee.is_active ? (
                    <button 
                      onClick={() => {
                        if (window.confirm(t("employees.details.terminate_confirm"))) {
                          updateMutation.mutate({ id: selectedEmployee.id, employee: { is_active: false, termination_date: format(new Date(), 'yyyy-MM-dd') } });
                        }
                      }}
                      className="flex items-center justify-center gap-2 py-3 bg-rose-50 text-rose-600 font-bold rounded-2xl hover:bg-rose-100 transition-all text-xs"
                    >
                      <Trash2 className="w-4 h-4" />
                      {t("employees.details.terminate")}
                    </button>
                  ) : (
                    <button 
                      onClick={() => setIsRehireModalOpen(true)}
                      className="flex items-center justify-center gap-2 py-3 bg-primary-600 text-white font-bold rounded-2xl hover:bg-primary-700 transition-all text-xs shadow-xl shadow-primary-200"
                    >
                      <UserPlus className="w-4 h-4" />
                      {t("employees.details.rehire")}
                    </button>
                  )}
                </div>

                {/* Current Dept */}
                <section className="space-y-4">
                  <div className="flex justify-between items-center px-1">
                    <span className="text-[10px] font-black text-slate-400 uppercase tracking-widest flex items-center gap-2">
                      <Building2 size={12} /> {t("employees.details.current_dept")}
                    </span>
                    <button 
                      onClick={() => {
                        setSelectedDeptId(deptHistory?.[0]?.department_id || "");
                        setIsTransferModalOpen(true);
                      }}
                      className="text-[10px] font-black text-primary-600 uppercase tracking-tighter hover:underline"
                    >
                      {t("employees.details.transfer")}
                    </button>
                  </div>
                  
                  <div className="bg-slate-50 border border-slate-100 p-6 rounded-[2rem] group hover:border-primary-200 transition-all">
                    {deptHistory && deptHistory.length > 0 ? (
                      <div>
                        <div className="font-black text-slate-900 text-xl tracking-tight uppercase">
                          {departments?.find(d => d.id === deptHistory[0].department_id)?.name}
                        </div>
                        <div className="text-[10px] text-slate-400 mt-2 font-black uppercase tracking-widest flex items-center gap-2">
                           <Calendar className="w-3.5 h-3.5" />
                           {t("employees.transfer.effective_date")}: {formatDate(deptHistory[0].effective_date)}
                        </div>
                      </div>
                    ) : (
                      <div className="text-sm text-slate-400 italic font-bold">{t("employees.details.not_assigned_dept")}</div>
                    )}
                  </div>
                </section>

                <section className="space-y-4">
                  <span className="text-[10px] font-black text-slate-400 uppercase tracking-widest flex items-center gap-2 px-1">
                    <Calendar size={12} /> {t("employees.table.hire_date")}
                  </span>
                  <div className="bg-slate-50 border border-slate-100 p-6 rounded-[2rem] font-black text-slate-700 text-lg">
                    {formatDate(selectedEmployee.hire_date)}
                  </div>
                </section>
              </div>
            )}

            {activeDetailTab === "salary" && (
              <div className="animate-in slide-in-from-bottom-2">
                <EmployeeSalaryConfig wsId={wsId} employeeId={selectedEmployee.id} />
              </div>
            )}

            {activeDetailTab === "history" && (
              <div className="space-y-10 animate-in slide-in-from-bottom-2">
                <div className="relative pl-8 space-y-8 before:absolute before:left-3 before:top-2 before:bottom-2 before:w-[2px] before:bg-slate-100">
                  {employmentHistory?.map((period, idx) => (
                    <React.Fragment key={period.id}>
                      {period.termination_date && (
                        <div className="relative animate-in fade-in slide-in-from-left duration-300">
                          <div className="absolute -left-[1.65rem] top-1.5 w-4 h-4 rounded-full bg-rose-500 border-[3px] border-white shadow-md ring-4 ring-rose-50"></div>
                          <div className="text-sm font-black text-slate-800 uppercase tracking-tight">{t("employees.table.terminated")}</div>
                          <div className="text-xs text-slate-500 mt-1 font-bold">{formatDate(period.termination_date)}</div>
                        </div>
                      )}
                      <div className="relative animate-in fade-in slide-in-from-left duration-300">
                        <div className={`absolute -left-[1.65rem] top-1.5 w-4 h-4 rounded-full border-[3px] border-white shadow-md ring-4 ${idx === 0 && !period.termination_date ? 'bg-emerald-500 ring-emerald-50' : 'bg-primary-500 ring-primary-50'}`}></div>
                        <div className="text-sm font-black text-slate-800 uppercase tracking-tight">
                          {period.is_rehire ? t("employees.details.returned") : t("employees.details.joined")}
                        </div>
                        <div className="text-xs text-slate-500 mt-1 font-bold">{formatDate(period.hire_date)}</div>
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
      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title={editingEmployee ? t("employees.form.edit_title") : t("employees.form.add_title")}
        subtitle={t("employees.form.subtitle")}
        icon={<UserCircle className="w-6 h-6" />}
        maxWidth="lg"
      >
        <form onSubmit={handleSubmit} className="space-y-6">
          <div className="grid grid-cols-2 gap-6">
            <div className="space-y-2">
              <label className="text-xs font-black text-slate-400 uppercase tracking-widest px-1">{t("employees.form.last_name")}</label>
              <input name="last_name" required defaultValue={editingEmployee?.last_name} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 outline-none transition-all font-bold" />
            </div>
            <div className="space-y-2">
              <label className="text-xs font-black text-slate-400 uppercase tracking-widest px-1">{t("employees.form.first_name")}</label>
              <input name="first_name" required defaultValue={editingEmployee?.first_name} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 outline-none transition-all font-bold" />
            </div>
          </div>
          <div className="space-y-2">
            <label className="text-xs font-black text-slate-400 uppercase tracking-widest px-1">{t("employees.form.employee_id")}</label>
            <input name="employee_id_number" required defaultValue={editingEmployee?.employee_id_number} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 outline-none transition-all font-mono font-bold" />
          </div>
          <div className="space-y-2">
            <label className="text-xs font-black text-slate-400 uppercase tracking-widest px-1">{t("employees.form.hire_date")}</label>
            <input type="date" name="hire_date" required defaultValue={editingEmployee?.hire_date} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 outline-none transition-all font-bold" />
          </div>
          <div className="flex items-center gap-4 py-2">
            <input type="checkbox" name="is_active" id="is_active" defaultChecked={editingEmployee ? editingEmployee.is_active : true} className="w-6 h-6 text-primary-600 rounded-xl border-slate-200 focus:ring-primary-500 transition-all cursor-pointer" />
            <label htmlFor="is_active" className="text-sm font-black text-slate-700 uppercase tracking-widest cursor-pointer">{t("employees.form.is_active")}</label>
          </div>
          <div className="pt-6 grid grid-cols-2 gap-5">
            <button type="button" onClick={() => setIsModalOpen(false)} className="px-8 py-4 border border-slate-100 rounded-2xl font-bold text-slate-500 hover:bg-slate-50 transition-all">{t("employees.form.cancel")}</button>
            <button type="submit" data-testid="save-employee-btn" className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black hover:bg-slate-800 transition-all shadow-xl shadow-slate-200 hover:scale-[1.02] active:scale-95">
              {editingEmployee ? t("employees.form.update") : t("employees.form.save")}
            </button>
          </div>
        </form>
      </Modal>


      {/* Transfer Modal */}
      <Modal
        isOpen={isTransferModalOpen}
        onClose={() => {
          setIsTransferModalOpen(false);
          setSelectedDeptId("");
        }}
        title={t("employees.transfer.title")}
        subtitle={t("employees.transfer.subtitle")}
        icon={<Building2 className="w-6 h-6" />}
      >
        <form onSubmit={handleTransferSubmit} className="space-y-5">
          <div className="space-y-2">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("employees.transfer.new_dept")}</label>
            <Combobox
              options={departments?.map(d => ({
                value: d.id,
                label: d.name,
                icon: <Building2 className="w-4 h-4" />
              })) || []}
              value={selectedDeptId}
              onChange={setSelectedDeptId}
              placeholder={t("employees.transfer.select_dept")}
              className="bg-slate-50 border-slate-200"
            />
          </div>
          <div>
            <label className="block text-sm font-bold text-slate-700 mb-2">{t("employees.transfer.effective_date")}</label>
            <input type="date" name="effective_date" required defaultValue={format(new Date(), 'yyyy-MM-dd')} className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl outline-none" />
          </div>
          <div>
            <label className="block text-sm font-bold text-slate-700 mb-2">{t("employees.transfer.notes")}</label>
            <textarea name="notes" className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl outline-none h-24 resize-none" />
          </div>
          <div className="grid grid-cols-2 gap-4 pt-4">
            <button type="button" onClick={() => setIsTransferModalOpen(false)} className="px-6 py-3 border border-slate-200 text-slate-600 font-bold rounded-2xl">{t("employees.transfer.cancel")}</button>
            <button type="submit" data-testid="submit-transfer-btn" className="px-6 py-3 bg-slate-900 text-white font-bold rounded-2xl shadow-lg shadow-slate-100 hover:bg-slate-800">{t("employees.transfer.submit")}</button>
          </div>
        </form>
      </Modal>

      {/* History Modal */}
      <Modal
        isOpen={isHistoryModalOpen && !!selectedEmployee}
        onClose={() => setIsHistoryModalOpen(false)}
        title={t("employees.history.title")}
        subtitle={t("employees.history.subtitle")}
        icon={<HistoryIcon className="w-6 h-6" />}
        maxWidth="lg"
      >
        <div className="space-y-4">
          {employmentHistory?.map((period) => (
            <div key={period.id} className="p-4 bg-slate-50 border border-slate-100 rounded-2xl">
              <div className="flex justify-between items-start">
                <div>
                  <div className="font-bold text-slate-900 flex items-center gap-2">
                    {period.is_rehire ? t("employees.history.rehire_period") : t("employees.history.first_period")}
                    {!period.termination_date && (
                      <span className="px-2 py-0.5 bg-green-100 text-green-700 text-[9px] font-bold rounded-full uppercase">{t("employees.history.currently_working")}</span>
                    )}
                  </div>
                  <div className="text-sm text-slate-500 mt-1">
                    {t("employees.history.period_range", { start: formatDate(period.hire_date), end: period.termination_date ? formatDate(period.termination_date) : t("employees.history.present") })}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
        
        <div className="mt-8">
          <button 
            onClick={() => setIsHistoryModalOpen(false)}
            className="w-full py-3 bg-slate-100 text-slate-600 font-bold rounded-2xl hover:bg-slate-200 transition-all"
          >
            {t("employees.history.close")}
          </button>
        </div>
      </Modal>

      {/* Rehire Modal */}
      <Modal
        isOpen={isRehireModalOpen && !!selectedEmployee}
        onClose={() => setIsRehireModalOpen(false)}
        title={t("employees.rehire.title")}
        subtitle={t("employees.rehire.subtitle")}
        icon={<UserPlus className="w-6 h-6" />}
      >
        <p className="text-slate-500 mb-6 font-medium">{t("employees.rehire.confirm_msg")}</p>
        
        <form onSubmit={handleRehireSubmit} className="space-y-5">
          <div>
            <label className="block text-sm font-bold text-slate-700 mb-2">{t("employees.rehire.rehire_date")}</label>
            <input 
              type="date" 
              name="hire_date" 
              required 
              defaultValue={format(new Date(), 'yyyy-MM-dd')} 
              className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-2xl outline-none" 
            />
          </div>
          <div className="grid grid-cols-2 gap-4 pt-4">
            <button type="button" onClick={() => setIsRehireModalOpen(false)} className="px-6 py-3 border border-slate-200 text-slate-600 font-bold rounded-2xl">{t("employees.rehire.cancel")}</button>
            <button type="submit" data-testid="confirm-rehire-btn" disabled={rehireMutation.isPending} className="px-6 py-3 bg-primary-600 text-white font-bold rounded-2xl shadow-lg shadow-primary-100 hover:bg-primary-700 disabled:opacity-50">
              {rehireMutation.isPending ? t("employees.rehire.processing") : t("employees.rehire.confirm")}
            </button>
          </div>
        </form>
      </Modal>
    </div>
  );
}