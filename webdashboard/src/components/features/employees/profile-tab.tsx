import React, { useEffect, useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Pencil, Trash2, UserPlus } from "lucide-react";
import PermissionGate from "@/components/common/permission-gate";
import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { CustomFieldSection } from "@/components/custom-fields/custom-field-section";
import type { Department, Employee, WorkspaceMember, Employment } from "@/lib/api";
import type { WorkLocation } from "@/lib/api-generated/models";
import { fetchEmployeeEmployments, updateEmployeeEmployment } from "@/lib/api";
import { DepartmentSection, LocationSection, UserSection, HireDatesSection } from "./profile-sections";
import type { DeptHistoryEntry } from "./profile-sections";

export function ProfileActions({
    employee,
    onEdit,
    onTerminate,
    onRehire,
    t
}: {
    employee: Employee;
    onEdit: (e: Employee) => void;
    onRehire: (e: Employee) => void;
    onTerminate: (e: Employee) => void;
    t: (k: string) => string;
}) {
    return (
        <div className="grid grid-cols-2 gap-6">
            <PermissionGate permission="employees:update">
                <Button className="flex items-center justify-center gap-3 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 font-bold rounded-2xl hover:bg-slate-50 dark:hover:bg-slate-800 hover:border-slate-300 dark:hover:border-slate-600 transition-all text-xs uppercase tracking-widest shadow-sm active:scale-95 h-auto" data-testid="employees-btn-edit" onClick={() => { onEdit(employee); }} type="button" variant="outline"><Pencil size={18} /> {t("employees.details.edit_profile").toUpperCase()}</Button>
            </PermissionGate>
            <PermissionGate permission="employees:update">
                {employee.active_employment_id ? (
                    <Button className="flex items-center justify-center gap-3 py-4 bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 font-bold rounded-2xl hover:bg-rose-100 dark:bg-rose-900 transition-all text-xs uppercase tracking-widest active:scale-95 shadow-sm shadow-rose-900/5 h-auto" data-testid="employees-btn-terminate" onClick={() => { onTerminate(employee); }} type="button" variant="destructive"><Trash2 size={18} /> {t("employees.details.terminate").toUpperCase()}</Button>
                ) : (
                    <Button className="flex items-center justify-center gap-3 py-4 bg-primary-600 text-white font-bold rounded-2xl hover:bg-primary-700 transition-all text-xs uppercase tracking-widest shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 h-auto" data-testid="employees-btn-rehire" onClick={() => { onRehire(employee); }} type="button" variant="primary"><UserPlus size={18} /> {t("employees.details.rehire").toUpperCase()}</Button>
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
    locations?: WorkLocation[];
    wsId: string;
    formatDate: (date: string) => string;
    linkedUser: WorkspaceMember | undefined;
    onEdit: (e: Employee) => void;
    onRehire: (e: Employee) => void;
    onTerminate: (e: Employee) => void;
    onTransfer: (id: string) => void;
    t: (k: string) => string;
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
                onEdit={onEdit}
                onRehire={onRehire}
                onTerminate={onTerminate}
                t={t}
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
