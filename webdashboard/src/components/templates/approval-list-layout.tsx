import { Fragment, isValidElement, type ReactNode } from "react";
import { Clock } from "lucide-react";
import { useTranslation } from "react-i18next";

import { ErrorBoundary } from "@/components/common/error-boundary";
import Button from "@/components/ui/button";
import { PageContainer } from "@/components/ui/page-container";
import { TableEmptyRow } from "@/components/ui/table-empty-row";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

export interface TabConfig {
  colorScheme: "amber" | "emerald" | "rose" | "blue" | "slate";
  icon?: ReactNode;
  id: string;
  label: string;
}

function ApprovalListTabs({ activeTab, onTabChange, tabs }: { activeTab: string; onTabChange: (tabId: string) => void; tabs: TabConfig[] }) {
  return (
    <div className="flex w-full max-w-full overflow-x-auto scrollbar-hide bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] border border-primary-200 dark:border-primary-800/50/50 shadow-inner sm:w-fit" role="tablist">
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
          <Button
            aria-selected={isActive}
            className={cn(
              "flex shrink-0 items-center gap-2 px-4 py-2.5 rounded-[18px] text-xs font-semibold transition-all sm:px-8",
              activeClasses,
            )}
            key={tab.id}
            onClick={() => { onTabChange(tab.id); }}
            role="tab"
            type="button"
            variant="ghost"
          >
            {tab.icon} {tab.label}
          </Button>
        );
      })}
    </div>
  );
}

interface ApprovalListLayoutProps {
  activeTab: string;
  banner?: ReactNode;
  children?: ReactNode;
  emptyMessage?: string;
  headerActions?: ReactNode;
  hideHeader?: boolean;
  hideTabs?: boolean;
  isEmpty?: boolean;
  isLoading?: boolean;
  modals?: ReactNode;
  onTabChange: (tabId: string) => void;
  subtitle?: string;
  tableHeaders: ReactNode[];
  tabs?: TabConfig[];
  title?: string;
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
  hideHeader,
  hideTabs,
}: ApprovalListLayoutProps) {
  const { t } = useTranslation();

  return (
    <ErrorBoundary>
      <PageContainer>
      {!hideHeader && (
        <div className="flex flex-col items-stretch md:flex-row md:items-center justify-between gap-6">
          <div className="space-y-1">
            <Typography as="h1" className="text-2xl font-bold tracking-tight text-slate-850 dark:text-slate-100 normal-case mb-0.5" variant="h3">
              {title}
            </Typography>
            {!!subtitle && (
              <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium normal-case tracking-normal" variant="small">
                {subtitle}
              </Typography>
            )}
          </div>
          <div className="flex w-full flex-wrap items-center justify-start gap-3 md:w-auto md:justify-end">
            {headerActions}
          </div>
        </div>
      )}

      {banner}

      {!hideTabs && !!tabs && <ApprovalListTabs activeTab={activeTab} onTabChange={onTabChange} tabs={tabs} />}

      <div className="bg-white dark:bg-slate-900 border border-primary-100 dark:border-primary-800/50 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">
        <div className="overflow-x-auto overscroll-contain">
        <table className="w-full min-w-max text-left border-collapse">
          <thead>
            <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-primary-100 dark:border-primary-800/50">
              {tableHeaders.map((header, index) => {
                if (isValidElement(header) && (header.type === "th" || header.type === "td")) {
                  return <Fragment key={index}>{header}</Fragment>;
                }

                const isFirst = index === 0;
                const isLast = index === tableHeaders.length - 1;
                const headerText = typeof header === "string" ? header : "";
                
                const isStatus = headerText.toLowerCase().includes("status") || headerText.toLowerCase().includes("trạng thái");
                const alignClass = isStatus ? "text-center" : isLast ? "text-right" : "text-left";

                return (
                  <th
                    className={cn(
                      "py-4 text-xs font-semibold text-slate-700 dark:text-slate-300 uppercase tracking-wider",
                      isFirst ? "px-4 md:px-10" : isLast ? "px-4 md:px-10" : "px-4 md:px-8",
                      alignClass
                    )}
                    key={index}
                  >
                    {header}
                  </th>
                );
              })}
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
            {isLoading ? (
              Array.from({ length: 3 }).map((_, i) => (
                 <tr className="animate-pulse border-b border-primary-50 dark:border-primary-800/30" key={`skeleton-${i}`}>
                   {tableHeaders.map((_, colIdx) => (
                     <td className="px-4 py-6 md:px-8 md:py-8" key={`skel-col-${colIdx}`}>
                       <div className={cn("h-4 bg-slate-200 dark:bg-slate-800 rounded-lg opacity-50", colIdx === 0 ? "w-32" : colIdx === tableHeaders.length - 1 ? "w-24 ml-auto" : "w-20")} />
                     </td>
                   ))}
                 </tr>
               ))
            ) : isEmpty ? (
              <TableEmptyRow
                colSpan={tableHeaders.length}
                description={t("common.no_records_found")}
                icon={<Clock className="w-8 h-8" />}
                title={emptyMessage ?? t("common.no_data")}
              />
            ) : (
              children
            )}
          </tbody>
        </table>
        </div>
      </div>

      {modals}
      </PageContainer>
    </ErrorBoundary>
  );
}
