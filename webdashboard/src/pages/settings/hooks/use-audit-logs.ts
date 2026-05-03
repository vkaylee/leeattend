import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

import { getCurrentWorkspaceId, fetchAuditLogs } from "@/lib/api";

export function useAuditLogs() {
    const workspaceId = getCurrentWorkspaceId();
    const [page, setPage] = useState(1);
    const limit = 20;

    const [actionFilter, setActionFilter] = useState("");
    const [startDateFilter, setStartDateFilter] = useState("");
    const [endDateFilter, setEndDateFilter] = useState("");

    const { data, isLoading, isError } = useQuery({
        queryKey: ["audit_logs", workspaceId, page, actionFilter, startDateFilter, endDateFilter],
        queryFn: () => {
            if (!workspaceId) throw new Error("Workspace ID is required");
            return fetchAuditLogs(workspaceId, {
                page,
                limit,
                action: actionFilter || undefined,
                startDate: startDateFilter ? new Date(startDateFilter).toISOString() : undefined,
                endDate: endDateFilter ? new Date(endDateFilter).toISOString() : undefined,
            });
        },
        enabled: !!workspaceId,
    });

    const handleActionFilterChange = (value: string) => {
        setActionFilter(value);
        setPage(1);
    };

    const handleStartDateChange = (value: string) => {
        setStartDateFilter(value);
        setPage(1);
    };

    const handleEndDateChange = (value: string) => {
        setEndDateFilter(value);
        setPage(1);
    };

    const handlePageChange = (newPage: number) => {
        setPage(newPage);
    };

    return {
        data,
        isLoading,
        isError,
        page,
        limit,
        actionFilter,
        startDateFilter,
        endDateFilter,
        handleActionFilterChange,
        handleStartDateChange,
        handleEndDateChange,
        handlePageChange,
    };
}
