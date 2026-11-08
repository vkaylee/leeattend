import React, { useState } from "react";
import { cn } from "@/lib/utils";
import { useTranslation } from "react-i18next";

import type { Department, Employee, Employment, OrganizationMember } from "@/lib/api";
import type { WorkLocation } from "@/lib/api/locations";

import WorkingConditionConfig from "@/components/employees/working-condition-config";
import EmployeeSalaryConfig from "@/components/payroll/employee-salary-config";
import Sheet from "@/components/ui/sheet";
import CompensationHistoryTab from "@/pages/employees/compensation-history-tab";
import { EmployeeDocumentsTab } from "@/pages/employees/employee-documents-tab";
import EmployeeLifecycleTab from "@/pages/employees/lifecycle-tab";

import type { DeptHistoryEntry } from "./profile-sections";

import DrawerHeader, { DetailTabType } from "./drawer-header";
import EmployeeAssignmentsTab from "./employee-assignments-tab";
import EmployeeAttendancePolicyTab from "./employee-attendance-policy-tab";
import EmployeeContractsTab from "./employee-contracts-tab";
import EmployeePhasesTab from "./employee-phases-tab";
import { useEmployeeDetail } from "./hooks/use-employee-detail";
import ProfileTab from "./profile-tab";

interface EmployeeDetailDrawerProps {
    departments?: Department[];
    deptHistory?: DeptHistoryEntry[];
    employee: Employee | null;
    formatDate: (date: string) => string;
    isOpen: boolean;
    onClose: () => void;
    onEdit: (employee: Employee) => void;
    onEmployeeUpdate?: (emp: Employee) => void;
    onRehire: (employee: Employee) => void;
    onTerminate: (employee: Employee) => void;
    onTransfer: (departmentId: string) => void;
}

export interface DrawerTabContentProps {
    activeDetailTab: DetailTabType;
    activeEmployment?: Employment;
    departments?: Department[];
    deptHistory?: DeptHistoryEntry[];
    employee: Employee;
    formatDate: (date: string) => string;
    linkedUser?: OrganizationMember;
    locations?: WorkLocation[];
    onEdit: (e: Employee) => void;
    onEmployeeUpdate?: (emp: Employee) => void;
    onRehire: (e: Employee) => void;
    onTerminate: (e: Employee) => void;
    onTransfer: (id: string) => void;
    t: (k: string) => string;
}

const TAB_CONTENT_MAP: Record<DetailTabType, (props: DrawerTabContentProps) => React.ReactNode> = {
    assignments: (props) => <EmployeeAssignmentsTab employeeId={props.employee.id} formatDate={props.formatDate}  />,
    compensation: (props) => <CompensationHistoryTab employeeId={props.employee.id} employmentId={props.activeEmployment?.id} formatDate={props.formatDate}  />,
    contracts: (props) => <EmployeeContractsTab employeeId={props.employee.id} formatDate={props.formatDate}  />,
    documents: (props) => <EmployeeDocumentsTab employeeId={props.employee.id} />,
    lifecycle: (props) => <EmployeeLifecycleTab employmentId={props.activeEmployment?.id}  />,
    phases: (props) => <EmployeePhasesTab employeeId={props.employee.id} formatDate={props.formatDate}  />,
    "working-conditions": (props) => <WorkingConditionConfig employeeId={props.employee.id} />,
    "attendance-policy": (props) => <EmployeeAttendancePolicyTab employeeId={props.employee.id}  />,
    profile: (props) => (
        <ProfileTab departments={props.departments}
        deptHistory={props.deptHistory}
        employee={props.employee}
        formatDate={props.formatDate}
        linkedUser={props.linkedUser}
        locations={props.locations}
        onEdit={props.onEdit}
        onEmployeeUpdate={props.onEmployeeUpdate}
        onRehire={props.onRehire}
        onTerminate={props.onTerminate}
        onTransfer={props.onTransfer}
        t={props.t}  />
    ),
    salary: (props) => <EmployeeSalaryConfig employeeId={props.employee.id} employmentId={props.activeEmployment?.id ?? ""}  />,
};

export function DrawerTabContent(props: DrawerTabContentProps) {
    const renderContent = TAB_CONTENT_MAP[props.activeDetailTab];
    return renderContent(props);
}

export default function EmployeeDetailDrawer({
    isOpen,
    onClose,
    employee,departments,
    deptHistory,
    formatDate,
    onEdit,
    onTerminate,
    onRehire,
    onTransfer,
    onEmployeeUpdate,
}: EmployeeDetailDrawerProps) {
    const { t } = useTranslation();
    const [isExpanded, setIsExpanded] = useState(false);
    const {
        activeDetailTab,
        activeEmployment,
        linkedUser,
        locations,
        scrollRef,
        setActiveDetailTab,
    } = useEmployeeDetail({ employee, isOpen});
    if (!employee) return null;

    return (
        <Sheet
            className={cn(
                "transition-all duration-300",
                isExpanded ? "!max-w-[96vw]" : "!max-w-5xl 2xl:!max-w-6xl"
            )}
            contentClassName="!p-0"
            hideHeader
            isOpen={isOpen}
            onClose={onClose}
        >
            <DrawerHeader
                activeDetailTab={activeDetailTab}
                employee={employee}
                isExpanded={isExpanded}
                onClose={onClose}
                onToggleExpand={() => { setIsExpanded((prev) => !prev); }}
                setActiveDetailTab={setActiveDetailTab}
                t={t}
            />

            <div
                aria-labelledby={`employees-tab-${activeDetailTab}`}
                className="px-6 py-6 sm:px-8 sm:py-8 pb-20 space-y-8 min-w-0 max-w-full overflow-x-hidden"
                id={`employees-tabpanel-${activeDetailTab}`}
                ref={scrollRef}
                role="tabpanel"
            >
                <DrawerTabContent activeDetailTab={activeDetailTab}
                activeEmployment={activeEmployment}
                departments={departments}
                deptHistory={deptHistory}
                employee={employee}
                formatDate={formatDate}
                linkedUser={linkedUser}
                locations={locations}
                onEdit={onEdit}
                onEmployeeUpdate={onEmployeeUpdate}
                onRehire={onRehire}
                onTerminate={onTerminate}
                onTransfer={onTransfer}
                t={t}  />
            </div>
        </Sheet>
    );
}
