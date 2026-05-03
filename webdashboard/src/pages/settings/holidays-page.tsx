import { HolidayForm } from "./components/holidays/holiday-form";
import { HolidayHeader } from "./components/holidays/holiday-header";
import { HolidayTable } from "./components/holidays/holiday-table";
import { YearNavigation } from "./components/holidays/year-navigation";
import { useHolidays } from "./hooks/use-holidays";

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
        ConfirmationDialog
    } = useHolidays();

    return (
        <div className="space-y-8">
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
        </div>
    );
}
