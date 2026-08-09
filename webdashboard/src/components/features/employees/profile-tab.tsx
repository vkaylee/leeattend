/* eslint-disable complexity -- multi-section profile tab with 5 defensive optional chains on API data */
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Pencil, RotateCcw, Trash2, UserPlus } from "lucide-react";
import React, { useState } from "react";

import type { Department, Employee, WorkspaceMember } from "@/lib/api";
import type { WorkLocation } from "@/lib/api/locations";

import PermissionGate from "@/components/common/permission-gate";
import { CustomFieldSection } from "@/components/custom-fields/custom-field-section";
import EditTerminationModal from "@/components/features/employees/edit-termination-modal";
import UndoTerminationModal from "@/components/features/employees/undo-termination-modal";
import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { useWorkspaceMutation } from "@/hooks/common/use-workspace-query";
import { fetchEmployeeEmployments, updateEmployeeEmployment } from "@/lib/api";
import { employeeKeys } from "@/lib/query-keys";
import { maybeNull } from "@/types/api-overrides";

import type { DeptHistoryEntry } from "./profile-sections";

import { DepartmentSection, LocationSection, UserSection, HireDatesSection } from "./profile-sections";

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
}: {
    employee: Employee;
    onEdit: (e: Employee) => void;
    onRehire: (e: Employee) => void;
    onTerminate: (e: Employee) => void;
    onUndoTerminate?: (e: Employee) => void;
    t: (k: string) => string;
    terminationDate?: string | null;
    formatDate: (date: string) => string;
    onEditTerminationDate?: () => void;
    showEditTerminationDate?: boolean;
}) {
    return (
        <div className="grid grid-cols-2 gap-6 items-stretch">
            <PermissionGate permission="employees:update">
                <Button className="flex items-center justify-center gap-3 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 font-bold rounded-2xl hover:bg-slate-50 dark:hover:bg-slate-800 hover:border-slate-300 dark:hover:border-slate-600 transition-all text-xs uppercase tracking-widest shadow-sm active:scale-95 h-full" data-testid="employees-btn-edit" onClick={() => { onEdit(employee); }} type="button" variant="outline"><Pencil size={18} /> {t("employees.details.edit_profile").toUpperCase()}</Button>
            </PermissionGate>
            <PermissionGate permission="employees:update">
                {employee.active_employment_id ? (
                    <Button className="flex items-center justify-center gap-3 py-4 bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 font-bold rounded-2xl hover:bg-rose-100 dark:bg-rose-900 transition-all text-xs uppercase tracking-widest active:scale-95 shadow-sm shadow-rose-900/5 h-auto" data-testid="employees-btn-terminate" onClick={() => { onTerminate(employee); }} type="button" variant="destructive"><Trash2 size={18} /> {t("employees.details.terminate").toUpperCase()}</Button>
                ) : (
                    <div className="flex flex-col gap-2 w-full justify-between h-full">
                        <div className="flex items-center justify-center gap-2 py-3 px-4 bg-slate-50 dark:bg-slate-800 text-slate-600 dark:text-slate-300 font-bold rounded-2xl border border-slate-100 dark:border-slate-700/50 text-[10px] uppercase tracking-wider h-11 shadow-inner relative group">
                            <span className="w-1.5 h-1.5 rounded-full bg-slate-400 dark:bg-slate-500 animate-pulse" />
                            {t("employees.details.terminated_at")}: {terminationDate ? formatDate(terminationDate) : "—"}
                            {!!showEditTerminationDate && !!onEditTerminationDate && (
                                <Button
                                    className="ml-2 !h-auto !min-w-0 !p-1 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors cursor-pointer"
                                    data-testid="edit-termination-date-btn"
                                    onClick={onEditTerminationDate}
                                    title={t("employees.terminate.edit_date_title")}
                                    type="button"
                                    variant="ghost"
                                >
                                    <Pencil size={12} />
                                </Button>
                              )}
                        </div>
                        <div className="flex gap-4 w-full">
                            <Button className="flex items-center justify-center gap-2 py-2.5 bg-primary-600 hover:bg-primary-700 text-white font-bold rounded-2xl transition-all text-[10px] uppercase tracking-widest shadow-md active:scale-95 h-9 flex-1" data-testid="employees-btn-rehire" onClick={() => { onRehire(employee); }} type="button" variant="primary"><UserPlus size={14} /> {t("employees.details.rehire").toUpperCase()}</Button>
                            <Button className="flex items-center justify-center gap-2 py-2.5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 font-bold rounded-2xl hover:bg-slate-50 dark:hover:bg-slate-800 transition-all text-[10px] uppercase tracking-widest shadow-sm active:scale-95 h-9 flex-1" data-testid="employees-btn-undo-terminate" onClick={() => { onUndoTerminate?.(employee); }} type="button" variant="outline"><RotateCcw size={14} /> {t("employees.details.undo_terminate").toUpperCase()}</Button>
                        </div>
                    </div>
                )}
            </PermissionGate>
        </div>
    );
}

function GridWrap({ children }: { children: React.ReactNode }) {
    return <div className="w-full">{children}</div>;
}

export interface ProfileTabProps {
    departments?: Department[];
    deptHistory?: DeptHistoryEntry[];
    employee: Employee;
    formatDate: (date: string) => string;
    linkedUser: WorkspaceMember | undefined;
    locations?: WorkLocation[];
    onEdit: (e: Employee) => void;
    onEmployeeUpdate?: (emp: Employee) => void;
    onRehire: (e: Employee) => void;
    onTerminate: (e: Employee) => void;
    onTransfer: (id: string) => void;
    t: (k: string) => string;
    wsId: string;
}

export default function ProfileTab({
    departments,
    deptHistory,
    employee,
    locations,
    wsId,
    formatDate,
    linkedUser,
    onEdit,
    onRehire,
    onTerminate,
    onTransfer,
    t,
    onEmployeeUpdate
}: ProfileTabProps) {
    const queryClient = useQueryClient();
    const toast = useToast();
    const [isEditingLocation, setIsEditingLocation] = useState(false);
    const [selectedLocationIdState, setSelectedLocationIdState] = useState<string | null>(null);
    const [isEditTerminationModalOpen, setIsEditTerminationModalOpen] = useState(false);
    const [isUndoTerminationModalOpen, setIsUndoTerminationModalOpen] = useState(false);

    const { data: employments = [] } = useQuery({
        queryKey: ["employee-employments", wsId, employee.id],
        queryFn: () => fetchEmployeeEmployments(wsId, employee.id),
        enabled: !!wsId && !!employee.id,
    });
    const activeEmployment = employments.find(e => e.status === "active") ?? employments[0];
    const terminatedEmployment = employments.find(e => e.status === "terminated");
    const activeLocationId = maybeNull(activeEmployment)?.primary_location_id ?? "";

    const selectedLocationId = selectedLocationIdState ?? activeLocationId;

    const handleSetIsEditingLocation = (editing: boolean) => {
        setIsEditingLocation(editing);
        if (!editing) {
            setSelectedLocationIdState(null);
        }
    };

    const handleSetSelectedLocationId = (id: string) => {
        setSelectedLocationIdState(id);
    };

    const updateLocationMutation = useWorkspaceMutation<
        string,
        Awaited<ReturnType<typeof updateEmployeeEmployment>>
    >({
        workspaceId: wsId,
        keys: [employeeKeys.all(wsId)],
        mutationFn: (workspaceId, locationId) => {
            if (!maybeNull(activeEmployment)) return Promise.reject(new Error("No active employment found"));
            return updateEmployeeEmployment(workspaceId, employee.id, activeEmployment.id, { primary_location_id: locationId });
        },
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: ["employee-employments", wsId, employee.id] });
            handleSetIsEditingLocation(false);
            toast.success(t("common.success"));
        },
        onError: () => {
            toast.error("Failed to update location");
            setSelectedLocationIdState(null);
        }
    });

    const handleSaveLocation = () => {
        if (selectedLocationId !== activeLocationId) {
            updateLocationMutation.mutate(selectedLocationId);
        } else {
            handleSetIsEditingLocation(false);
        }
    };

    const currentLocation = locations?.find(l => l.id === activeLocationId);

    return (
        <div className="space-y-10">
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

            <DepartmentSection
                departments={departments}
                deptHistory={deptHistory}
                formatDate={formatDate}
                onTransfer={onTransfer}
                t={t}
            />

            <LocationSection
                activeLocationId={activeLocationId}
                currentLocation={currentLocation}
                employee={employee}
                handleSaveLocation={handleSaveLocation}
                isEditingLocation={isEditingLocation}
                locations={locations}
                selectedLocationId={selectedLocationId}
                setIsEditingLocation={handleSetIsEditingLocation}
                setSelectedLocationId={handleSetSelectedLocationId}
                t={t}
                updateLocationMutation={updateLocationMutation}
            />

            <UserSection
                employee={employee}
                linkedUser={linkedUser}
                t={t}
            />

            <GridWrap>
                <HireDatesSection
                    employments={employments}
                    formatDate={formatDate}
                    t={t}
                />
            </GridWrap>

            <CustomFieldSection entityId={employee.id} entityType="employee" />

            {!!maybeNull(activeEmployment)?.id && (
                <CustomFieldSection entityId={activeEmployment.id} entityType="employment" />
            )}

            {isEditTerminationModalOpen && terminatedEmployment ? (
                <EditTerminationModal
                    currentEndDate={terminatedEmployment.end_date ?? null}
                    employeeId={employee.id}
                    employmentId={terminatedEmployment.id}
                    isOpen={isEditTerminationModalOpen}
                    onClose={() => { setIsEditTerminationModalOpen(false); }}
                    wsId={wsId}
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
                            onEmployeeUpdate({
                                ...employee,
                                active_employment_id: terminatedEmployment.id,
                            });
                        }
                    }}
                    wsId={wsId}
                />
              ) : null}
        </div>
    );
}
