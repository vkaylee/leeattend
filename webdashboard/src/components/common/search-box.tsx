import React from "react";
import { useTranslation } from "react-i18next";

import { SearchInput } from "@/components/ui/search-input";

interface SearchBoxProps {
    className?: string;
    "data-testid"?: string;
    inputRef?: React.RefObject<HTMLInputElement | null>;
    onChange: (value: string) => void;
    onKeyDown?: (e: React.KeyboardEvent<HTMLInputElement>) => void;
    placeholder?: string;
    value: string;
}

export default function SearchBox({
    value,
    onChange,
    placeholder,
    inputRef,
    onKeyDown,
    className,
    "data-testid": testId,
}: SearchBoxProps) {
    const { t } = useTranslation();
    const displayPlaceholder = placeholder ?? t("common.search");

    return (
        <SearchInput
            containerClassName={className}
            data-testid={testId}
            onChange={(e) => onChange(e.target.value)}
            onClear={() => onChange("")}
            onFocus={(e) => e.target.select()}
            onKeyDown={onKeyDown}
            placeholder={displayPlaceholder}
            ref={inputRef}
            size="md"
            value={value}
        />
    );
}
