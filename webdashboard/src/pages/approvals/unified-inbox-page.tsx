import {
  ArrowLeftRight,
  Calendar,
  CheckCircle2,
  Clock,
  FileEdit,
  Inbox,
  RefreshCw,
  Search,
  X,
} from "lucide-react";
import { useState, type ReactNode } from "react";
import type { TFunction } from "i18next";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import { Skeleton } from "@/components/ui/skeleton";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

import { ApprovalBatchToolbar } from "./components/approval-batch-toolbar";
import { ApprovalDetailDrawer } from "./components/approval-detail-drawer";
import { ApprovalItemRow } from "./components/approval-item-row";
import { RejectionReasonModal } from "./components/rejection-reason-modal";
import { useBatchApproval } from "./hooks/use-batch-approval";
import {
  useUnifiedInbox,
  type UnifiedInboxCounts,
} from "./hooks/use-unified-inbox";
import type {
  ApprovalItemStatus,
  ApprovalType,
  UnifiedApprovalItem,
  UnifiedInboxFilter,
} from "./types/unified-approvals";

interface ApprovalTab {
  count?: number;
  icon?: ReactNode;
  id: ApprovalItemStatus | ApprovalType | "all";
  label: string;
}

interface InboxHeaderProps {
  isLoading: boolean;
  pendingCount: number;
  refetchAll: () => Promise<void>;
  t: TFunction;
}

function InboxHeader({ isLoading, pendingCount, refetchAll, t }: InboxHeaderProps) {
  return (
    <header className="flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
      <div className="space-y-1">
        <div className="flex items-center gap-3">
          <Typography className="text-2xl font-black tracking-tight" variant="h2">
            {t("approvals.inbox.title", "Hộp thư Phê duyệt")}
          </Typography>
          {pendingCount > 0 ? (
            <span
              className="inline-flex items-center rounded-full border border-amber-200/80 bg-amber-100 px-2.5 py-0.5 text-xs font-bold text-amber-800 dark:border-amber-800/80 dark:bg-amber-950/80 dark:text-amber-300"
              data-testid="pending-total-badge"
            >
              {pendingCount} {t("approvals.inbox.pending_badge", "chờ duyệt")}
            </span>
          ) : null}
        </div>
        <p className="text-xs text-slate-500 dark:text-slate-400">
          {t(
            "approvals.inbox.subtitle",
            "Quản lý và phê duyệt tập trung các yêu cầu Nghỉ phép, Tăng ca, Giải trình công và Đổi ca",
          )}
        </p>
      </div>
      <Button
        aria-label={t("common.refresh", "Làm mới dữ liệu")}
        className="rounded-xl border border-slate-200 text-slate-700 hover:bg-slate-100 dark:border-slate-800 dark:text-slate-300 dark:hover:bg-slate-800"
        data-testid="inbox-refresh-btn"
        disabled={isLoading}
        onClick={() => void refetchAll()}
        size="sm"
        variant="outline"
      >
        <RefreshCw
          aria-hidden="true"
          className={cn("mr-1.5 h-4 w-4", isLoading && "animate-spin")}
        />
        {t("common.refresh", "Làm mới")}
      </Button>
    </header>
  );
}

function createTypeTabs(
  counts: UnifiedInboxCounts,
  t: TFunction,
): ApprovalTab[] {
  return [
    {
      id: "all",
      label: t("common.all", "Tất cả"),
      count: counts.all,
      icon: <Inbox aria-hidden="true" className="h-4 w-4" />,
    },
    {
      id: "leave",
      label: t("approvals.tabs.leave", "Nghỉ phép"),
      count: counts.leave,
      icon: <Calendar aria-hidden="true" className="h-4 w-4" />,
    },
    {
      id: "overtime",
      label: t("approvals.tabs.overtime", "Tăng ca"),
      count: counts.overtime,
      icon: <Clock aria-hidden="true" className="h-4 w-4" />,
    },
    {
      id: "adjustment",
      label: t("approvals.tabs.adjustment", "Giải trình"),
      count: counts.adjustment,
      icon: <FileEdit aria-hidden="true" className="h-4 w-4" />,
    },
    {
      id: "shift_swap",
      label: t("approvals.tabs.shift_swap", "Đổi ca"),
      count: counts.shift_swap,
      icon: <ArrowLeftRight aria-hidden="true" className="h-4 w-4" />,
    },
  ];
}

function createStatusTabs(t: TFunction): ApprovalTab[] {
  return [
    { id: "pending", label: t("approvals.status.pending", "Chờ duyệt") },
    { id: "approved", label: t("approvals.status.approved", "Đã duyệt") },
    { id: "rejected", label: t("approvals.status.rejected", "Đã từ chối") },
    { id: "all", label: t("approvals.status.all_status", "Tất cả trạng thái") },
  ];
}

interface FilterPanelProps {
  filter: UnifiedInboxFilter;
  pendingCounts: UnifiedInboxCounts;
  setFilter: (patch: Partial<UnifiedInboxFilter>) => void;
  t: TFunction;
}

function FilterPanel({ filter, pendingCounts, setFilter, t }: FilterPanelProps) {
  const typeTabs = createTypeTabs(pendingCounts, t);
  const statusTabs = createStatusTabs(t);

  return (
    <section className="space-y-4 rounded-3xl border border-slate-200/80 bg-white p-4 shadow-sm dark:border-slate-800/80 dark:bg-slate-900">
      <div
        aria-label={t("approvals.type_tabs_label", "Loại yêu cầu")}
        className="flex items-center gap-2 overflow-x-auto pb-1 scrollbar-none"
        role="tablist"
      >
        {typeTabs.map((tab) => (
          <TypeTabButton
            isActive={filter.type === tab.id}
            key={tab.id}
            onClick={() => setFilter({ type: tab.id as ApprovalType | "all" })}
            tab={tab}
          />
        ))}
      </div>
      <div className="flex flex-col items-stretch justify-between gap-3 border-t border-slate-100 pt-2 dark:border-slate-800 sm:flex-row sm:items-center">
        <div className="flex items-center gap-1.5 overflow-x-auto">
          {statusTabs.map((tab) => (
            <Button
              className={cn(
                "rounded-xl px-3 py-1.5 text-xs font-semibold transition-all",
                filter.status === tab.id
                  ? "bg-slate-900 text-white shadow-sm dark:bg-slate-100 dark:text-slate-900"
                  : "text-slate-500 hover:text-slate-800 dark:hover:text-slate-200",
              )}
              data-testid={`status-tab-${tab.id}`}
              key={tab.id}
              onClick={() => setFilter({ status: tab.id as ApprovalItemStatus | "all" })}
              size="sm"
              variant="ghost"
            >
              {tab.label}
            </Button>
          ))}
        </div>
        <SearchFilter filter={filter} setFilter={setFilter} t={t} />
      </div>
    </section>
  );
}

interface TypeTabButtonProps {
  isActive: boolean;
  onClick: () => void;
  tab: ApprovalTab;
}

function TypeTabButton({ isActive, onClick, tab }: TypeTabButtonProps) {
  return (
    <Button
      aria-selected={isActive}
      className={cn(
        "flex shrink-0 items-center gap-2 rounded-2xl px-3.5 py-2 text-xs font-bold transition-all",
        isActive
          ? "bg-primary-600 text-white shadow-md"
          : "bg-slate-100 text-slate-600 dark:bg-slate-800/60 dark:text-slate-400 dark:hover:text-slate-100 hover:text-slate-900",
      )}
      data-testid={`type-tab-${tab.id}`}
      onClick={onClick}
      role="tab"
      size="sm"
      variant="ghost"
    >
      {tab.icon}
      <span>{tab.label}</span>
      {tab.count && tab.count > 0 ? (
        <span
          className={cn(
            "rounded-full px-1.5 py-0.2 text-[10px] font-black",
            isActive
              ? "bg-white/20 text-white"
              : "bg-amber-100 text-amber-800 dark:bg-amber-950 dark:text-amber-300",
          )}
        >
          {tab.count}
        </span>
      ) : null}
    </Button>
  );
}

interface SearchFilterProps {
  filter: UnifiedInboxFilter;
  setFilter: (patch: Partial<UnifiedInboxFilter>) => void;
  t: TFunction;
}

function SearchFilter({ filter, setFilter, t }: SearchFilterProps) {
  const placeholder = t("approvals.search_placeholder", "Tìm kiếm nhân sự, lý do...");
  return (
    <div className="relative min-w-[240px] sm:w-72">
      <Search
        aria-hidden="true"
        className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400"
      />
      <input
        aria-label={placeholder}
        className="h-9 w-full rounded-xl border border-slate-200 bg-slate-50 pl-9 pr-8 text-xs text-slate-900 placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500 dark:border-slate-700 dark:bg-slate-800/80 dark:text-slate-100"
        data-testid="inbox-search-input"
        onChange={(event) => setFilter({ searchQuery: event.target.value })}
        placeholder={placeholder}
        type="text"
        value={filter.searchQuery || ""}
      />
      {filter.searchQuery ? (
        <Button
          aria-label={t("common.clear", "Xóa tìm kiếm")}
          className="absolute right-2 top-1/2 h-5 w-5 -translate-y-1/2 rounded-full p-0 text-slate-400 hover:text-slate-600"
          onClick={() => setFilter({ searchQuery: "" })}
          size="sm"
          variant="ghost"
        >
          <X aria-hidden="true" className="h-3.5 w-3.5" />
        </Button>
      ) : null}
    </div>
  );
}

interface SelectionHeaderProps {
  clearSelection: () => void;
  isAllSelected: boolean;
  itemsCount: number;
  selectAll: () => void;
  t: TFunction;
}

function SelectionHeader({
  clearSelection,
  isAllSelected,
  itemsCount,
  selectAll,
  t,
}: SelectionHeaderProps) {
  return (
    <div className="flex items-center justify-between px-2 text-xs font-semibold text-slate-500 dark:text-slate-400">
      <label className="flex cursor-pointer select-none items-center gap-2">
        <input
          aria-label={t("approvals.select_all", "Chọn tất cả đơn chờ duyệt")}
          checked={isAllSelected}
          className="h-4 w-4 cursor-pointer rounded border-slate-300 text-primary-600 focus:ring-primary-500"
          data-testid="select-all-checkbox"
          onChange={() => (isAllSelected ? clearSelection() : selectAll())}
          type="checkbox"
        />
        <span>
          {isAllSelected
            ? t("approvals.unselect_all", "Bỏ chọn tất cả")
            : t("approvals.select_all", "Chọn tất cả đơn chờ duyệt")}
        </span>
      </label>
      <span>{t("approvals.showing_count", "Hiển thị {{count}} yêu cầu", { count: itemsCount })}</span>
    </div>
  );
}

interface RequestContentProps {
  isLoading: boolean;
  isProcessing: boolean;
  items: UnifiedApprovalItem[];
  selectedIds: Set<string>;
  setSelectedItem: (item: UnifiedApprovalItem) => void;
  t: TFunction;
  toggleSelect: (id: string) => void;
  onQuickApprove: (item: UnifiedApprovalItem) => Promise<void>;
  onQuickReject: (item: UnifiedApprovalItem) => void;
}

function RequestContent({
  isLoading,
  isProcessing,
  items,
  onQuickApprove,
  onQuickReject,
  selectedIds,
  setSelectedItem,
  t,
  toggleSelect,
}: RequestContentProps) {
  if (isLoading) {
    return (
      <div className="space-y-3" data-testid="inbox-skeleton-list">
        {Array.from({ length: 5 }, (_, index) => (
          <Skeleton className="h-20 w-full rounded-2xl" key={index} />
        ))}
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <div className="py-12" data-testid="inbox-empty-state">
        <EmptyState
          description={t(
            "approvals.inbox.empty_desc",
            "Không tìm thấy yêu cầu phê duyệt nào phù hợp với bộ lọc hiện tại",
          )}
          icon={<CheckCircle2 aria-hidden="true" className="h-12 w-12 text-emerald-500" />}
          title={t("approvals.inbox.empty_title", "Không có yêu cầu chờ xử lý")}
        />
      </div>
    );
  }

  return (
    <div className="space-y-3" data-testid="inbox-items-container">
      {items.map((item) => (
        <ApprovalItemRow
          isProcessing={isProcessing}
          isSelected={selectedIds.has(item.id)}
          item={item}
          key={item.id}
          onClick={setSelectedItem}
          onQuickApprove={onQuickApprove}
          onQuickReject={onQuickReject}
          onToggleSelect={toggleSelect}
        />
      ))}
    </div>
  );
}

export default function UnifiedInboxPage() {
  const { t } = useTranslation();
  const {
    items,
    pendingCounts,
    isLoading,
    filter,
    setFilter,
    selectedIds,
    toggleSelect,
    selectAll,
    clearSelection,
    isAllSelected,
    selectedItem,
    setSelectedItem,
    refetchAll,
  } = useUnifiedInbox();
  const { batchApprove, batchReject, isProcessing } = useBatchApproval();
  const [rejectingItems, setRejectingItems] = useState<UnifiedApprovalItem[]>([]);
  const [isRejectModalOpen, setIsRejectModalOpen] = useState(false);

  const handleBatchApprove = async () => {
    const selected = items.filter((item) => selectedIds.has(item.id) && item.status === "pending");
    if (selected.length === 0) return;
    const result = await batchApprove(selected);
    if (result.succeeded.length > 0) clearSelection();
  };

  const handleBatchRejectPrompt = () => {
    const selected = items.filter((item) => selectedIds.has(item.id) && item.status === "pending");
    if (selected.length === 0) return;
    setRejectingItems(selected);
    setIsRejectModalOpen(true);
  };

  const handleQuickApprove = async (item: UnifiedApprovalItem) => {
    await batchApprove([item]);
    if (selectedItem?.id === item.id) setSelectedItem(null);
  };

  const handleQuickRejectPrompt = (item: UnifiedApprovalItem) => {
    setRejectingItems([item]);
    setIsRejectModalOpen(true);
  };

  const handleConfirmRejection = async (reason: string) => {
    const result = await batchReject(rejectingItems, reason);
    if (result.succeeded.length === 0) return;
    setIsRejectModalOpen(false);
    setRejectingItems([]);
    clearSelection();
    if (selectedItem && rejectingItems.some((item) => item.id === selectedItem.id)) {
      setSelectedItem(null);
    }
  };

  return (
    <div className="animate-in fade-in space-y-6 pb-28 duration-500" data-testid="unified-inbox-page">
      <InboxHeader
        isLoading={isLoading}
        pendingCount={pendingCounts.all}
        refetchAll={refetchAll}
        t={t}
      />
      <FilterPanel filter={filter} pendingCounts={pendingCounts} setFilter={setFilter} t={t} />
      {filter.status === "pending" && items.length > 0 ? (
        <SelectionHeader
          clearSelection={clearSelection}
          isAllSelected={isAllSelected}
          itemsCount={items.length}
          selectAll={selectAll}
          t={t}
        />
      ) : null}
      <RequestContent
        isLoading={isLoading}
        isProcessing={isProcessing}
        items={items}
        onQuickApprove={handleQuickApprove}
        onQuickReject={handleQuickRejectPrompt}
        selectedIds={selectedIds}
        setSelectedItem={setSelectedItem}
        t={t}
        toggleSelect={toggleSelect}
      />
      <ApprovalBatchToolbar
        isProcessing={isProcessing}
        onApproveAll={handleBatchApprove}
        onClearSelection={clearSelection}
        onRejectAll={handleBatchRejectPrompt}
        selectedCount={selectedIds.size}
      />
      <ApprovalDetailDrawer
        isOpen={selectedItem !== null}
        isProcessing={isProcessing}
        item={selectedItem}
        onApprove={handleQuickApprove}
        onClose={() => setSelectedItem(null)}
        onReject={handleQuickRejectPrompt}
      />
      <RejectionReasonModal
        isOpen={isRejectModalOpen}
        isProcessing={isProcessing}
        itemCount={rejectingItems.length}
        onClose={() => {
          setIsRejectModalOpen(false);
          setRejectingItems([]);
        }}
        onConfirm={handleConfirmRejection}
      />
    </div>
  );
}
