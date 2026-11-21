import React from "react";

import type { Department, Employee, Employment, OrganizationMember } from "@/lib/api";
import type { WorkLocation } from "@/lib/api/locations";

import EmployeeSalaryConfig from "@/components/payroll/employee-salary-config";
import CompensationHistoryTab from "@/pages/employees/compensation-history-tab";
import { EmployeeDocumentsTab } from "@/pages/employees/employee-documents-tab";

import type { DeptHistoryEntry } from "./profile-sections";

import { DetailTabType } from "./drawer-header";
import EmployeeContractsTab from "./employee-contracts-tab";
import ProfileTab from "./profile-tab";

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
    compensation: (props) => <CompensationHistoryTab employeeId={props.employee.id} employmentId={props.activeEmployment?.id} formatDate={props.formatDate}  />,
    contracts: (props) => <EmployeeContractsTab employeeId={props.employee.id} formatDate={props.formatDate}  />,
    documents: (props) => <EmployeeDocumentsTab employeeId={props.employee.id} />,
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
