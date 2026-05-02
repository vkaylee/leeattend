import React from "react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/utils";

export interface TabConfig {
  colorScheme: "amber" | "emerald" | "rose" | "blue" | "slate";
  icon?: React.ReactNode;
  id: string;
  label: string;
}

interface ApprovalListLayoutProps {
  activeTab: string;
  banner?: React.ReactNode;
  children?: React.ReactNode;
  emptyMessage?: string;
  headerActions?: React.ReactNode;
  isEmpty?: boolean;
  isLoading?: boolean;
  modals?: React.ReactNode;
  onTabChange: (tabId: string) => void;
  subtitle: string;
  tableHeaders: React.ReactNode[];
  tabs: TabConfig[];
  title: string;
}

export default function ApprovalListLayout({
  title,
  subtitle,
  headerActions,
  banner,
  tabs,
  activeTab,
  onTabChange,
  tableHeaders,
  isLoading,
  isEmpty,
  emptyMessage,
  children,
  modals,
}: ApprovalListLayoutProps) {
  const { t } = useTranslation();

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row justify-between items-center gap-8 px-2">
        <div className="space-y-1">
          <h1 className="text-5xl font-bold tracking-tighter text-primary-700 dark:text-primary-300 uppercase leading-none">
            {title}
          </h1>
          <p className="text-[10px] font-bold text-primary-400 uppercase tracking-[0.25em] opacity-60">
            {subtitle}
          </p>
        </div>
        <div className="flex items-center gap-3 flex-wrap">
          {headerActions}
        </div>
      </div>

      {banner}

      <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-primary-200 dark:border-primary-800/50/50 shadow-inner">
        {tabs.map((tab) => {
          const isActive = activeTab === tab.id;
          
          let activeClasses = "";
          if (isActive) {
            switch (tab.colorScheme) {
              case "amber":
                activeClasses = "bg-white dark:bg-slate-900 text-amber-600 dark:text-amber-300 shadow-sm border border-primary-100 dark:border-primary-800/50";
                break;
              case "emerald":
                activeClasses = "bg-white dark:bg-slate-900 text-emerald-600 dark:text-emerald-300 shadow-sm border border-primary-100 dark:border-primary-800/50";
                break;
              case "rose":
                activeClasses = "bg-white dark:bg-slate-900 text-rose-600 dark:text-rose-300 shadow-sm border border-primary-100 dark:border-primary-800/50";
                break;
              case "blue":
                activeClasses = "bg-white dark:bg-slate-900 text-blue-600 dark:text-blue-300 shadow-sm border border-primary-100 dark:border-primary-800/50";
                break;
              default:
                activeClasses = "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-300 shadow-sm border border-primary-100 dark:border-primary-800/50";
            }
          } else {
            activeClasses = "text-primary-400 hover:text-primary-700 dark:text-primary-300";
          }

          return (
            <button
              className={cn(
                "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
                activeClasses
              )}
              key={tab.id}
              onClick={() => { onTabChange(tab.id); }}
            >
              {tab.icon} {tab.label}
            </button>
          );
        })}
      </div>

      <div className="bg-white dark:bg-slate-900 border border-primary-100 dark:border-primary-800/50 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">
        <table className="w-full text-left border-collapse">
          <thead>
            <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-primary-100 dark:border-primary-800/50">
              {tableHeaders.map((header, index) => (
                <React.Fragment key={index}>{header}</React.Fragment>
              ))}
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
            {isLoading ? (
              <tr>
                <td className="px-10 py-20 text-center text-primary-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse" colSpan={tableHeaders.length}>
                  {t("common.loading")}
                </td>
              </tr>
            ) : isEmpty ? (
              <tr>
                <td className="px-10 py-32 text-center text-primary-400 font-bold italic uppercase tracking-widest text-xs opacity-50" colSpan={tableHeaders.length}>
                  {emptyMessage || t("common.no_data")}
                </td>
              </tr>
            ) : (
              children
            )}
          </tbody>
        </table>
      </div>

      {modals}
    </div>
  );
}
