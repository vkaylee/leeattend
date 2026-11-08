import { Building2, Calendar, MapPin, Check, Globe, Clock, User } from "lucide-react";
import React from "react";

import type { Department, Employee, OrganizationMember, Employment } from "@/lib/api";
import type { WorkLocation } from "@/lib/api/locations";

import PermissionGate from "@/components/common/permission-gate";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { maybeNull } from "@/types/api-overrides";

export interface DeptHistoryEntry {
    department_id: string;
    effective_date: string;
}

export function DepartmentSection({
    deptHistory,
    departments,
    onTransfer,
    formatDate,
    t
}: {
    deptHistory?: DeptHistoryEntry[];
    departments?: Department[];
    onTransfer: (id: string) => void;
    formatDate: (date: string) => string;
    t: (k: string) => string;
}) {
    return (
        <section className="space-y-5">
            <div className="flex justify-between items-center px-2">
                <span className="text-xs font-semibold text-slate-600 dark:text-slate-300 uppercase tracking-wider flex items-center gap-2"><Building2 className="text-primary-500" size={14} /> {t("employees.details.current_dept").toUpperCase()}</span>
                <PermissionGate permission="employees:update">
                    <Button className="text-xs font-semibold text-primary-600 dark:text-primary-400 uppercase tracking-wider hover:underline active:scale-90 h-auto shadow-none border-none p-0" onClick={() => { onTransfer(deptHistory?.[0]?.department_id ?? ""); }} type="button" variant="ghost">{t("employees.details.transfer").toUpperCase()}</Button>
                </PermissionGate>
            </div>
            <div className="bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] group hover:bg-white dark:hover:bg-slate-900 hover:shadow-xl hover:shadow-primary-100/10 transition-all shadow-inner relative overflow-hidden">
                {deptHistory && deptHistory.length > 0 ? (
                    <div>
                        <div className="font-bold text-slate-700 dark:text-slate-200 text-2xl tracking-tighter uppercase leading-none mb-3">{departments?.find(d => d.id === deptHistory[0].department_id)?.name}</div>
                        <div className="flex items-center gap-2 text-xs text-slate-600 dark:text-slate-400 font-medium uppercase tracking-wide"><Calendar className="text-primary-400" size={12} /> {t("employees.transfer.effective_date")}: {formatDate(deptHistory[0].effective_date)}</div>
                    </div>
                ) : (<div className="text-xs text-slate-500 dark:text-slate-400 font-bold uppercase tracking-widest italic opacity-60">{t("employees.details.not_assigned_dept")}</div>)}
                <Building2 className="absolute -right-8 -bottom-8 w-32 h-32 text-slate-700 dark:text-slate-200 opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
            </div>
        </section>
    );
}

function LocationDisplay({
    isEditingLocation,
    selectedLocationId,
    setSelectedLocationId,
    locations,
    currentLocation,
    setIsEditingLocation,
    t,
}: {
    isEditingLocation: boolean;
    selectedLocationId: string;
    setSelectedLocationId: (v: string) => void;
    locations?: WorkLocation[];
    currentLocation?: WorkLocation;
    setIsEditingLocation: (v: boolean) => void;
    t: (k: string) => string;
}) {
    if (isEditingLocation) {
        return (
            <div className="relative z-10">
                <Combobox
                    clearOnEscape
                    data-testid="profile-location-combobox"
                    onChange={setSelectedLocationId}
                    options={(locations ?? []).map((loc) => ({
                        value: loc.id,
                        label: loc.name,
                        description: loc.is_headquarters ? "Headquarters" : loc.timezone,
                    }))}
                    placeholder={t("employees.details.select_new_location")}
                    value={selectedLocationId}
                />
                <div className="mt-3 flex gap-2">
                    <Button className="text-xs uppercase tracking-wider rounded-lg px-4" onClick={() => { setIsEditingLocation(false); }} size="sm" type="button" variant="ghost">{t("common.cancel")}</Button>
                </div>
            </div>
        );
    }

    return (
        <div>
            <div className="font-bold text-slate-700 dark:text-slate-200 text-2xl tracking-tighter uppercase leading-none mb-3 flex items-center gap-2">
                {currentLocation?.name ?? t("employees.details.unknown_location")}
                {currentLocation?.is_headquarters ? (
                    <span className="bg-rose-100 text-rose-700 dark:bg-rose-900 dark:text-rose-300 text-[9px] px-2 py-0.5 rounded-full font-bold uppercase tracking-widest border border-rose-200 dark:border-rose-800">HQ</span>
                ) : null}
            </div>
            <div className="flex items-center gap-2 text-xs text-slate-600 dark:text-slate-400 font-medium uppercase tracking-wide group/tz cursor-help relative inline-flex">
                <Clock className="text-rose-400" size={12} /> {currentLocation?.timezone ?? t("employees.details.no_timezone")}
                <div className="absolute opacity-0 invisible group-hover/tz:opacity-100 group-hover/tz:visible transition-all bottom-full left-0 mb-2 whitespace-nowrap bg-slate-800 dark:bg-slate-100 text-white dark:text-slate-800 px-3 py-1.5 rounded-lg text-xs font-mono shadow-lg border border-slate-700 dark:border-slate-300">
                    Timezone set for {currentLocation?.name}
                </div>
            </div>
        </div>
    );
}

export function LocationSection({
    locations,
    isEditingLocation,
    setIsEditingLocation,
    selectedLocationId,
    setSelectedLocationId,
    updateLocationMutation,
    handleSaveLocation,
    currentLocation,
    activeLocationId,
    t
}: {
    employee: Employee;
    locations?: WorkLocation[];
    isEditingLocation: boolean;
    setIsEditingLocation: (v: boolean) => void;
    selectedLocationId: string;
    setSelectedLocationId: (v: string) => void;
    updateLocationMutation: { isPending: boolean };
    handleSaveLocation: () => void;
    currentLocation?: WorkLocation;
    activeLocationId: string;
    t: (k: string) => string;
}) {
    return (
        <section className="space-y-5">
            <div className="flex justify-between items-center px-2">
                <span className="text-xs font-semibold text-slate-600 dark:text-slate-300 uppercase tracking-wider flex items-center gap-2"><MapPin className="text-rose-500" size={14} /> {t("employees.details.work_location").toUpperCase()}</span>
                <PermissionGate permission="employees:update">
                    {!isEditingLocation ? (
                        <Button className="text-xs font-semibold text-primary-600 dark:text-primary-400 uppercase tracking-wider hover:underline active:scale-90 h-auto shadow-none border-none p-0" onClick={() => { setIsEditingLocation(true); setSelectedLocationId(activeLocationId); }} type="button" variant="ghost">{t("common.edit").toUpperCase()}</Button>
                    ) : (
                        <Button className="text-xs font-semibold text-emerald-600 dark:text-emerald-400 uppercase tracking-wider hover:underline active:scale-90 h-auto shadow-none border-none p-0 flex items-center gap-1" disabled={updateLocationMutation.isPending} onClick={handleSaveLocation} type="button" variant="ghost"><Check size={12}/> {t("common.save").toUpperCase()}</Button>
                    )}
                </PermissionGate>
            </div>
            <div className="bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] group hover:bg-white dark:hover:bg-slate-900 hover:shadow-xl hover:shadow-rose-100/10 transition-all shadow-inner relative overflow-hidden">
                <LocationDisplay
                    currentLocation={currentLocation}
                    isEditingLocation={isEditingLocation}
                    locations={locations}
                    selectedLocationId={selectedLocationId}
                    setIsEditingLocation={setIsEditingLocation}
                    setSelectedLocationId={setSelectedLocationId}
                    t={t}
                />
                <Globe className="absolute -right-8 -bottom-8 w-32 h-32 text-slate-700 dark:text-slate-200 opacity-5 group-hover:opacity-10 group-hover:scale-110 transition-all duration-700" />
            </div>
        </section>
    );
}

export function UserSection({
    linkedUser,
    employee,
    t
}: {
    linkedUser?: OrganizationMember;
    employee: Employee;
    t: (k: string) => string;
}) {
    return (
        <section className="space-y-5">
            <span className="text-xs font-semibold text-slate-600 dark:text-slate-300 uppercase tracking-wider flex items-center gap-2 px-2"><User className="text-primary-500" size={14} /> {t("common.user").toUpperCase()}</span>
            <div className="bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] group hover:bg-white dark:hover:bg-slate-900 hover:shadow-xl hover:shadow-primary-100/10 transition-all shadow-inner relative overflow-hidden">
                {linkedUser ? (
                    <div>
                        <div className="font-bold text-slate-700 dark:text-slate-200 text-lg tracking-tight uppercase leading-none mb-1">{linkedUser.full_name ?? linkedUser.email}</div>
                        <div className="text-xs text-slate-600 dark:text-slate-400 font-medium tracking-wide">{linkedUser.email}</div>
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
    );
}

export function HireDatesSection({
    employments,
    formatDate,
    t
}: {
    employments: Employment[];
    formatDate: (date: string) => string;
    t: (k: string) => string;
}) {
    const sortedEmployments = [...employments].sort((a, b) => a.start_date.localeCompare(b.start_date));
    const originalHireDate = sortedEmployments[0]?.start_date ?? "";
    const activeOrLatestEmployment = employments.find(e => e.status === "active") ?? sortedEmployments[sortedEmployments.length - 1];
    const hireDate = maybeNull(activeOrLatestEmployment)?.start_date ?? "";

    return (
        <section className="space-y-5">
            <div className="grid grid-cols-2 gap-6">
                <div>
                    <span className="text-xs font-semibold text-slate-600 dark:text-slate-300 uppercase tracking-wider flex items-center gap-2 px-2 mb-5"><Calendar className="text-emerald-500" size={14} /> {t("employees.details.original_hire_date").toUpperCase()}</span>
                    <div className="bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] font-bold text-slate-800 dark:text-slate-100 text-2xl tracking-tighter shadow-inner uppercase font-mono">{originalHireDate ? formatDate(originalHireDate) : "—"}</div>
                </div>
                <div>
                    <span className="text-xs font-semibold text-slate-600 dark:text-slate-300 uppercase tracking-wider flex items-center gap-2 px-2 mb-5"><Calendar className="text-emerald-500" size={14} /> {t("employees.table.hire_date").toUpperCase()}</span>
                    <div className="bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] font-bold text-slate-800 dark:text-slate-100 text-2xl tracking-tighter shadow-inner uppercase font-mono">{hireDate ? formatDate(hireDate) : "—"}</div>
                </div>
            </div>
        </section>
    );
}
