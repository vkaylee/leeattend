import { PageContainer } from "@/components/ui/page-container";

import { HolidayForm } from "./holidays/components/holiday-form";
import { HolidayHeader } from "./holidays/components/holiday-header";
import { HolidayTable } from "./holidays/components/holiday-table";
import { YearNavigation } from "./holidays/components/year-navigation";
import { useHolidays } from "./holidays/hooks/use-holidays";

export default function HolidaysPage() {
    const {
        year,
        setYear,
        showForm,
        setShowForm,
        editingHoliday,
        error,
        holidays,
        isLoading,
        leaveTypes,
        formState,
        formActions,
        ConfirmationDialog,
    } = useHolidays();

    return (
        <PageContainer>
            <HolidayHeader onAdd={() => { formActions.resetForm(); setShowForm(true); }} />

            <YearNavigation setYear={setYear} year={year} />

            {error ? (
                <div className="p-3 bg-rose-50 dark:bg-rose-950 border border-rose-200 dark:border-rose-800 rounded-xl text-rose-600 dark:text-rose-300 text-sm font-medium">
                    {error}
                </div>
            ) : null}

            {showForm ? (
                <HolidayForm
                    editingHoliday={editingHoliday}
                    formActions={formActions}
                    formState={formState}
                    leaveTypes={leaveTypes}
                />
            ) : null}

            <HolidayTable
                holidays={holidays}
                isLoading={isLoading}
                leaveTypes={leaveTypes}
                onDelete={formActions.handleDelete}
                onEdit={formActions.openEdit}
                year={year}
            />

            <ConfirmationDialog />
        </PageContainer>
    );
}
