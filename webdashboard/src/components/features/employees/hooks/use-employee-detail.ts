import { useQuery } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";

import type { Employee, WorkspaceMember, Employment } from "@/lib/api";
import type { WorkLocation } from "@/lib/api-generated/models";
import { fetchWorkspaceMembers, fetchEmployeeEmployments } from "@/lib/api";
import { useLocations } from "@/pages/settings/hooks/use-locations";

import type { DetailTabType } from "../drawer-header";

const TAB_ORDER: DetailTabType[] = ["profile", "documents", "contracts", "phases", "assignments", "salary", "compensation", "policies", "lifecycle"];
const SCROLL_STEP = 80;

interface UseEmployeeDetailProps {
    employee: Employee | null;
    isOpen: boolean;
    wsId: string;
}

export function useEmployeeDetail({ isOpen, wsId, employee }: UseEmployeeDetailProps) {
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
        enabled: !!wsId && !!employee?.user_id,
        queryFn: () => fetchWorkspaceMembers(wsId),
        queryKey: ["workspace-members", wsId],
    });

    const { data: locations } = useLocations(wsId);

    const { data: employments = [] } = useQuery({
        enabled: !!wsId && !!employee?.id,
        queryFn: () => employee ? fetchEmployeeEmployments(wsId, employee.id) : Promise.resolve([]),
        queryKey: ["employee-employments", wsId, employee?.id],
    });
    const activeEmployment = employments.find((e: Employment) => e.status === "active") || employments[0];

    const linkedUser = members?.find((m: WorkspaceMember) => m.user_id === employee?.user_id);

    return {
        activeDetailTab,
        activeEmployment,
        linkedUser,
        locations,
        scrollRef,
        setActiveDetailTab,
    };
}
