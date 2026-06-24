import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Pencil, Trash2, UserPlus } from "lucide-react";
import React, { useState } from "react";

import type { Department, Employee, WorkspaceMember } from "@/lib/api";
import type { WorkLocation } from "@/lib/api/locations";

import PermissionGate from "@/components/common/permission-gate";
import { CustomFieldSection } from "@/components/custom-fields/custom-field-section";
import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { fetchEmployeeEmployments, updateEmployeeEmployment } from "@/lib/api";

import type { DeptHistoryEntry } from "./profile-sections";

import { DepartmentSection, LocationSection, UserSection, HireDatesSection } from "./profile-sections";

export function ProfileActions({
    employee,
    onEdit,
    onTerminate,
    onRehire,
    t,
    terminationDate,
    formatDate,
}: {
    employee: Employee;
    onEdit: (e: Employee) => void;
    onRehire: (e: Employee) => void;
    onTerminate: (e: Employee) => void;
    t: (k: string) => string;
    terminationDate?: string | null;
    formatDate: (date: string) => string;
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
                        <div className="flex items-center justify-center gap-2 py-3 px-4 bg-slate-50 dark:bg-slate-800 text-slate-600 dark:text-slate-300 font-bold rounded-2xl border border-slate-100 dark:border-slate-700/50 text-[10px] uppercase tracking-wider h-11 shadow-inner">
                            <span className="w-1.5 h-1.5 rounded-full bg-slate-400 dark:bg-slate-500 animate-pulse" />
                            {t("employees.details.terminated_at")}: {terminationDate ? formatDate(terminationDate) : "—"}
                        </div>
                        <Button className="flex items-center justify-center gap-2 py-2.5 bg-primary-600 hover:bg-primary-700 text-white font-bold rounded-2xl transition-all text-[10px] uppercase tracking-widest shadow-md active:scale-95 h-9 w-full" data-testid="employees-btn-rehire" onClick={() => { onRehire(employee); }} type="button" variant="primary"><UserPlus size={14} /> {t("employees.details.rehire").toUpperCase()}</Button>
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
    t
}: ProfileTabProps) {
    const queryClient = useQueryClient();
    const toast = useToast();
    const [isEditingLocation, setIsEditingLocation] = useState(false);
    const [selectedLocationIdState, setSelectedLocationIdState] = useState<string | null>(null);

    const { data: employments = [] } = useQuery({
        queryKey: ["employee-employments", wsId, employee.id],
        queryFn: () => fetchEmployeeEmployments(wsId, employee.id),
        enabled: !!wsId && !!employee.id,
    });
    const activeEmployment = employments.find(e => e.status === "active") || employments[0];
    const activeLocationId = activeEmployment?.primary_location_id ?? "";

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

    const updateLocationMutation = useMutation({
        mutationFn: (locationId: string) => {
            if (!activeEmployment) return Promise.reject(new Error("No active employment found"));
            return updateEmployeeEmployment(wsId, employee.id, activeEmployment.id, { primary_location_id: locationId });
        },
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: ["employee-employments", wsId, employee.id] });
            void queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
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
                onRehire={onRehire}
                onTerminate={onTerminate}
                t={t}
                terminationDate={!employee.active_employment_id ? activeEmployment?.end_date : null}
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

            {!!activeEmployment?.id && (
                <CustomFieldSection entityId={activeEmployment.id} entityType="employment" />
            )}
        </div>
    );
}
