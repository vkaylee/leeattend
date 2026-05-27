import { Plus } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface HolidayHeaderProps {
    onAdd: () => void;
}

export function HolidayHeader({ onAdd }: HolidayHeaderProps) {
    const { t } = useTranslation();
    return (
        <div className="flex items-center justify-between">
            <div>
                <Typography className="text-2xl font-bold text-slate-900 dark:text-white tracking-tight" variant="h1">
                    🏖️ {t('holidays.title')}
                </Typography>
                <Typography className="text-sm text-slate-500 dark:text-slate-400 mt-1" variant="p">
                    {t('holidays.subtitle')}
                </Typography>
            </div>
            <Button
                className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-xl font-bold text-sm hover:bg-primary-700 transition-colors shadow-sm"
                onClick={onAdd}
            >
                <Plus size={16} />
                {t('holidays.add_btn')}
            </Button>
        </div>
    );
}
