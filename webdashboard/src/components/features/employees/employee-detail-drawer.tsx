import React from "react";
import { useTranslation } from "react-i18next";

import type { Department, Employee, Employment, WorkspaceMember } from "@/lib/api";
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
    onRehire: (employee: Employee) => void;
    onTerminate: (employee: Employee) => void;
    onTransfer: (departmentId: string) => void;
    wsId: string;
    onEmployeeUpdate?: (emp: Employee) => void;
}

interface DrawerTabContentProps {
    activeDetailTab: DetailTabType;
    activeEmployment?: Employment;
    departments?: Department[];
    deptHistory?: DeptHistoryEntry[];
    employee: Employee;
    formatDate: (date: string) => string;
    linkedUser?: WorkspaceMember;
    locations?: WorkLocation[];
    onEdit: (e: Employee) => void;
    onRehire: (e: Employee) => void;
    onTerminate: (e: Employee) => void;
    onTransfer: (id: string) => void;
    t: (k: string) => string;
    wsId: string;
    onEmployeeUpdate?: (emp: Employee) => void;
}

const TAB_CONTENT_MAP: Record<DetailTabType, (props: DrawerTabContentProps) => React.ReactNode> = {
    assignments: (props) => <EmployeeAssignmentsTab employeeId={props.employee.id} formatDate={props.formatDate} wsId={props.wsId} />,
    compensation: (props) => <CompensationHistoryTab employeeId={props.employee.id} employmentId={props.activeEmployment?.id} formatDate={props.formatDate} wsId={props.wsId} />,
    contracts: (props) => <EmployeeContractsTab employeeId={props.employee.id} formatDate={props.formatDate} wsId={props.wsId} />,
    documents: (props) => <EmployeeDocumentsTab employeeId={props.employee.id} />,
    lifecycle: (props) => <EmployeeLifecycleTab employmentId={props.activeEmployment?.id} wsId={props.wsId} />,
    phases: (props) => <EmployeePhasesTab employeeId={props.employee.id} formatDate={props.formatDate} wsId={props.wsId} />,
    policies: (props) => <WorkingConditionConfig employeeId={props.employee.id} />,
    profile: (props) => (
        <ProfileTab
            departments={props.departments}
            deptHistory={props.deptHistory}
            employee={props.employee}
            formatDate={props.formatDate}
            linkedUser={props.linkedUser}
            locations={props.locations}
            onEdit={props.onEdit}
            onRehire={props.onRehire}
            onTerminate={props.onTerminate}
            onTransfer={props.onTransfer}
            t={props.t}
            wsId={props.wsId}
            onEmployeeUpdate={props.onEmployeeUpdate}
        />
    ),
    salary: (props) => <EmployeeSalaryConfig employeeId={props.employee.id} employmentId={props.activeEmployment?.id ?? ""} wsId={props.wsId} />,
};

function DrawerTabContent(props: DrawerTabContentProps) {
    const renderContent = TAB_CONTENT_MAP[props.activeDetailTab];
    return renderContent(props);
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
    onEmployeeUpdate,
}: EmployeeDetailDrawerProps) {
    const { t } = useTranslation();
    const {
        activeDetailTab,
        activeEmployment,
        linkedUser,
        locations,
        scrollRef,
        setActiveDetailTab,
    } = useEmployeeDetail({ employee, isOpen, wsId });

    if (!employee) return null;

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
                    activeEmployment={activeEmployment}
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
                    onEmployeeUpdate={onEmployeeUpdate}
                />
            </div>
        </Sheet>
    );
}
