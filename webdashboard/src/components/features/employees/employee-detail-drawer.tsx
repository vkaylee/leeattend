import { useQuery } from "@tanstack/react-query";
import React, { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import type { Department, Employee, WorkspaceMember, Employment } from "@/lib/api";
import { fetchWorkspaceMembers, fetchEmployeeEmployments } from "@/lib/api";
import { useLocations } from "@/pages/settings/hooks/use-locations";

import Sheet from "@/components/ui/sheet";
import { CustomFieldSection } from "@/components/custom-fields/custom-field-section";
import CompensationHistoryTab from "@/pages/employees/compensation-history-tab";
import EmployeeLifecycleTab from "@/pages/employees/lifecycle-tab";
import { EmployeeDocumentsTab } from "@/pages/employees/employee-documents-tab";
import WorkingConditionConfig from "@/components/employees/working-condition-config";
import EmployeeSalaryConfig from "@/components/payroll/employee-salary-config";

import EmployeeAssignmentsTab from "./employee-assignments-tab";
import EmployeeContractsTab from "./employee-contracts-tab";
import EmployeePhasesTab from "./employee-phases-tab";
import DrawerHeader, { DetailTabType } from "./drawer-header";
import ProfileTab from "./profile-tab";
import type { DeptHistoryEntry } from "./profile-sections";

const TAB_ORDER: DetailTabType[] = ["profile", "documents", "contracts", "phases", "assignments", "salary", "compensation", "policies", "lifecycle"];
const SCROLL_STEP = 80;

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

function DrawerTabContent({
    activeDetailTab,
    employee,
    wsId,
    departments,
    deptHistory,
    formatDate,
    onEdit,
    onRehire,
    onTerminate,
    onTransfer,
    linkedUser,
    locations,
    t,
    activeEmployment,
}: {
    activeDetailTab: DetailTabType;
    employee: Employee;
    wsId: string;
    departments?: Department[];
    deptHistory?: DeptHistoryEntry[];
    formatDate: (date: string) => string;
    onEdit: (e: Employee) => void;
    onRehire: (e: Employee) => void;
    onTerminate: (e: Employee) => void;
    onTransfer: (id: string) => void;
    linkedUser?: WorkspaceMember;
    locations?: any[];
    t: (k: string) => string;
    activeEmployment?: Employment;
}) {
    switch (activeDetailTab) {
        case "profile":
            return (
                <ProfileTab
                    departments={departments}
                    deptHistory={deptHistory}
                    employee={employee}
                    formatDate={formatDate}
                    linkedUser={linkedUser}
                    locations={locations}
                    onEdit={onEdit}
                    onRehire={onRehire}
                    onTerminate={onTerminate}
                    onTransfer={onTransfer}
                    t={t}
                    wsId={wsId}
                />
            );
        case "documents":
            return <EmployeeDocumentsTab employeeId={employee.id} />;
        case "contracts":
            return <EmployeeContractsTab employeeId={employee.id} formatDate={formatDate} wsId={wsId} />;
        case "phases":
            return <EmployeePhasesTab employeeId={employee.id} formatDate={formatDate} wsId={wsId} />;
        case "assignments":
            return <EmployeeAssignmentsTab employeeId={employee.id} formatDate={formatDate} wsId={wsId} />;
        case "salary":
            return <EmployeeSalaryConfig employeeId={employee.id} employmentId={activeEmployment?.id ?? ""} wsId={wsId} />;
        case "compensation":
            return <CompensationHistoryTab employeeId={employee.id} employmentId={activeEmployment?.id} wsId={wsId} formatDate={formatDate} />;
        case "policies":
            return <WorkingConditionConfig employeeId={employee.id} />;
        case "lifecycle":
            return <EmployeeLifecycleTab employmentId={activeEmployment?.id} wsId={wsId} />;
        default:
            return null;
    }
}

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

    const { data: locations } = useLocations(wsId);

    const { data: employments = [] } = useQuery({
        queryKey: ["employee-employments", wsId, employee?.id],
        queryFn: () => employee ? fetchEmployeeEmployments(wsId, employee.id) : Promise.resolve([]),
        enabled: !!wsId && !!employee?.id,
    });
    const activeEmployment = employments.find((e: Employment) => e.status === "active") || employments[0];

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
                <DrawerTabContent
                    activeDetailTab={activeDetailTab}
                    departments={departments}
                    deptHistory={deptHistory}
                    employee={employee}
                    formatDate={formatDate}
                    linkedUser={linkedUser}
                    locations={locations}
                    onEdit={onEdit}
                    onRehire={onRehire}
                    onTerminate={onTerminate}
                    onTransfer={onTransfer}
                    t={t}
                    wsId={wsId}
                    activeEmployment={activeEmployment}
                />
            </div>
        </Sheet>
    );
}
