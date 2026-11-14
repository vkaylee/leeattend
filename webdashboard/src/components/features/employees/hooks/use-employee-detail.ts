import { useQuery } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";

import type { Employee, OrganizationMember, Employment } from "@/lib/api";

import { organizationKeys } from "@/hooks/use-organization-queries";
import { fetchOrganizationMembers, fetchEmployeeEmployments } from "@/lib/api";
import { useLocations } from "@/pages/settings/hooks/use-locations";

import type { DetailTabType } from "../drawer-header";

const TAB_ORDER: DetailTabType[] = ["profile", "documents", "contracts", "salary", "compensation"];
const SCROLL_STEP = 80;

interface UseEmployeeDetailProps {
    employee: Employee | null;
    isOpen: boolean;
}

export function useEmployeeDetail({ isOpen,employee }: UseEmployeeDetailProps) {
    const [activeDetailTab, setActiveDetailTab] = useState<DetailTabType>("profile");
    const scrollRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!isOpen) return;
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
                e.preventDefault();
                setActiveDetailTab((previous) => {
                    const index = TAB_ORDER.indexOf(previous);
                    if (e.key === "ArrowRight") return TAB_ORDER[(index + 1) % TAB_ORDER.length];
                    return TAB_ORDER[(index - 1 + TAB_ORDER.length) % TAB_ORDER.length];
                });
            } else if (e.key === "ArrowUp" || e.key === "ArrowDown") {
                e.preventDefault();
                e.stopPropagation();
                const sheetScrollContainer = scrollRef.current?.closest(".overflow-y-auto");
                sheetScrollContainer?.scrollBy({
                    behavior: "smooth",
                    top: e.key === "ArrowDown" ? SCROLL_STEP : -SCROLL_STEP,
                });
            }
        };
        window.addEventListener("keydown", handleKeyDown, true);
        return () => { window.removeEventListener("keydown", handleKeyDown, true); };
    }, [isOpen]);

    const { data: members } = useQuery({
        enabled: !!employee?.user_id,
        queryFn: () => fetchOrganizationMembers(),
        queryKey: organizationKeys.members(),
    });

    const { data: locations } = useLocations();

    const { data: employments = [] } = useQuery({
        enabled: !!employee?.id,
        queryFn: () => employee ? fetchEmployeeEmployments(employee.id) : Promise.resolve([]),
        queryKey: ["employee-employments",employee?.id],
    });
    const activeEmployment = employments.find((e: Employment) => e.status === "active") ?? employments[0];

    const linkedUser = members?.find((m: OrganizationMember) => m.user_id === employee?.user_id);

    return {
        activeDetailTab,
        activeEmployment,
        linkedUser,
        locations,
        scrollRef,
        setActiveDetailTab,
    };
}
