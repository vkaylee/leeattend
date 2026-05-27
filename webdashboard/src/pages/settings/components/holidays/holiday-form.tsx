import { Info } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { Holiday, LeaveType } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { Typography } from "@/components/ui/typography";

interface HolidayFormProps {
    editingHoliday: Holiday | null;
    formActions: {
        resetForm: () => void;
        setDate: (v: string) => void;
        setDescription: (v: string) => void;
        handleSubmit: (e: React.SyntheticEvent) => Promise<void>;
        setIsCreditOnWork: (v: boolean) => void;
        setIsPaid: (v: boolean) => void;
        setLeaveTypeId: (v: string) => void;
        setName: (v: string) => void;
        setOtMultiplier: (v: string) => void;
        setThresholdPercent: (v: string) => void;
        setType: (v: string) => void;
    };
    formState: {
        date: string;
        description: string;
        isCreditOnWork: boolean;
        isPaid: boolean;
        leaveTypeId: string;
        name: string;
        otMultiplier: string;
        thresholdPercent: string;
        type: string;
    };
    leaveTypes: LeaveType[];
}

function HolidayFormBasics({ formActions, formState }: Pick<HolidayFormProps, "formActions" | "formState">) {
    const { t } = useTranslation();
    return (
        <>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                    <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                        {t("holidays.fields.name")} *
                    </label>
                    <input
                        className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm"
                        onChange={(e) => { formActions.setName(e.target.value); }}
                        placeholder={t("holidays.fields.name_placeholder")}
                        required
                        type="text"
                        value={formState.name}
                    />
                </div>
                <div>
                    <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                        {t("holidays.fields.date")} *
                    </label>
                    <input
                        className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm"
                        onChange={(e) => { formActions.setDate(e.target.value); }}
                        required
                        type="date"
                        value={formState.date}
                    />
                </div>
                <div>
                    <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                        {t("holidays.fields.type")}
                    </label>
                    <Combobox
                        className="w-full border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm py-[7px]"
                        onChange={formActions.setType}
                        options={[
                            { label: t("holidays.fields.types.national"), value: "national" },
                            { label: t("holidays.fields.types.company"), value: "company" },
                            { label: t("holidays.fields.types.regional"), value: "regional" },
                        ]}
                        placeholder={t("holidays.fields.type")}
                        value={formState.type}
                    />
                </div>
            </div>
            <div>
                <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                    {t("holidays.fields.description")}
                </label>
                <input
                    className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm"
                    onChange={(e) => { formActions.setDescription(e.target.value); }}
                    placeholder={t("holidays.fields.description_placeholder")}
                    type="text"
                    value={formState.description}
                />
            </div>
        </>
    );
}

function HolidayFormPaidSection({ formActions, formState, leaveTypes }: Pick<HolidayFormProps, "formActions" | "formState" | "leaveTypes">) {
    const { t } = useTranslation();
    return (
        <>
            <div className="flex items-center gap-2">
                <input
                    checked={formState.isPaid}
                    className="rounded"
                    id="isPaid"
                    onChange={(e) => { formActions.setIsPaid(e.target.checked); if (e.target.checked) { formActions.setLeaveTypeId(""); } }}
                    type="checkbox"
                />
                <label className="text-sm text-slate-700 dark:text-slate-300" htmlFor="isPaid">
                    {t("holidays.fields.paid_holiday")}
                </label>
            </div>
            {!formState.isPaid ? (
                <div className="space-y-3">
                    <div className="p-3 bg-blue-50 dark:bg-blue-900/30 border border-blue-200 dark:border-blue-800/50 rounded-xl flex items-start gap-3">
                        <Info className="w-5 h-5 text-blue-500 dark:text-blue-400 mt-0.5 shrink-0" />
                        <div>
                            <Typography className="text-sm font-bold text-blue-900 dark:text-blue-200 mb-1" variant="h4">
                                {t("holidays.info.title")}
                            </Typography>
                            <Typography className="text-xs text-blue-800 dark:text-blue-300 font-medium mb-2" variant="p">
                                {t("holidays.info.body_1")}
                            </Typography>
                            <ul className="text-xs text-blue-800 dark:text-blue-300 space-y-1 list-disc list-inside">
                                <li><span className="font-bold">{t("holidays.info.rule_1").split(":")[0]}:</span> {t("holidays.info.rule_1").split(":")[1]}</li>
                                <li><span className="font-bold">{t("holidays.info.rule_2").split(":")[0]}:</span> {t("holidays.info.rule_2").split(":")[1]}</li>
                            </ul>
                        </div>
                    </div>
                    <div>
                        <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                            {t("holidays.fields.deduct_leave_type")}
                        </label>
                        <Combobox
                            className="w-full border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm py-[7px]"
                            onChange={formActions.setLeaveTypeId}
                            options={leaveTypes.filter((lt) => lt.is_paid).map((lt) => ({
                                label: `${lt.name} (${lt.code})`,
                                value: lt.id,
                            }))}
                            placeholder={t("holidays.fields.auto_first_paid")}
                            value={formState.leaveTypeId}
                        />
                        <Typography className="text-xs text-amber-600 dark:text-amber-400 mt-1" variant="p">
                            ⚠️ {t("holidays.fields.deduction_warning")}
                        </Typography>
                    </div>
                </div>
            ) : null}
        </>
    );
}

function HolidayWorkPolicySection({ formActions, formState }: Pick<HolidayFormProps, "formActions" | "formState">) {
    const { t } = useTranslation();
    return (
        <div className="mt-2 p-4 bg-slate-50 dark:bg-slate-800/50 rounded-xl border border-slate-200 dark:border-slate-700">
            <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-3 flex items-center gap-1.5" variant="h3">
                ⚙️ {t("holidays.fields.work_policy_title", "Chính sách khi NV đi làm ngày lễ")}
            </Typography>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div className="flex items-start gap-2">
                    <input
                        checked={formState.isCreditOnWork}
                        className="rounded mt-0.5"
                        id="isCreditOnWork"
                        onChange={(e) => { formActions.setIsCreditOnWork(e.target.checked); }}
                        type="checkbox"
                    />
                    <div>
                        <label className="text-sm text-slate-700 dark:text-slate-300 font-medium" htmlFor="isCreditOnWork">
                            {t("holidays.fields.credit_on_work", "Credit lương ngày lễ")}
                        </label>
                        <Typography className="text-[10px] text-slate-400" variant="p">
                            {t("holidays.fields.credit_on_work_help", "NV vẫn nhận lương ngày lễ + lương giờ làm")}
                        </Typography>
                    </div>
                </div>
                <div>
                    <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                        {t("holidays.fields.threshold_percent", "Ngưỡng hoàn phép")} (%)
                    </label>
                    <input
                        className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm"
                        max={100}
                        min={0}
                        onChange={(e) => { formActions.setThresholdPercent(e.target.value); }}
                        placeholder="50"
                        type="number"
                        value={formState.thresholdPercent}
                    />
                    <Typography className="text-[10px] text-slate-400 mt-0.5" variant="p">
                        {t("holidays.fields.threshold_help", "NV làm ≥ ngưỡng → hoàn phép. < ngưỡng → giữ phép")}
                    </Typography>
                </div>
                <div>
                    <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                        {t("holidays.fields.ot_multiplier", "Hệ số nhân giờ làm")}
                    </label>
                    <input
                        className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm"
                        onChange={(e) => { formActions.setOtMultiplier(e.target.value); }}
                        placeholder={t("holidays.fields.ot_multiplier_placeholder", "3.0")}
                        step="0.1"
                        type="number"
                        value={formState.otMultiplier}
                    />
                    <Typography className="text-[10px] text-slate-400 mt-0.5" variant="p">
                        {t("holidays.fields.ot_multiplier_help", "VD: 3× = 300% lương cho mỗi giờ đi làm")}
                    </Typography>
                </div>
            </div>
        </div>
    );
}

export function HolidayForm({ editingHoliday, formActions, formState, leaveTypes }: HolidayFormProps) {
    const { t } = useTranslation();

    return (
        <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200 dark:border-slate-800 p-6 shadow-lg">
            <Typography className="text-lg font-bold text-slate-900 dark:text-white mb-4" variant="h2">
                {editingHoliday ? t("holidays.modal_title_edit") : t("holidays.modal_title_new")}
            </Typography>
            <form className="space-y-4" onSubmit={formActions.handleSubmit}>
                <HolidayFormBasics formActions={formActions} formState={formState} />
                <HolidayFormPaidSection formActions={formActions} formState={formState} leaveTypes={leaveTypes} />
                <HolidayWorkPolicySection formActions={formActions} formState={formState} />

                <div className="flex gap-2 justify-end">
                    <Button
                        className="px-4 py-2 rounded-xl border border-slate-200 dark:border-slate-700 text-sm font-bold text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800"
                        onClick={formActions.resetForm}
                        type="button"
                    >
                        {t("common.cancel")}
                    </Button>
                    <Button
                        className="px-4 py-2 bg-primary-600 text-white rounded-xl font-bold text-sm hover:bg-primary-700 transition-colors"
                        type="submit"
                    >
                        {editingHoliday ? t("common.update") : t("common.create")}
                    </Button>
                </div>
            </form>
        </div>
    );
}
