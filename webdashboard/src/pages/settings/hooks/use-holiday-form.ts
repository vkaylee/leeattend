import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";

import type { Holiday } from "@/lib/api/types";

import { createHoliday, updateHoliday } from "@/lib/api/holidays";

interface UseHolidayFormProps {
    editingHoliday: Holiday | null;
    onCancel: () => void;
    onSaveSuccess: () => void;
    workspaceId: string | undefined;
}

export function useHolidayForm({ editingHoliday, onCancel, onSaveSuccess, workspaceId }: UseHolidayFormProps) {
    const { t } = useTranslation();
    const [name, setName] = useState("");
    const [date, setDate] = useState("");
    const [type, setType] = useState("national");
    const [isPaid, setIsPaid] = useState(true);
    const [leaveTypeId, setLeaveTypeId] = useState("");
    const [otMultiplier, setOtMultiplier] = useState("");
    const [description, setDescription] = useState("");
    const [isCreditOnWork, setIsCreditOnWork] = useState(true);
    const [thresholdPercent, setThresholdPercent] = useState("");
    const [error, setError] = useState<string | null>(null);

    const resetForm = useCallback(() => {
        setName("");
        setDate("");
        setType("national");
        setIsPaid(true);
        setLeaveTypeId("");
        setOtMultiplier("");
        setDescription("");
        setIsCreditOnWork(true);
        setThresholdPercent("");
        setError(null);
        onCancel();
    }, [onCancel]);

    const loadHoliday = useCallback((h: Holiday) => {
        setName(h.name);
        setDate(h.date);
        setType(h.holiday_type);
        setIsPaid(h.is_paid);
        setLeaveTypeId(h.leave_type_id ?? "");
        setOtMultiplier(h.ot_multiplier?.toString() ?? "");
        setDescription(h.description ?? "");
        setIsCreditOnWork(h.is_credit_on_work);
        setThresholdPercent(h.work_threshold_percent?.toString() ?? "");
    }, []);

    const handleSubmit = async (e: React.SyntheticEvent) => {
        e.preventDefault();
        if (!workspaceId) { return; }
        setError(null);

        const payload = {
            date,
            description: description || undefined,
            holiday_type: type,
            is_credit_on_work: isCreditOnWork,
            is_paid: isPaid,
            leave_type_id: !isPaid && leaveTypeId ? leaveTypeId : null,
            name,
            ot_multiplier: otMultiplier ? parseFloat(otMultiplier) : undefined,
            work_threshold_percent: thresholdPercent ? parseInt(thresholdPercent, 10) : null,
        } as Parameters<typeof createHoliday>[1];

        try {
            if (editingHoliday) {
                await updateHoliday(workspaceId, editingHoliday.id, payload);
            } else {
                await createHoliday(workspaceId, payload);
            }
            onSaveSuccess();
            resetForm();
        } catch (err: unknown) {
            setError(err instanceof Error ? err.message : t("holidays.messages.save_error"));
        }
    };

    return {
        formActions: {
            handleSubmit,
            loadHoliday,
            resetForm,
            setDate,
            setDescription,
            setIsCreditOnWork,
            setIsPaid,
            setLeaveTypeId,
            setName,
            setOtMultiplier,
            setThresholdPercent,
            setType,
        },
        formState: {
            date,
            description,
            error,
            isCreditOnWork,
            isPaid,
            leaveTypeId,
            name,
            otMultiplier,
            thresholdPercent,
            type,
        },
    };
}
