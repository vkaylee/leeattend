import { Plus } from "lucide-react";
import { useTranslation } from "react-i18next";

interface HolidayHeaderProps {
    onAdd: () => void;
}

export function HolidayHeader({ onAdd }: HolidayHeaderProps) {
    const { t } = useTranslation();
    return (
        <div className="flex items-center justify-between">
            <div>
                <h1 className="text-2xl font-bold text-slate-900 dark:text-white tracking-tight">
                    🏖️ {t('holidays.title')}
                </h1>
                <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">
                    {t('holidays.subtitle')}
                </p>
            </div>
            <button
                className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-xl font-bold text-sm hover:bg-primary-700 transition-colors shadow-sm"
                onClick={onAdd}
            >
                <Plus size={16} />
                {t('holidays.add_btn')}
            </button>
        </div>
    );
}
