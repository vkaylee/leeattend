import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Pencil, RotateCcw, Trash2, UserPlus } from "lucide-react";
import { useState } from "react";
import type { ReactNode } from "react";

import type { Department, Employee, Employment, OrganizationMember } from "@/lib/api";
import type { WorkLocation } from "@/lib/api/locations";

import PermissionGate from "@/components/common/permission-gate";
import { CustomFieldSection } from "@/components/custom-fields/custom-field-section";
import EditTerminationModal from "@/components/features/employees/edit-termination-modal";
import UndoTerminationModal from "@/components/features/employees/undo-termination-modal";
import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { useOrganizationMutation } from "@/hooks/common/use-organization-query";
import { fetchEmployeeEmployments, updateEmployeeEmployment } from "@/lib/api";
import { createQueryKey, employeeKeys } from "@/lib/query-keys";
import { maybeNull } from "@/types/api-overrides";

import { DepartmentSection, HireDatesSection, LocationSection, UserSection } from "./profile-sections";
import type { DeptHistoryEntry } from "./profile-sections";

interface ProfileActionsProps {
    employee: Employee;
    onEdit: (employee: Employee) => void;
    onTerminate: (employee: Employee) => void;
    onRehire: (employee: Employee) => void;
    onUndoTerminate?: (employee: Employee) => void;
    t: (key: string) => string;
    terminationDate?: string | null;
    formatDate: (date: string) => string;
    onEditTerminationDate?: () => void;
    showEditTerminationDate?: boolean;
}

export function ProfileActions({
    employee,
    onEdit,
    onTerminate,
    onRehire,
    onUndoTerminate,
    t,
    terminationDate,
    formatDate,
    onEditTerminationDate,
    showEditTerminationDate,
}: ProfileActionsProps) {
    return (
        <div className="grid grid-cols-2 items-stretch gap-6">
            <PermissionGate permission="employees:update">
                <Button className="flex h-full items-center justify-center gap-3 rounded-2xl border border-slate-200 bg-white py-4 text-xs font-bold uppercase tracking-widest text-slate-700 shadow-sm transition-all hover:border-slate-300 hover:bg-slate-50 active:scale-95 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-200 dark:hover:border-slate-600 dark:hover:bg-slate-800" data-testid="employees-btn-edit" onClick={() => { onEdit(employee); }} type="button" variant="outline">
                    <Pencil size={18} /> {t("employees.details.edit_profile").toUpperCase()}
                </Button>
            </PermissionGate>
            <PermissionGate permission="employees:update">
                {employee.active_employment_id ? (
                    <Button className="flex h-full items-center justify-center gap-3 rounded-2xl bg-rose-50 py-4 text-xs font-bold uppercase tracking-widest text-rose-600 shadow-sm shadow-rose-900/5 transition-all hover:bg-rose-100 active:scale-95 dark:bg-rose-950 dark:text-rose-300 dark:hover:bg-rose-900" data-testid="employees-btn-terminate" onClick={() => { onTerminate(employee); }} type="button" variant="destructive">
                        <Trash2 size={18} /> {t("employees.details.terminate").toUpperCase()}
                    </Button>
                ) : (
                    <div className="flex h-full w-full flex-col justify-between gap-2">
                        <div className="relative flex h-11 items-center justify-center gap-2 rounded-2xl border border-slate-100 bg-slate-50 px-4 py-3 text-xs font-semibold uppercase tracking-wider text-slate-700 shadow-inner dark:border-slate-700/50 dark:bg-slate-800 dark:text-slate-200">
                            <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-slate-400 dark:bg-slate-500" />
                            {t("employees.details.terminated_at")}: {terminationDate ? formatDate(terminationDate) : "—"}
                            {showEditTerminationDate && onEditTerminationDate ? (
                                <Button aria-label={t("employees.terminate.edit_date_title")} className="ml-2 !h-auto !min-w-0 !p-1 text-slate-400 transition-colors hover:text-slate-600 dark:hover:text-slate-200" data-testid="edit-termination-date-btn" onClick={onEditTerminationDate} title={t("employees.terminate.edit_date_title")} type="button" variant="ghost">
                                    <Pencil size={12} />
                                </Button>
                            ) : null}
                        </div>
                        <div className="flex w-full gap-4">
                            <Button className="flex h-9 flex-1 items-center justify-center gap-2 rounded-2xl bg-primary-600 py-2.5 text-xs font-bold uppercase tracking-wider text-white shadow-md transition-all hover:bg-primary-700 active:scale-95" data-testid="employees-btn-rehire" onClick={() => { onRehire(employee); }} type="button" variant="primary">
                                <UserPlus size={14} /> {t("employees.details.rehire").toUpperCase()}
                            </Button>
                            <Button className="flex h-9 flex-1 items-center justify-center gap-2 rounded-2xl border border-slate-200 bg-white py-2.5 text-xs font-bold uppercase tracking-wider text-slate-700 shadow-sm transition-all hover:bg-slate-50 active:scale-95 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-200 dark:hover:bg-slate-800" data-testid="employees-btn-undo-terminate" onClick={() => { onUndoTerminate?.(employee); }} type="button" variant="outline">
                                <RotateCcw size={14} /> {t("employees.details.undo_terminate").toUpperCase()}
                            </Button>
                        </div>
                    </div>
                )}
            </PermissionGate>
        </div>
    );
}

function GridWrap({ children }: { children: ReactNode }) {
    return <div className="w-full">{children}</div>;
}

interface LocationEditorProps {
    employee: Employee;
    employments: Employment[];
    locations?: WorkLocation[];
    t: (key: string) => string;
}

function LocationEditor({ employee, employments, locations, t }: LocationEditorProps) {
    const queryClient = useQueryClient();
    const toast = useToast();
    const [isEditingLocation, setIsEditingLocation] = useState(false);
    const [selectedLocationIdState, setSelectedLocationIdState] = useState<string | null>(null);
    const activeEmployment = employments.find((employment) => employment.status === "active") ?? employments[0];
    const activeLocationId = maybeNull(activeEmployment)?.primary_location_id ?? "";
    const selectedLocationId = selectedLocationIdState ?? activeLocationId;
    const currentLocation = locations?.find((location) => location.id === activeLocationId);

    const setIsEditing = (editing: boolean) => {
        setIsEditingLocation(editing);
        if (!editing) setSelectedLocationIdState(null);
    };

    const updateLocationMutation = useOrganizationMutation<string, Awaited<ReturnType<typeof updateEmployeeEmployment>>>({
        keys: [employeeKeys.all()],
        mutationFn: (locationId) => {
            if (!maybeNull(activeEmployment)) return Promise.reject(new Error("No active employment found"));
            return updateEmployeeEmployment(employee.id, activeEmployment.id, { primary_location_id: locationId });
        },
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: createQueryKey("employee-employments", employee.id) });
            setIsEditing(false);
            toast.success(t("common.success"));
        },
        onError: () => {
            toast.error("Failed to update location");
            setSelectedLocationIdState(null);
        },
    });

    const saveLocation = () => {
        if (selectedLocationId !== activeLocationId) {
            updateLocationMutation.mutate(selectedLocationId);
            return;
        }
        setIsEditing(false);
    };

    return (
        <LocationSection
            activeLocationId={activeLocationId}
            currentLocation={currentLocation}
            employee={employee}
            handleSaveLocation={saveLocation}
            isEditingLocation={isEditingLocation}
            locations={locations}
            selectedLocationId={selectedLocationId}
            setIsEditingLocation={setIsEditing}
            setSelectedLocationId={setSelectedLocationIdState}
            t={t}
            updateLocationMutation={updateLocationMutation}
        />
    );
}

interface ProfileTerminationProps {
    activeEmployment?: Employment;
    employee: Employee;
    formatDate: (date: string) => string;
    onEdit: (employee: Employee) => void;
    onEmployeeUpdate?: (employee: Employee) => void;
    onRehire: (employee: Employee) => void;
    onTerminate: (employee: Employee) => void;
    terminatedEmployment?: Employment;
    t: (key: string) => string;
}

function ProfileTermination({
    activeEmployment,
    employee,
    formatDate,
    onEdit,
    onEmployeeUpdate,
    onRehire,
    onTerminate,
    terminatedEmployment,
    t,
}: ProfileTerminationProps) {
    const [isEditTerminationModalOpen, setIsEditTerminationModalOpen] = useState(false);
    const [isUndoTerminationModalOpen, setIsUndoTerminationModalOpen] = useState(false);

    return (
        <>
            <ProfileActions
                employee={employee}
                formatDate={formatDate}
                onEdit={onEdit}
                onEditTerminationDate={() => { setIsEditTerminationModalOpen(true); }}
                onRehire={onRehire}
                onTerminate={onTerminate}
                onUndoTerminate={() => { setIsUndoTerminationModalOpen(true); }}
                showEditTerminationDate={!employee.active_employment_id && !!terminatedEmployment}
                t={t}
                terminationDate={!employee.active_employment_id ? maybeNull(activeEmployment)?.end_date : null}
            />
            {isEditTerminationModalOpen && terminatedEmployment ? (
                <EditTerminationModal
                    currentEndDate={terminatedEmployment.end_date ?? null}
                    employeeId={employee.id}
                    employmentId={terminatedEmployment.id}
                    isOpen={isEditTerminationModalOpen}
                    onClose={() => { setIsEditTerminationModalOpen(false); }}
                />
            ) : null}
            {isUndoTerminationModalOpen && terminatedEmployment ? (
                <UndoTerminationModal
                    employeeId={employee.id}
                    employmentId={terminatedEmployment.id}
                    isOpen={isUndoTerminationModalOpen}
                    onClose={() => { setIsUndoTerminationModalOpen(false); }}
                    onSuccess={() => {
                        if (onEmployeeUpdate && maybeNull(terminatedEmployment)) {
                            onEmployeeUpdate({ ...employee, active_employment_id: terminatedEmployment.id });
                        }
                    }}
                />
            ) : null}
        </>
    );
}

export interface ProfileTabProps {
    departments?: Department[];
    deptHistory?: DeptHistoryEntry[];
    employee: Employee;
    formatDate: (date: string) => string;
    linkedUser: OrganizationMember | undefined;
    locations?: WorkLocation[];
    onEdit: (employee: Employee) => void;
    onEmployeeUpdate?: (employee: Employee) => void;
    onRehire: (employee: Employee) => void;
    onTerminate: (employee: Employee) => void;
    onTransfer: (id: string) => void;
    t: (key: string) => string;
}

export default function ProfileTab({
    departments,
    deptHistory,
    employee,
    locations,
    formatDate,
    linkedUser,
    onEdit,
    onEmployeeUpdate,
    onRehire,
    onTerminate,
    onTransfer,
    t,
}: ProfileTabProps) {
    const { data: employments = [] } = useQuery({
        queryKey: createQueryKey("employee-employments", employee.id),
        queryFn: () => fetchEmployeeEmployments(employee.id),
        enabled: !!employee.id,
    });
    const activeEmployment = employments.find((employment) => employment.status === "active") ?? employments[0];
    const terminatedEmployment = employments.find((employment) => employment.status === "terminated");

    return (
        <div className="space-y-10">
            <ProfileTermination
                activeEmployment={activeEmployment}
                employee={employee}
                formatDate={formatDate}
                onEdit={onEdit}
                onEmployeeUpdate={onEmployeeUpdate}
                onRehire={onRehire}
                onTerminate={onTerminate}
                terminatedEmployment={terminatedEmployment}
                t={t}
            />
            <DepartmentSection departments={departments} deptHistory={deptHistory} formatDate={formatDate} onTransfer={onTransfer} t={t} />
            <LocationEditor employee={employee} employments={employments} locations={locations} t={t} />
            <UserSection employee={employee} linkedUser={linkedUser} t={t} />
            <GridWrap><HireDatesSection employments={employments} formatDate={formatDate} t={t} /></GridWrap>
            <CustomFieldSection entityId={employee.id} entityType="employee" />
            {maybeNull(activeEmployment)?.id ? <CustomFieldSection entityId={activeEmployment.id} entityType="employment" /> : null}
        </div>
    );
}
