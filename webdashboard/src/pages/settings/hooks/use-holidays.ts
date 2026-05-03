import { useQuery } from "@tanstack/react-query";
import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";

import type { Holiday } from "@/lib/api/types";

import { useConfirm } from "@/hooks/use-confirm";
import { getCurrentWorkspaceId } from "@/lib/api";
import { deleteHoliday, fetchHolidays } from "@/lib/api/holidays";
import { fetchLeaveTypes } from "@/lib/api/leaves";

import { useHolidayForm } from "./use-holiday-form";

export function useHolidays() {
    const { t } = useTranslation();
    const [year, setYear] = useState(new Date().getFullYear());
    const [showForm, setShowForm] = useState(false);
    const [editingHoliday, setEditingHoliday] = useState<Holiday | null>(null);
    const [error, setError] = useState<string | null>(null);
    const { confirm, ConfirmationDialog } = useConfirm();

    const wsId = getCurrentWorkspaceId();

    const { data: holidays = [], isLoading, refetch: loadHolidays } = useQuery({
        enabled: !!wsId,
        queryFn: () => {
            if (!wsId) { throw new Error("Workspace ID is required"); }
            return fetchHolidays(wsId, year);
        },
        queryKey: ["holidays", wsId, year],
    });

    const { data: leaveTypes = [] } = useQuery({
        enabled: !!wsId,
        queryFn: () => {
            if (!wsId) { throw new Error("Workspace ID is required"); }
            return fetchLeaveTypes(wsId);
        },
        queryKey: ["leaveTypes", wsId],
    });

    const onSaveSuccess = useCallback(() => {
        setEditingHoliday(null);
        setShowForm(false);
        setError(null);
        void loadHolidays();
    }, [loadHolidays]);

    const onCancel = useCallback(() => {
        setEditingHoliday(null);
        setShowForm(false);
        setError(null);
    }, []);

    const form = useHolidayForm({
        editingHoliday,
        onCancel,
        onSaveSuccess,
        workspaceId: wsId ?? undefined,
    });

    const openEdit = useCallback((h: Holiday) => {
        setEditingHoliday(h);
        form.formActions.loadHoliday(h);
        setShowForm(true);
    }, [form.formActions]);

    const handleDelete = async (id: string) => {
        if (!wsId) { return; }
        const ok = await confirm({
            message: t("holidays.messages.confirm_delete"),
            title: t("common.confirm_delete", "Confirm Delete"),
            variant: "danger",
        });
        if (!ok) { return; }
        try {
            await deleteHoliday(wsId, id);
            void loadHolidays();
        } catch {
            setError(t("holidays.messages.delete_error"));
        }
    };

    return {
        ConfirmationDialog,
        editingHoliday,
        error,
        formActions: {
            ...form.formActions,
            handleDelete,
            openEdit,
        },
        formState: form.formState,
        holidays,
        isLoading,
        leaveTypes,
        setShowForm,
        setYear,
        showForm,
        year,
    };
}
