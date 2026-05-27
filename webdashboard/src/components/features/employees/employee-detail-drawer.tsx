import { useQuery } from "@tanstack/react-query";
import {
    Building2,
    Calendar,
    Pencil,
    Trash2,
    User,
    UserPlus,
    X,
} from "lucide-react";
import React, { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import type { Department, Employee, WorkspaceMember } from "@/lib/api";

import PermissionGate from "@/components/common/permission-gate";
import WorkingConditionConfig from "@/components/employees/working-condition-config";
import EmployeeSalaryConfig from "@/components/payroll/employee-salary-config";
import Button from "@/components/ui/button";
import Sheet from "@/components/ui/sheet";
import { Typography } from "@/components/ui/typography";
import { fetchWorkspaceMembers } from "@/lib/api";
import { cn } from "@/lib/utils";

import EmployeeContractsTab from "./employee-contracts-tab";
import EmployeePhasesTab from "./employee-phases-tab";
import EmployeeAssignmentsTab from "./employee-assignments-tab";


type DetailTabType = "profile" | "contracts" | "phases" | "assignments" | "salary" | "policies";
const TAB_ORDER: DetailTabType[] = ["profile", "contracts", "phases", "assignments", "salary", "policies"];
const SCROLL_STEP = 80;

interface DeptHistoryEntry {
    department_id: string;
    effective_date: string;
 
}
 

interface EmployeeDetailDrawerProps {
    departments?: Department[];
    deptHistory?: DeptHistoryEntry[];
    employee: Employee | null;
    formatDate: (date: string) => string;
    isOpen: boolean;
    onClose: () => void;
    onEdit: (employee: Employee) => void;
    onRehire: (employee: Employee) => void;
    onTerminate: (employee: Employee) => void;
    onTransfer: (departmentId: string) => void;
    wsId: string;
}

function DrawerHeader({
    employee,
    onClose,
    activeDetailTab,
    setActiveDetailTab,
    t
}: {
    activeDetailTab: DetailTabType;
    employee: Employee;
    onClose: () => void;
    setActiveDetailTab: (tab: DetailTabType) => void;
    t: (k: string) => string;
}) {
    return (
        <div className="sticky top-0 z-20 p-10 border-b border-slate-50 dark:border-slate-800 bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm shadow-sm">
            <div className="flex justify-between items-start mb-8">
                <div className="flex items-center gap-6">
                    <div className="w-20 h-20 bg-primary-600 text-white rounded-[2rem] flex items-center justify-center font-bold text-3xl shadow-xl shadow-primary-100 dark:shadow-black/30 border-4 border-white rotate-3 group hover:rotate-0 transition-transform duration-500 uppercase">{employee.full_name[0]}</div>
                    <div>
                        <Typography className="text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none mb-2" variant="h2">{employee.full_name}</Typography>
                        <div className="flex items-center gap-3">
                            <span className="text-primary-600 dark:text-primary-400 font-mono text-xs font-bold tracking-widest uppercase bg-white dark:bg-slate-900 px-3 py-1 rounded-full border border-primary-100 dark:border-primary-800 shadow-sm">{employee.employee_id_number}</span>
                            <div className="w-1.5 h-1.5 bg-slate-200 dark:bg-slate-600 rounded-full" />
                            <span className={cn("text-[9px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-full border shadow-sm", employee.is_active ? 'bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800' : 'bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 opacity-60')}>{employee.is_active ? t("employees.table.active").toUpperCase() : t("employees.table.terminated").toUpperCase()}</span>
                        </div>
                    </div>
                </div>
                <Button className="p-3 bg-white dark:bg-slate-900 hover:bg-slate-50 dark:hover:bg-slate-800 rounded-[1.25rem] transition-all shadow-md active:scale-90 border border-slate-100 dark:border-slate-800 text-slate-300 dark:text-slate-600 hover:text-rose-600 dark:text-rose-300 h-auto w-auto shadow-none" onClick={onClose} size="icon" type="button" variant="ghost"><X size={24} /></Button>
            </div>

            <div className="flex overflow-x-auto scrollbar-hide bg-white dark:bg-slate-900/50 p-1.5 rounded-[22px] border border-slate-200 dark:border-slate-700/50 shadow-inner w-full max-w-full relative">
                {[
                    { id: "profile", label: t("common.profile") },
                    { id: "contracts", label: "Hợp đồng" },
                    { id: "phases", label: "Hành trình" },
                    { id: "assignments", label: "Phái cử" },
                    { id: "salary", label: t("employees.details.salary_benefits") },
                    { id: "policies", label: t("employees.details.special_policies") }
                ].map(tab => (
                    <Button className={cn("px-5 py-2.5 shrink-0 whitespace-nowrap rounded-[18px] text-[10px] font-bold uppercase tracking-widest transition-colors duration-200 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 h-auto shadow-none border border-transparent", activeDetailTab === tab.id ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-transparent")} data-testid={`employees-tab-${tab.id}`} key={tab.id} onClick={() => { setActiveDetailTab(tab.id as DetailTabType); }} type="button" variant="ghost">{tab.label.toUpperCase()}</Button>
                ))}
            </div>
        </div>
    );
}



function ProfileTab({
    employee,
    linkedUser,
    departments,
    deptHistory,
    formatDate,
    onEdit,
    onTerminate,
    onRehire,
    onTransfer,
    t
}: {
    departments?: Department[];
    deptHistory?: DeptHistoryEntry[];
    employee: Employee;
    formatDate: (date: string) => string;
    linkedUser: WorkspaceMember | undefined;
    onEdit: (e: Employee) => void;
    onRehire: (e: Employee) => void;
    onTerminate: (e: Employee) => void;
    onTransfer: (id: string) => void;
    t: (k: string) => string;
}) {
    return (
        <div className="space-y-10">
            <div className="grid grid-cols-2 gap-6">
                <PermissionGate permission="employees:update">
                    <Button className="flex items-center justify-center gap-3 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 font-bold rounded-2xl hover:bg-slate-50 dark:hover:bg-slate-800 hover:border-slate-300 dark:hover:border-slate-600 transition-all text-xs uppercase tracking-widest shadow-sm active:scale-95 h-auto" data-testid="employees-btn-edit" onClick={() => { onEdit(employee); }} type="button" variant="outline"><Pencil size={18} /> {t("employees.details.edit_profile").toUpperCase()}</Button>
                </PermissionGate>
                <PermissionGate permission="employees:update">
                    {employee.is_active ? (
                        <Button className="flex items-center justify-center gap-3 py-4 bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 font-bold rounded-2xl hover:bg-rose-100 dark:bg-rose-900 transition-all text-xs uppercase tracking-widest active:scale-95 shadow-sm shadow-rose-900/5 h-auto" data-testid="employees-btn-terminate" onClick={() => { onTerminate(employee); }} type="button" variant="destructive"><Trash2 size={18} /> {t("employees.details.terminate").toUpperCase()}</Button>
                    ) : (
                        <Button className="flex items-center justify-center gap-3 py-4 bg-primary-600 text-white font-bold rounded-2xl hover:bg-primary-700 transition-all text-xs uppercase tracking-widest shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 h-auto" data-testid="employees-btn-rehire" onClick={() => { onRehire(employee); }} type="button" variant="primary"><UserPlus size={18} /> {t("employees.details.rehire").toUpperCase()}</Button>
                    )}
                </PermissionGate>
            </div>

            <section className="space-y-5">
                <div className="flex justify-between items-center px-2">
                    <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em] flex items-center gap-2 opacity-70"><Building2 className="text-primary-500" size={14} /> {t("employees.details.current_dept").toUpperCase()}</span>
                    <PermissionGate permission="employees:update">
                        <Button className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest hover:underline active:scale-90 h-auto shadow-none border-none p-0" onClick={() => { onTransfer(deptHistory?.[0]?.department_id ?? ""); }} type="button" variant="ghost">{t("employees.details.transfer").toUpperCase()}</Button>
                    </PermissionGate>
                </div>
                <div className="bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] group hover:bg-white dark:hover:bg-slate-900 hover:shadow-xl hover:shadow-primary-100/10 transition-all shadow-inner relative overflow-hidden">
                    {deptHistory && deptHistory.length > 0 ? (
                        <div>
                            <div className="font-bold text-slate-700 dark:text-slate-200 text-2xl tracking-tighter uppercase leading-none mb-3">{departments?.find(d => d.id === deptHistory[0].department_id)?.name}</div>
                            <div className="flex items-center gap-2 text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase tracking-widest opacity-70"><Calendar className="text-primary-400" size={12} /> {t("employees.transfer.effective_date")}: {formatDate(deptHistory[0].effective_date)}</div>
                        </div>
                    ) : (<div className="text-xs text-slate-500 dark:text-slate-400 font-bold uppercase tracking-widest italic opacity-60">{t("employees.details.not_assigned_dept")}</div>)}
                    <Building2 className="absolute -right-8 -bottom-8 w-32 h-32 text-slate-700 dark:text-slate-200 opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
                </div>
            </section>

            <section className="space-y-5">
                <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em] flex items-center gap-2 px-2 opacity-70"><User className="text-primary-500" size={14} /> {t("common.user").toUpperCase()}</span>
                <div className="bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] group hover:bg-white dark:hover:bg-slate-900 hover:shadow-xl hover:shadow-primary-100/10 transition-all shadow-inner relative overflow-hidden">
                    {linkedUser ? (
                        <div>
                            <div className="font-bold text-slate-700 dark:text-slate-200 text-lg tracking-tight uppercase leading-none mb-1">{linkedUser.full_name ?? linkedUser.email}</div>
                            <div className="text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase tracking-widest opacity-70">{linkedUser.email}</div>
                        </div>
                    ) : employee.user_id ? (
                        <div className="flex items-center gap-3">
                            <div className="w-2 h-2 rounded-full bg-amber-400 animate-pulse" />
                            <div className="text-xs text-amber-600 dark:text-amber-400 font-bold uppercase tracking-widest italic">{t("common.loading")}</div>
                        </div>
                    ) : (
                        <div className="text-slate-300 dark:text-slate-600 font-bold text-xl tracking-tighter uppercase font-mono">—</div>
                    )}
                </div>
            </section>

            <section className="space-y-5">
                <div className="grid grid-cols-2 gap-6">
                    <div>
                        <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em] flex items-center gap-2 px-2 mb-5 opacity-70"><Calendar className="text-emerald-500" size={14} /> {t("employees.details.original_hire_date").toUpperCase()}</span>
                        <div className="bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] font-bold text-slate-800 dark:text-slate-100 text-2xl tracking-tighter shadow-inner uppercase font-mono">{employee.original_hire_date ? formatDate(employee.original_hire_date) : formatDate(employee.hire_date)}</div>
                    </div>
                    <div>
                        <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em] flex items-center gap-2 px-2 mb-5 opacity-70"><Calendar className="text-emerald-500" size={14} /> {t("employees.table.hire_date").toUpperCase()}</span>
                        <div className="bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] font-bold text-slate-800 dark:text-slate-100 text-2xl tracking-tighter shadow-inner uppercase font-mono">{formatDate(employee.hire_date)}</div>
                    </div>
                </div>
            </section>
        </div>
    );
}

// eslint-disable-next-line max-lines-per-function
export default function EmployeeDetailDrawer({
    isOpen,
    onClose,
    employee,
    wsId,
    departments,
    deptHistory,
    formatDate,
    onEdit,
    onTerminate,
    onRehire,
    onTransfer,
}: EmployeeDetailDrawerProps) {
    const { t } = useTranslation();
    const [activeDetailTab, setActiveDetailTab] = useState<DetailTabType>("profile");
    const scrollRef = useRef<HTMLDivElement>(null);

    // Arrow keys: left/right = switch tabs, up/down = scroll content
    useEffect(() => {
        if (!isOpen) return;
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
                e.preventDefault();
                setActiveDetailTab((previous) => {
                    const index = TAB_ORDER.indexOf(previous);
                    if (e.key === "ArrowRight") return TAB_ORDER[(index + 1) % TAB_ORDER.length];
                    return TAB_ORDER[(index - 1 + TAB_ORDER.length) % TAB_ORDER.length];
                });
            } else if (e.key === "ArrowUp" || e.key === "ArrowDown") {
                e.preventDefault();
                e.stopPropagation();
                const sheetScrollContainer = scrollRef.current?.closest('.overflow-y-auto');
                sheetScrollContainer?.scrollBy({
                    top: e.key === "ArrowDown" ? SCROLL_STEP : -SCROLL_STEP,
                    behavior: "smooth",
                });
            }
        };
        window.addEventListener("keydown", handleKeyDown, true);
        return () => { window.removeEventListener("keydown", handleKeyDown, true); };
    }, [isOpen]);

    const { data: members } = useQuery({
        queryKey: ["workspace-members", wsId],
        queryFn: () => fetchWorkspaceMembers(wsId),
        enabled: !!wsId && !!employee?.user_id,
    });

    if (!employee) return null;

    const linkedUser = members?.find((m: WorkspaceMember) => m.user_id === employee.user_id);

    return (
        <Sheet
            className="!max-w-4xl"
            contentClassName="!p-0"
            hideHeader
            isOpen={isOpen}
            onClose={onClose}
        >
            <DrawerHeader
                activeDetailTab={activeDetailTab}
                employee={employee}
                onClose={onClose}
                setActiveDetailTab={setActiveDetailTab}
                t={t}
            />

            <div className="p-10 pb-20 space-y-10" ref={scrollRef}>
                {activeDetailTab === "profile" && (
                    <ProfileTab
                        departments={departments}
                        deptHistory={deptHistory}
                        employee={employee}
                        formatDate={formatDate}
                        linkedUser={linkedUser}
                        onEdit={onEdit}
                        onRehire={onRehire}
                        onTerminate={onTerminate}
                        onTransfer={onTransfer}
                        t={t}
                    />
                )}

                {activeDetailTab === "contracts" && (
                    <EmployeeContractsTab
                        employeeId={employee.id}
                        formatDate={formatDate}
                        wsId={wsId}
                    />
                )}

                {activeDetailTab === "phases" && (
                    <EmployeePhasesTab
                        employeeId={employee.id}
                        formatDate={formatDate}
                        wsId={wsId}
                    />
                )}

                {activeDetailTab === "assignments" && (
                    <EmployeeAssignmentsTab
                        employeeId={employee.id}
                        formatDate={formatDate}
                        wsId={wsId}
                    />
                )}

                {activeDetailTab === "salary" && (
                    <div>
                        <EmployeeSalaryConfig employeeId={employee.id} wsId={wsId} />
                    </div>
                )}

                {activeDetailTab === "policies" && (
                    <div>
                        <WorkingConditionConfig employeeId={employee.id} />
                    </div>
                )}


            </div>
        </Sheet>
    );
}
