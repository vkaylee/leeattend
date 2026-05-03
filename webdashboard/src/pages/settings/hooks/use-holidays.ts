import { useQuery } from "@tanstack/react-query";
import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import type { Holiday } from "@/lib/api/types";
import { useConfirm } from "@/hooks/use-confirm";
import { getCurrentWorkspaceId } from "@/lib/api";
import { fetchHolidays, createHoliday, updateHoliday, deleteHoliday } from "@/lib/api/holidays";
import { fetchLeaveTypes } from "@/lib/api/leaves";

export function useHolidays() {
    const { t } = useTranslation();
    const [year, setYear] = useState(new Date().getFullYear());
    const [showForm, setShowForm] = useState(false);
    const [editingHoliday, setEditingHoliday] = useState<Holiday | null>(null);
    const [error, setError] = useState<string | null>(null);
    const { confirm, ConfirmationDialog } = useConfirm();

    // Form state
    const [formName, setFormName] = useState("");
    const [formDate, setFormDate] = useState("");
    const [formType, setFormType] = useState("national");
    const [formIsPaid, setFormIsPaid] = useState(true);
    const [formLeaveTypeId, setFormLeaveTypeId] = useState("");
    const [formOtMultiplier, setFormOtMultiplier] = useState("");
    const [formDescription, setFormDescription] = useState("");
    const [formIsCreditOnWork, setFormIsCreditOnWork] = useState(true);
    const [formThresholdPercent, setFormThresholdPercent] = useState("");

    const wsId = getCurrentWorkspaceId();

    const { data: holidays = [], isLoading, refetch: loadHolidays } = useQuery({
        queryKey: ["holidays", wsId, year],
        queryFn: () => {
            if (!wsId) throw new Error("Workspace ID is required");
            return fetchHolidays(wsId, year);
        },
        enabled: !!wsId
    });

    const { data: leaveTypes = [] } = useQuery({
        queryKey: ["leaveTypes", wsId],
        queryFn: () => {
            if (!wsId) throw new Error("Workspace ID is required");
            return fetchLeaveTypes(wsId);
        },
        enabled: !!wsId
    });

    const resetForm = useCallback(() => {
        setFormName("");
        setFormDate("");
        setFormType("national");
        setFormIsPaid(true);
        setFormLeaveTypeId("");
        setFormOtMultiplier("");
        setFormDescription("");
        setFormIsCreditOnWork(true);
        setFormThresholdPercent("");
        setEditingHoliday(null);
        setShowForm(false);
        setError(null);
    }, []);

    const openEdit = useCallback((h: Holiday) => {
        setEditingHoliday(h);
        setFormName(h.name);
        setFormDate(h.date);
        setFormType(h.holiday_type);
        setFormIsPaid(h.is_paid);
        setFormLeaveTypeId(h.leave_type_id ?? "");
        setFormOtMultiplier(h.ot_multiplier?.toString() ?? "");
        setFormDescription(h.description ?? "");
        setFormIsCreditOnWork(h.is_credit_on_work);
        setFormThresholdPercent(h.work_threshold_percent?.toString() ?? "");
        setShowForm(true);
    }, []);

    const handleSubmit = async (e: React.SyntheticEvent) => {
        e.preventDefault();
        const currentWsId = wsId;
        if (!currentWsId) return;
        setError(null);

        const payload = {
            name: formName,
            date: formDate,
            holiday_type: formType,
            is_paid: formIsPaid,
            leave_type_id: !formIsPaid && formLeaveTypeId ? formLeaveTypeId : null,
            description: formDescription || undefined,
            ot_multiplier: formOtMultiplier ? parseFloat(formOtMultiplier) : undefined,
            is_credit_on_work: formIsCreditOnWork,
            work_threshold_percent: formThresholdPercent ? parseInt(formThresholdPercent) : null,
        } as Parameters<typeof createHoliday>[1];

        try {
            if (editingHoliday) {
                await updateHoliday(currentWsId, editingHoliday.id, payload);
            } else {
                await createHoliday(currentWsId, payload);
            }
            resetForm();
            void loadHolidays();
        } catch (error_: unknown) {
            setError(error_ instanceof Error ? error_.message : t('holidays.messages.save_error'));
        }
    };

    const handleDelete = async (id: string) => {
        if (!wsId) return;
        const ok = await confirm({
            title: t('common.confirm_delete', 'Confirm Delete'),
            message: t('holidays.messages.confirm_delete'),
            variant: 'danger'
        });
        if (!ok) return;
        try {
            await deleteHoliday(wsId, id);
            void loadHolidays();
        } catch {
            setError(t('holidays.messages.delete_error'));
        }
    };

    return {
        year,
        setYear,
        showForm,
        setShowForm,
        editingHoliday,
        error,
        holidays,
        isLoading,
        leaveTypes,
        formState: {
            name: formName,
            date: formDate,
            type: formType,
            isPaid: formIsPaid,
            leaveTypeId: formLeaveTypeId,
            otMultiplier: formOtMultiplier,
            description: formDescription,
            isCreditOnWork: formIsCreditOnWork,
            thresholdPercent: formThresholdPercent,
        },
        formActions: {
            setName: setFormName,
            setDate: setFormDate,
            setType: setFormType,
            setIsPaid: setFormIsPaid,
            setLeaveTypeId: setFormLeaveTypeId,
            setOtMultiplier: setFormOtMultiplier,
            setDescription: setFormDescription,
            setIsCreditOnWork: setFormIsCreditOnWork,
            setThresholdPercent: setFormThresholdPercent,
            resetForm,
            handleSubmit,
            openEdit,
            handleDelete
        },
        ConfirmationDialog
    };
}
