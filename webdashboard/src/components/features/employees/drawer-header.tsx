import { ChevronDown, X, User, FolderOpen, FileText, Milestone, ArrowLeftRight, DollarSign, History, Settings, CheckSquare } from "lucide-react";
import React from "react";

import type { Employee } from "@/lib/api";

import Button from "@/components/ui/button";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

export type DetailTabType = "profile" | "documents" | "contracts" | "phases" | "assignments" | "salary" | "compensation" | "policies" | "lifecycle";

export interface DrawerHeaderProps {
    activeDetailTab: DetailTabType;
    employee: Employee;
    onClose: () => void;
    setActiveDetailTab: (tab: DetailTabType) => void;
    t: (k: string) => string;
}

export default function DrawerHeader({
    employee,
    onClose,
    activeDetailTab,
    setActiveDetailTab,
    t
}: DrawerHeaderProps) {
    const tabs = [
        { id: "profile" as DetailTabType, label: t("common.profile"), icon: User },
        { id: "documents" as DetailTabType, label: "Tài liệu & Hồ sơ", icon: FolderOpen },
        { id: "contracts" as DetailTabType, label: "Hợp đồng", icon: FileText },
        { id: "phases" as DetailTabType, label: "Hành trình", icon: Milestone },
        { id: "assignments" as DetailTabType, label: "Phái cử", icon: ArrowLeftRight },
        { id: "salary" as DetailTabType, label: t("employees.details.salary_benefits"), icon: DollarSign },
        { id: "compensation" as DetailTabType, label: t("employees.details.compensation_history"), icon: History },
        { id: "policies" as DetailTabType, label: t("employees.details.special_policies"), icon: Settings },
        { id: "lifecycle" as DetailTabType, label: "Checklist", icon: CheckSquare }
    ];
    const activeTabObj = tabs.find(tab => tab.id === activeDetailTab) || tabs[0];

    return (
        <div className="sticky top-0 z-20 p-10 border-b border-slate-50 dark:border-slate-800 bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm shadow-sm">
            <div className="flex justify-between items-start mb-8">
                <div className="flex items-center gap-6">
                    <div className="w-20 h-20 bg-primary-600 text-white rounded-[2rem] flex items-center justify-center font-bold text-3xl shadow-xl shadow-primary-100 dark:shadow-black/30 border-4 border-white rotate-3 group hover:rotate-0 transition-transform duration-500 uppercase">{employee.full_name[0]}</div>
                    <div>
                        <Typography className="text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none mb-2" variant="h2">{employee.full_name}</Typography>
                        <div className="flex items-center gap-3">
                            <span className="text-primary-600 dark:text-primary-400 font-mono text-xs font-bold tracking-widest uppercase bg-white dark:bg-slate-900 px-3 py-1 rounded-full border border-primary-100 dark:border-primary-800 shadow-sm">{employee.employee_id_number}</span>
                            <div className="w-1.5 h-1.5 bg-slate-200 dark:bg-slate-600 rounded-full" />
                            <span className={cn("text-[9px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-full border shadow-sm", employee.active_employment_id ? 'bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800' : 'bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 opacity-60')}>{employee.active_employment_id ? t("employees.table.active").toUpperCase() : t("employees.table.terminated").toUpperCase()}</span>
                        </div>
                    </div>
                </div>
                <Button aria-label={t("common.close")} className="p-3 bg-white dark:bg-slate-900 hover:bg-slate-50 dark:hover:bg-slate-800 rounded-[1.25rem] transition-all shadow-md active:scale-90 border border-slate-100 dark:border-slate-800 text-slate-300 dark:text-slate-600 hover:text-rose-600 dark:text-rose-300 h-auto w-auto shadow-none" onClick={onClose} size="icon" type="button" variant="ghost"><X size={24} /></Button>
            </div>

            {/* Desktop Navigation Tabs */}
            <div className="hidden md:flex overflow-x-auto scrollbar-hide bg-white dark:bg-slate-900/50 p-1.5 rounded-[22px] border border-slate-200 dark:border-slate-700/50 shadow-inner w-full max-w-full relative">
                {tabs.map(tab => (
                    <Button className={cn("px-5 py-2.5 shrink-0 whitespace-nowrap rounded-[18px] text-[10px] font-bold uppercase tracking-widest transition-colors duration-200 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 h-auto shadow-none border border-transparent", activeDetailTab === tab.id ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-transparent")} data-testid={`employees-tab-${tab.id}`} key={tab.id} onClick={() => { setActiveDetailTab(tab.id); }} type="button" variant="ghost">{tab.label.toUpperCase()}</Button>
                ))}
            </div>

            {/* Mobile Navigation Dropdown */}
            <div className="flex md:hidden w-full relative">
                <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <Button
                            className="w-full flex items-center justify-between px-6 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700/50 rounded-[18px] shadow-sm text-[10px] font-bold uppercase tracking-widest text-slate-700 dark:text-slate-200 transition-all hover:bg-slate-50 dark:hover:bg-slate-800 h-auto"
                            type="button"
                            variant="ghost"
                        >
                            <span className="flex items-center gap-3">
                                {React.createElement(activeTabObj.icon, { size: 14, className: "text-primary-500" })}
                                {activeTabObj.label.toUpperCase()}
                            </span>
                            <ChevronDown className="text-slate-400 dark:text-slate-500 transition-transform duration-200" size={14} />
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start" className="w-[calc(100vw-5rem)] max-w-md max-h-[300px] overflow-y-auto rounded-2xl shadow-xl">
                        {tabs.map((tab) => (
                            <DropdownMenuItem
                                className={cn(
                                    "flex items-center gap-3 px-5 py-3 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all",
                                    activeDetailTab === tab.id
                                        ? "bg-primary-50 dark:bg-primary-950/50 text-primary-600 dark:text-primary-400"
                                        : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
                                )}
                                key={tab.id}
                                onClick={() => {
                                    setActiveDetailTab(tab.id);
                                }}
                            >
                                {React.createElement(tab.icon, {
                                    size: 12,
                                    className: cn(
                                        activeDetailTab === tab.id
                                            ? "text-primary-500"
                                            : "text-slate-400 dark:text-slate-500"
                                    ),
                                })}
                                {tab.label.toUpperCase()}
                            </DropdownMenuItem>
                        ))}
                    </DropdownMenuContent>
                </DropdownMenu>
            </div>
        </div>
    );
}
