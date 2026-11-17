import type { TFunction } from "i18next";
import type { RefObject } from "react";

import { Clock, Plus, ShieldCheck, Upload } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import type { LeaveStatusFilter, TabType } from "@/components/features/leave/use-leave-requests";
import type { Employee, LeaveRequest } from "@/lib/api";

import { EmployeeSearch } from "@/components/common/employee-search";
import FloatingRecordBadge from "@/components/common/floating-record-badge";
import InfiniteScrollSentinel from "@/components/common/infinite-scroll-sentinel";
import { ResilientWidget } from "@/components/common/resilient-widget";
import { CreateLeaveRequestModal } from "@/components/features/leave/create-leave-request-modal";
import { LeaveDetailSheet } from "@/components/features/leave/leave-detail-sheet";
import { LeaveImportBanner } from "@/components/features/leave/leave-import-banner";
import { LeaveRequestMobileCard } from "@/components/features/leave/leave-request-mobile-card";
import { LeaveRequestRow } from "@/components/features/leave/leave-request-row";
import { LeaveStatusFilterHeader } from "@/components/features/leave/leave-status-filter-header";
import { useLeaveRequests } from "@/components/features/leave/use-leave-requests";
import { PDFPreviewModal } from "@/components/files/pdf-preview-modal";
import ApprovalListLayout from "@/components/templates/approval-list-layout";
import Button from "@/components/ui/button";
import { FilterPanel } from "@/components/ui/filter-panel";
import { FilterTrigger } from "@/components/ui/filter-trigger";
import { PageContainer } from "@/components/ui/page-container";
import { PageHeader } from "@/components/ui/page-header";
import { PageTab, PageTabs } from "@/components/ui/page-tabs";
import ResponsiveTable from "@/components/ui/responsive-table";
import ResponsiveToolbar from "@/components/ui/responsive-toolbar";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useDepartments } from "@/hooks/use-departments";
import { useMyPermissions } from "@/hooks/use-permissions";

import { BehalfAssignmentsTab } from "./approvals/components/behalf-assignments-tab";

interface StatusFilterDropdownProps {
    setStatusFilter: (status: LeaveStatusFilter) => void;
    statusFilter: LeaveStatusFilter;
}

function StatusFilterDropdown({ setStatusFilter, statusFilter }: StatusFilterDropdownProps) {
    const { t } = useTranslation();

    return (
        <div className="space-y-2">
            <label className="text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">{t("common.status")}</label>
            <FilterPanel
                container="dropdown"
                mode="single"
                onClear={() => { setStatusFilter("all"); }}
                onToggle={(value) => { setStatusFilter(value as LeaveStatusFilter); }}
                options={[
                    { value: "all", label: t("common.all", "All").toUpperCase() },
                    { value: "pending", label: t("common.status_pending", "Pending").toUpperCase() },
                    { value: "approved", label: t("common.status_approved", "Approved").toUpperCase() },
                    { value: "pre_approved", label: t("common.status_pre_approved", "Pre-approved").toUpperCase() },
                    { value: "rejected", label: t("common.status_rejected", "Rejected").toUpperCase() },
                ]}
                selected={statusFilter !== "all" ? [statusFilter] : []}
                trigger={<FilterTrigger chevron hasActive={statusFilter !== "all"} label={t(`common.status_${statusFilter}`).toUpperCase()} variant="pill-button" />}
            />
        </div>
    );
}

interface LeavePageActionsProps {
    onCreate: () => void;
    onImport: () => void;
    t: TFunction;
}

function LeavePageActions({ onImport, onCreate, t }: LeavePageActionsProps) {
    return (
        <div className="flex w-full flex-col gap-3 sm:w-auto sm:flex-row">
            <Button className="h-10 w-full rounded-xl text-xs font-bold uppercase tracking-wider shadow-sm sm:w-auto" onClick={onImport} variant="outline">
                <Upload className="mr-2 h-4 w-4" />{t("leave_requests.import_excel")}
            </Button>
            <Button className="h-10 w-full rounded-xl text-xs font-bold uppercase tracking-wider shadow-md sm:w-auto" onClick={onCreate} variant="primary">
                <Plus className="mr-2 h-4 w-4" />{t("leave_requests.add")}
            </Button>
        </div>
    );
}

interface LeavePageNavigationProps {
    activeTab: TabType;
    canManageBehalf: boolean;
    departments: ReturnType<typeof useDepartments>["data"];
    employeeId: string;
    setActiveTab: (tab: TabType) => void;
    setEmployeeId: (employeeId: string) => void;
    t: TFunction;
}

function LeavePageNavigation({ activeTab, canManageBehalf, employeeId, departments, setActiveTab, setEmployeeId, t }: LeavePageNavigationProps) {
    const tabs: PageTab[] = [
        { id: "requests", label: t("leave_requests.title", "Danh sách đơn").toUpperCase(), icon: <Clock size={14} />, colorScheme: "primary" },
        { id: "behalf_assignments", label: t("leave_requests.tabs.behalf_assignments", "Behalf Assignments").toUpperCase(), icon: <ShieldCheck size={14} />, colorScheme: "violet", hidden: !canManageBehalf },
    ];

    return (
        <div className="flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
            <PageTabs activeTab={activeTab} onTabChange={(id) => { setActiveTab(id as TabType); }} tabs={tabs} />
            {activeTab !== "behalf_assignments" ? (
                <div className="w-full sm:w-72">
                    <EmployeeSearch
                        clearOnEscape
                        departments={departments ?? []}
                        keyboardShortcut={["/", "Enter"]}
                        onChange={(value) => { setEmployeeId((Array.isArray(value) ? value[0] : value) || "all"); }}
                        placeholder={employeeId === "all" ? t("common.all_employees") : undefined}
                        value={employeeId === "all" ? "" : employeeId}
                    />
                </div>
            ) : null}
        </div>
    );
}

interface LeaveRequestListProps {
    activeTab: TabType;
    employeeMap: Map<string, Employee>;
    formatDate: (date: string) => string;
    hasNextPage: boolean;
    isFetchingNextPage: boolean;
    leaveTypes: ReturnType<typeof useLeaveRequests>["leaveTypes"];
    loadMoreRef: RefObject<HTMLTableRowElement | null>;
    onApprove: (id: string, status: string) => void;
    onPreview: (url: string, title: string) => void;
    onSelect: (request: LeaveRequest) => void;
    requests: LeaveRequest[];
    requestsLoading: boolean;
    setActiveTab: (tab: TabType) => void;
    setStatusFilter: (status: LeaveStatusFilter) => void;
    statusFilter: LeaveStatusFilter;
    t: TFunction;
}

function LeaveRequestList({
    employeeMap,
    formatDate,
    hasNextPage,
    isFetchingNextPage,
    leaveTypes,
    loadMoreRef,
    onApprove,
    onPreview,
    onSelect,
    requests,
    requestsLoading,
    activeTab,
    setActiveTab,
    setStatusFilter,
    statusFilter,
    t,
}: LeaveRequestListProps) {
    const tableHeaders = [
        t("common.employee"),
        t("common.type"),
        t("leave_requests.table.details"),
        <LeaveStatusFilterHeader key="status-header" setStatusFilter={setStatusFilter} statusFilter={statusFilter} />,
        "",
    ];

    return (
        <div className="space-y-6">
            <ResponsiveToolbar filterCount={statusFilter !== "all" ? 1 : 0} mobileOnly>
                <StatusFilterDropdown setStatusFilter={setStatusFilter} statusFilter={statusFilter} />
            </ResponsiveToolbar>
            <ResponsiveTable<LeaveRequest>
                data={requests}
                emptyState={{ title: t("leave_requests.messages.no_requests"), icon: <Clock className="h-10 w-10" /> }}
                getCardKey={(request) => request.id}
                isLoading={requestsLoading}
                renderCard={(request) => (
                    <LeaveRequestMobileCard emp={employeeMap.get(request.employment_id)} formatDate={formatDate} leaveTypes={leaveTypes} onApprove={onApprove} onPreviewAttachment={onPreview} request={request} t={t} />
                )}
                renderDesktopTable={() => (
                    <ApprovalListLayout activeTab={activeTab} emptyMessage={t("leave_requests.messages.no_requests")} hideHeader hideTabs isEmpty={requests.length === 0} isLoading={requestsLoading} onTabChange={(id) => { setActiveTab(id as TabType); }} tableHeaders={tableHeaders}>
                        {requests.map((request) => (
                            <LeaveRequestRow emp={employeeMap.get(request.employment_id)} formatDate={formatDate} key={request.id} leaveTypes={leaveTypes} onApprove={onApprove} onPreviewAttachment={onPreview} onSelect={onSelect} request={request} t={t} />
                        ))}
                        {hasNextPage ? <InfiniteScrollSentinel colSpan={tableHeaders.length} isFetchingNextPage={isFetchingNextPage} sentinelRef={loadMoreRef} /> : null}
                    </ApprovalListLayout>
                )}
            />
        </div>
    );
}

interface LeaveRequestOverlaysProps {
    employeeMap: Map<string, Employee>;
    formatDate: (date: string) => string;
    isModalOpen: boolean;
    isPreviewOpen: boolean;
    leaveTypes: ReturnType<typeof useLeaveRequests>["leaveTypes"];
    onApprove: (id: string, status: string) => void;
    onCloseDetails: () => void;
    onCloseModal: () => void;
    onClosePreview: () => void;
    previewFileUrl: string | null;
    previewTitle: string;
    selectedRequest: LeaveRequest | null;
    t: TFunction;
}

function LeaveRequestOverlays({ employeeMap, formatDate, isModalOpen, leaveTypes, onApprove, onCloseDetails, onCloseModal, onClosePreview, previewFileUrl, previewTitle, selectedRequest, t, isPreviewOpen }: LeaveRequestOverlaysProps) {
    return (
        <>
            <LeaveDetailSheet emp={selectedRequest ? employeeMap.get(selectedRequest.employment_id) : undefined} formatDate={formatDate} isOpen={Boolean(selectedRequest)} leaveTypes={leaveTypes} onApprove={onApprove} onClose={onCloseDetails} request={selectedRequest} />
            <CreateLeaveRequestModal isOpen={isModalOpen} leaveTypes={leaveTypes} onClose={onCloseModal} t={t} />
            <PDFPreviewModal fileUrl={previewFileUrl} isOpen={isPreviewOpen} onClose={onClosePreview} title={previewTitle} />
        </>
    );
}

export default function LeaveRequestsPage() {
    const { formatDate } = useDateTimeFormatter();
    const { hasPermission } = useMyPermissions();
    const [previewFileUrl, setPreviewFileUrl] = useState<string | null>(null);
    const [previewTitle, setPreviewTitle] = useState("");
    const [isPreviewOpen, setIsPreviewOpen] = useState(false);
    const [selectedRequest, setSelectedRequest] = useState<LeaveRequest | null>(null);
    const {
        isModalOpen, setIsModalOpen, setIsImportModalOpen, activeTab, setActiveTab, statusFilter, setStatusFilter,
        employeeId, setEmployeeId, employees, requests, requestsLoading, requestsTotal, leaveTypes, hasNextPage, isFetchingNextPage,
        loadMoreRef, approveMutation, importJob, importBannerDismissed, setImportBannerDismissed, t,
    } = useLeaveRequests();
    const { data: departments = [] } = useDepartments();
    const employeeMap = useMemo(() => {
        const map = new Map<string, Employee>();
        employees.forEach((employee) => {
            map.set(employee.id, employee);
            if (employee.active_employment_id) map.set(employee.active_employment_id, employee);
        });
        return map;
    }, [employees]);
    const canManageBehalf = hasPermission("leave:policies:manage");
    const handleApprove = (id: string, status: string) => { approveMutation.mutate({ id, status: status as "approved" | "rejected" | "pre_approved" | "pending" | "cancelled" }); };
    const handlePreview = (url: string, title: string) => { setPreviewFileUrl(url); setPreviewTitle(title); setIsPreviewOpen(true); };

    return (
        <PageContainer>
            <PageHeader actions={<LeavePageActions onCreate={() => { setIsModalOpen(true); }} onImport={() => { setIsImportModalOpen(true); }} t={t} />} className="border-b border-slate-50 px-1 pb-4 dark:border-slate-800" title={t("leave_requests.title")} />
            <LeavePageNavigation activeTab={activeTab} canManageBehalf={canManageBehalf} departments={departments} employeeId={employeeId} setActiveTab={setActiveTab} setEmployeeId={setEmployeeId} t={t} />
            {activeTab === "behalf_assignments" && canManageBehalf ? (
                <ResilientWidget><BehalfAssignmentsTab assignmentType="leave" /></ResilientWidget>
            ) : (
                <ResilientWidget>
                    <div className="space-y-6">
                        <LeaveImportBanner importBannerDismissed={importBannerDismissed} importJob={importJob} setImportBannerDismissed={setImportBannerDismissed} t={t} />
                        <LeaveRequestList activeTab={activeTab} employeeMap={employeeMap} formatDate={formatDate} hasNextPage={hasNextPage} isFetchingNextPage={isFetchingNextPage} leaveTypes={leaveTypes} loadMoreRef={loadMoreRef} onApprove={handleApprove} onPreview={handlePreview} onSelect={setSelectedRequest} requests={requests} requestsLoading={requestsLoading} setActiveTab={setActiveTab} setStatusFilter={setStatusFilter} statusFilter={statusFilter} t={t} />
                        <LeaveRequestOverlays employeeMap={employeeMap} formatDate={formatDate} isModalOpen={isModalOpen} isPreviewOpen={isPreviewOpen} leaveTypes={leaveTypes} onApprove={handleApprove} onCloseDetails={() => { setSelectedRequest(null); }} onCloseModal={() => { setIsModalOpen(false); }} onClosePreview={() => { setIsPreviewOpen(false); }} previewFileUrl={previewFileUrl} previewTitle={previewTitle} selectedRequest={selectedRequest} t={t} />
                    </div>
                </ResilientWidget>
            )}
            {activeTab !== "behalf_assignments" ? <FloatingRecordBadge label={t("common.pagination.records")} loaded={requests.length} total={requestsTotal} /> : null}
        </PageContainer>
    );
}
