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
import React, { useState } from "react";
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
import { useUnifiedInbox } from "./hooks/use-unified-inbox";
import {
  ApprovalItemStatus,
  ApprovalType,
  UnifiedApprovalItem,
} from "./types/unified-approvals";

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

  // Rejection modal state
  const [rejectingItems, setRejectingItems] = useState<UnifiedApprovalItem[]>(
    []
  );
  const [isRejectModalOpen, setIsRejectModalOpen] = useState(false);

  // ── Handlers ──────────────────────────────────────────────────────────
  const handleBatchApprove = async () => {
    const selected = items.filter(
      (i) => selectedIds.has(i.id) && i.status === "pending"
    );
    if (selected.length === 0) return;
    const res = await batchApprove(selected);
    if (res.succeeded.length > 0) {
      clearSelection();
    }
  };

  const handleBatchRejectPrompt = () => {
    const selected = items.filter(
      (i) => selectedIds.has(i.id) && i.status === "pending"
    );
    if (selected.length === 0) return;
    setRejectingItems(selected);
    setIsRejectModalOpen(true);
  };

  const handleQuickApprove = async (item: UnifiedApprovalItem) => {
    await batchApprove([item]);
    if (selectedItem?.id === item.id) {
      setSelectedItem(null);
    }
  };

  const handleQuickRejectPrompt = (item: UnifiedApprovalItem) => {
    setRejectingItems([item]);
    setIsRejectModalOpen(true);
  };

  const handleConfirmRejection = async (reason: string) => {
    const res = await batchReject(rejectingItems, reason);
    if (res.succeeded.length > 0) {
      setIsRejectModalOpen(false);
      setRejectingItems([]);
      clearSelection();
      if (selectedItem && rejectingItems.some((i) => i.id === selectedItem.id)) {
        setSelectedItem(null);
      }
    }
  };

  const typeTabs: Array<{
    id: ApprovalType | "all";
    label: string;
    count: number;
    icon: React.ReactNode;
  }> = [
    {
      id: "all",
      label: t("common.all", "Tất cả"),
      count: pendingCounts.all,
      icon: <Inbox className="h-4 w-4" />,
    },
    {
      id: "leave",
      label: t("approvals.tabs.leave", "Nghỉ phép"),
      count: pendingCounts.leave,
      icon: <Calendar className="h-4 w-4" />,
    },
    {
      id: "overtime",
      label: t("approvals.tabs.overtime", "Tăng ca"),
      count: pendingCounts.overtime,
      icon: <Clock className="h-4 w-4" />,
    },
    {
      id: "adjustment",
      label: t("approvals.tabs.adjustment", "Giải trình"),
      count: pendingCounts.adjustment,
      icon: <FileEdit className="h-4 w-4" />,
    },
    {
      id: "shift_swap",
      label: t("approvals.tabs.shift_swap", "Đổi ca"),
      count: pendingCounts.shift_swap,
      icon: <ArrowLeftRight className="h-4 w-4" />,
    },
  ];

  const statusTabs: Array<{ id: ApprovalItemStatus | "all"; label: string }> = [
    { id: "pending", label: t("approvals.status.pending", "Chờ duyệt") },
    { id: "approved", label: t("approvals.status.approved", "Đã duyệt") },
    { id: "rejected", label: t("approvals.status.rejected", "Đã từ chối") },
    { id: "all", label: t("approvals.status.all_status", "Tất cả trạng thái") },
  ];

  return (
    <div
      className="space-y-6 pb-28 animate-in fade-in duration-500"
      data-testid="unified-inbox-page"
    >
      {/* ── Page Header ──────────────────────────────────────────────── */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div className="space-y-1">
          <div className="flex items-center gap-3">
            <Typography className="text-2xl font-black tracking-tight" variant="h2">
              {t("approvals.inbox.title", "Hộp thư Phê duyệt")}
            </Typography>
            {pendingCounts.all > 0 && (
              <span
                className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-bold bg-amber-100 text-amber-800 dark:bg-amber-950/80 dark:text-amber-300 border border-amber-200/80 dark:border-amber-800/80"
                data-testid="pending-total-badge"
              >
                {pendingCounts.all} {t("approvals.inbox.pending_badge", "chờ duyệt")}
              </span>
            )}
          </div>
          <p className="text-xs text-slate-500 dark:text-slate-400">
            {t(
              "approvals.inbox.subtitle",
              "Quản lý và phê duyệt tập trung các yêu cầu Nghỉ phép, Tăng ca, Giải trình công và Đổi ca"
            )}
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Button
            aria-label={t("common.refresh", "Làm mới dữ liệu")}
            className="rounded-xl border border-slate-200 dark:border-slate-800 text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800"
            data-testid="inbox-refresh-btn"
            disabled={isLoading}
            onClick={() => void refetchAll()}
            size="sm"
            variant="outline"
          >
            <RefreshCw
              className={cn("h-4 w-4 mr-1.5", isLoading && "animate-spin")}
            />
            {t("common.refresh", "Làm mới")}
          </Button>
        </div>
      </div>

      {/* ── Filter & Search Control Panel ───────────────────────────── */}
      <div className="space-y-4 rounded-3xl bg-white dark:bg-slate-900 border border-slate-200/80 dark:border-slate-800/80 p-4 shadow-sm">
        {/* Type Category Tabs */}
        <div
          aria-label={t("approvals.type_tabs_label", "Loại yêu cầu")}
          className="flex items-center gap-2 overflow-x-auto pb-1 scrollbar-none"
          role="tablist"
        >
          {typeTabs.map((tab) => {
            const isActive = filter.type === tab.id;
            return (
              <Button
                aria-selected={isActive}
                className={cn(
                  "flex items-center gap-2 px-3.5 py-2 rounded-2xl text-xs font-bold transition-all shrink-0",
                  isActive
                    ? "bg-primary-600 text-white shadow-md"
                    : "bg-slate-100 dark:bg-slate-800/60 text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100"
                )}
                data-testid={`type-tab-${tab.id}`}
                key={tab.id}
                onClick={() => setFilter({ type: tab.id })}
                role="tab"
                size="sm"
                variant="ghost"
              >
                {tab.icon}
                <span>{tab.label}</span>
                {tab.count > 0 && (
                  <span
                    className={cn(
                      "px-1.5 py-0.2 rounded-full text-[10px] font-black",
                      isActive
                        ? "bg-white/20 text-white"
                        : "bg-amber-100 text-amber-800 dark:bg-amber-950 dark:text-amber-300"
                    )}
                  >
                    {tab.count}
                  </span>
                )}
              </Button>
            );
          })}
        </div>

        {/* Status Sub-filter and Search Input */}
        <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3 pt-2 border-t border-slate-100 dark:border-slate-800">
          {/* Status Tabs */}
          <div className="flex items-center gap-1.5 overflow-x-auto">
            {statusTabs.map((st) => {
              const isActive = filter.status === st.id;
              return (
                <Button
                  className={cn(
                    "px-3 py-1.5 rounded-xl text-xs font-semibold transition-all",
                    isActive
                      ? "bg-slate-900 dark:bg-slate-100 text-white dark:text-slate-900 shadow-sm"
                      : "text-slate-500 hover:text-slate-800 dark:hover:text-slate-200"
                  )}
                  data-testid={`status-tab-${st.id}`}
                  key={st.id}
                  onClick={() => setFilter({ status: st.id })}
                  size="sm"
                  variant="ghost"
                >
                  {st.label}
                </Button>
              );
            })}
          </div>

          {/* Quick Search Input */}
          <div className="relative min-w-[240px] sm:w-72">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-slate-400" />
            <input
              aria-label={t("approvals.search_placeholder", "Tìm kiếm nhân sự, lý do...")}
              className="w-full h-9 pl-9 pr-8 rounded-xl bg-slate-50 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700 text-xs text-slate-900 dark:text-slate-100 placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500"
              data-testid="inbox-search-input"
              onChange={(e) => setFilter({ searchQuery: e.target.value })}
              placeholder={t(
                "approvals.search_placeholder",
                "Tìm kiếm nhân sự, lý do..."
              )}
              type="text"
              value={filter.searchQuery || ""}
            />
            {filter.searchQuery && (
              <Button
                aria-label={t("common.clear", "Xóa tìm kiếm")}
                className="absolute right-2 top-1/2 -translate-y-1/2 h-5 w-5 p-0 text-slate-400 hover:text-slate-600 rounded-full"
                onClick={() => setFilter({ searchQuery: "" })}
                size="sm"
                variant="ghost"
              >
                <X className="h-3.5 w-3.5" />
              </Button>
            )}
          </div>
        </div>
      </div>

      {/* ── Selection Action Header (if pending filter) ─────────────── */}
      {filter.status === "pending" && items.length > 0 && (
        <div className="flex items-center justify-between px-2 text-xs font-semibold text-slate-500 dark:text-slate-400">
          <label className="flex items-center gap-2 cursor-pointer select-none">
            <input
              aria-label={t("approvals.select_all", "Chọn tất cả đơn chờ duyệt")}
              checked={isAllSelected}
              className="h-4 w-4 rounded border-slate-300 text-primary-600 focus:ring-primary-500 cursor-pointer"
              data-testid="select-all-checkbox"
              onChange={() => {
                if (isAllSelected) clearSelection();
                else selectAll();
              }}
              type="checkbox"
            />
            <span>
              {isAllSelected
                ? t("approvals.unselect_all", "Bỏ chọn tất cả")
                : t("approvals.select_all", "Chọn tất cả đơn chờ duyệt")}
            </span>
          </label>

          <span>
            {t("approvals.showing_count", "Hiển thị {{count}} yêu cầu", {
              count: items.length,
            })}
          </span>
        </div>
      )}

      {/* ── Request List or Loading / Empty States ──────────────────── */}
      {isLoading ? (
        <div className="space-y-3" data-testid="inbox-skeleton-list">
          {Array.from({ length: 5 }).map((_, i) => (
            <Skeleton className="h-20 w-full rounded-2xl" key={i} />
          ))}
        </div>
      ) : items.length === 0 ? (
        <div className="py-12" data-testid="inbox-empty-state">
          <EmptyState
            description={t(
              "approvals.inbox.empty_desc",
              "Không tìm thấy yêu cầu phê duyệt nào phù hợp với bộ lọc hiện tại"
            )}
            icon={<CheckCircle2 className="h-12 w-12 text-emerald-500" />}
            title={t("approvals.inbox.empty_title", "Không có yêu cầu chờ xử lý")}
          />
        </div>
      ) : (
        <div className="space-y-3" data-testid="inbox-items-container">
          {items.map((item) => (
            <ApprovalItemRow
              isProcessing={isProcessing}
              isSelected={selectedIds.has(item.id)}
              item={item}
              key={item.id}
              onClick={setSelectedItem}
              onQuickApprove={handleQuickApprove}
              onQuickReject={handleQuickRejectPrompt}
              onToggleSelect={toggleSelect}
            />
          ))}
        </div>
      )}

      {/* ── Floating Batch Action Toolbar ────────────────────────────── */}
      <ApprovalBatchToolbar
        isProcessing={isProcessing}
        onApproveAll={handleBatchApprove}
        onClearSelection={clearSelection}
        onRejectAll={handleBatchRejectPrompt}
        selectedCount={selectedIds.size}
      />

      {/* ── Slide-over Detail Drawer ─────────────────────────────────── */}
      <ApprovalDetailDrawer
        isOpen={selectedItem !== null}
        isProcessing={isProcessing}
        item={selectedItem}
        onApprove={handleQuickApprove}
        onClose={() => setSelectedItem(null)}
        onReject={handleQuickRejectPrompt}
      />

      {/* ── Rejection Reason Modal ───────────────────────────────────── */}
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
