import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Plus,
  Clock,
  CheckCircle,
  XCircle,
  ArrowRight,
  CalendarDays,
  Check,
  X,
} from "lucide-react";
import React, { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { TFunction } from "i18next";

import type { ShiftSwapsSearch } from "@/pages/shift-swaps/search-schema";

import ApprovalListLayout, { TabConfig } from "@/components/templates/approval-list-layout";
import Button from "@/components/ui/button";
import ResponsiveTable from "@/components/ui/responsive-table";
import StatusBadge from "@/components/ui/status-badge";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useAllEmployees } from "@/hooks/use-employees";
import { useSearchPatch } from "@/lib/state";
import { Route } from "@/routes/_authenticated/_workspace/shift-swaps";

import {
  getCurrentWorkspaceId,
} from "../lib/api";
import {
  fetchShiftSwaps,
  createShiftSwap,
  approveShiftSwap,
  fetchShiftSwapStats,
  ShiftSwapStatus,
  ShiftSwapRequest,
} from "../lib/api/shift-swaps";
import { ShiftSwapCreateForm } from "./shift-swaps/create-form";
import { ShiftSwapGuide } from "./shift-swaps/guide";
import { ShiftSwapListItem } from "./shift-swaps/list-item";
import { ShiftSwapStats } from "./shift-swaps/stats";
import { getTableHeaders } from "./shift-swaps/utils";

function ShiftSwapMobileCard({
  formatDate,
  onApprove,
  onReject,
  swap,
  t,
}: {
  formatDate: (d: string) => string;
  onApprove: (id: string) => void;
  onReject: (id: string) => void;
  swap: ShiftSwapRequest;
  t: TFunction;
}) {
  return (
    <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm transition-all active:scale-[0.99] dark:border-slate-800 dark:bg-slate-900">
      {/* Header: Date and status */}
      <div className="flex items-start justify-between gap-3">
        <div className="inline-flex items-center gap-2 rounded-xl border border-slate-100 bg-slate-50 px-4 py-2 shadow-inner dark:border-slate-800 dark:bg-slate-800">
          <CalendarDays className="text-primary-600 dark:text-primary-400" size={14} />
          <span className="text-xs font-bold uppercase tracking-tighter text-slate-700 dark:text-slate-200">
            {formatDate(swap.swap_date)}
          </span>
        </div>
        <StatusBadge
          variant={
            swap.status === "pending_admin"
              ? "warning"
              : swap.status === "approved"
                ? "success"
                : "danger"
          }
        >
          {t(`shift_swaps.status.${swap.status}`)}
        </StatusBadge>
      </div>

      {/* Swap details */}
      <div className="mt-4 space-y-3 border-t border-slate-100 pt-4 dark:border-slate-800">
        <div className="flex items-center gap-3">
          <div className="min-w-0 flex-1">
            <div className="truncate text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100">
              {swap.requester_name ?? "—"}
            </div>
            <div className="mt-1 text-[10px] font-bold text-primary-600 dark:text-primary-400">
              {swap.requester_shift_name ?? t("shift_swaps.day_off")}
            </div>
          </div>
          <ArrowRight className="shrink-0 text-slate-300 dark:text-slate-600" size={16} />
          <div className="min-w-0 flex-1 text-right">
            <div className="truncate text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100">
              {swap.target_name ?? "—"}
            </div>
            <div className="mt-1 text-[10px] font-bold text-primary-600 dark:text-primary-400">
              {swap.target_shift_name ?? t("shift_swaps.day_off")}
            </div>
          </div>
        </div>

        {/* Reason */}
        <Typography className="text-xs font-bold uppercase tracking-tight leading-relaxed text-slate-500 opacity-70 italic line-clamp-2 dark:text-slate-400" variant="p">
          {swap.reason ?? t("common.no_notes")}
        </Typography>
      </div>

      {/* Actions */}
      {swap.status === "pending_admin" && (
        <div className="mt-4 flex justify-end gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
          <Button
            className="p-3 bg-emerald-600 text-white hover:bg-emerald-700 rounded-xl shadow-lg shadow-emerald-100 transition-all active:scale-90"
            onClick={() => {
              onApprove(swap.id);
            }}
            title={t("common.approve")}
            type="button"
            variant="ghost"
          >
            <Check size={18} />
          </Button>
          <Button
            className="p-3 bg-white dark:bg-slate-900 text-rose-600 dark:text-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950 rounded-xl shadow-md border border-rose-100 dark:border-rose-800 transition-all active:scale-90"
            onClick={() => {
              onReject(swap.id);
            }}
            title={t("common.reject")}
            type="button"
            variant="ghost"
          >
            <X size={18} />
          </Button>
        </div>
      )}
    </div>
  );
}

type TabType = ShiftSwapsSearch["tab"];

export default function ShiftSwapsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  return <ShiftSwapsContent t={t} wsId={wsId} />;
}

function ShiftSwapsContent({ t, wsId }: { t: TFunction; wsId: string }) {
  const { formatDate } = useDateTimeFormatter();

  const search = Route.useSearch();
  const patchSearch = useSearchPatch<ShiftSwapsSearch>();
  const activeTab = search.tab;
  const setActiveTab = useCallback((tab: TabType) => { patchSearch({ tab }); }, [patchSearch]);

  const [showCreateForm, setShowCreateForm] = useState(false);
  const [showGuide, setShowGuide] = useState(false);

  const [requesterId, setRequesterId] = useState("");
  const [targetId, setTargetId] = useState("");
  const [swapDate, setSwapDate] = useState("");
  const [reason, setReason] = useState("");

  const { approveMutation, createMutation, isLoading, stats, swaps } = useShiftSwaps(wsId, activeTab, t, {
    onSuccessCreate: () => {
      setShowCreateForm(false);
      setRequesterId("");
      setTargetId("");
      setSwapDate("");
      setReason("");
    },
  });

  const { employees } = useAllEmployees(wsId);

  const tabs: TabConfig[] = [
    { id: "pending_admin", label: t("shift_swaps.status.pending_admin"), icon: <Clock size={14} />, colorScheme: "amber" },
    { id: "approved", label: t("shift_swaps.status.approved"), icon: <CheckCircle size={14} />, colorScheme: "emerald" },
    { id: "rejected", label: t("shift_swaps.status.rejected"), icon: <XCircle size={14} />, colorScheme: "rose" },
    { id: "cancelled", label: t("shift_swaps.status.cancelled"), icon: <XCircle size={14} />, colorScheme: "slate" },
  ];

  return (
    <ResponsiveTable
      data={swaps}
      emptyState={{
        icon: <Clock className="w-20 h-20" />,
        title: t("shift_swaps.messages.no_data"),
      }}
      getCardKey={(swap) => swap.id}
      isLoading={isLoading}
      renderCard={(swap) => (
        <ShiftSwapMobileCard
          formatDate={formatDate}
          onApprove={(id) => {
            approveMutation.mutate({ id, status: "approved" });
          }}
          onReject={(id) => {
            approveMutation.mutate({ id, status: "rejected" });
          }}
          swap={swap}
          t={t}
        />
      )}
      renderDesktopTable={() => (
        <ApprovalListLayout
          activeTab={activeTab}
          banner={
            <>
              <ShiftSwapGuide setShowGuide={setShowGuide} showGuide={showGuide} t={t} />
              {stats ? (
                <ShiftSwapStats
                  activeTab={activeTab}
                  onTabChange={(tab) => {
                    setActiveTab(tab as TabType);
                  }}
                  stats={stats}
                  t={t}
                />
              ) : null}
              {showCreateForm ? (
                <ShiftSwapCreateForm
                  employees={employees}
                  handleCreate={() => {
                    createMutation.mutate({ requester_id: requesterId, target_id: targetId, swap_date: swapDate, reason: reason || undefined });
                  }}
                  isPending={createMutation.isPending}
                  reason={reason}
                  requesterId={requesterId}
                  setReason={setReason}
                  setRequesterId={setRequesterId}
                  setShowCreateForm={setShowCreateForm}
                  setSwapDate={setSwapDate}
                  setTargetId={setTargetId}
                  swapDate={swapDate}
                  t={t}
                  targetId={targetId}
                />
              ) : null}
            </>
          }
          emptyMessage={t("shift_swaps.messages.no_data")}
          headerActions={
            <Button icon={<Plus size={18} />}
              onClick={() => {
                setShowCreateForm(!showCreateForm);
              }}
              variant="primary"
            >
              {t("shift_swaps.create_btn")}
            </Button>
          }
          isEmpty={swaps.length === 0}
          isLoading={isLoading}
          onTabChange={(id) => {
            setActiveTab(id as TabType);
          }}
          subtitle={t("shift_swaps.subtitle")}
          tableHeaders={getTableHeaders(t)}
          tabs={tabs}
          title={t("shift_swaps.title")}
        >
          {swaps.map((swap) => (
            <ShiftSwapListItem
              formatDate={formatDate}
              key={swap.id}
              onApprove={(id) => {
                approveMutation.mutate({ id, status: "approved" });
              }}
              onReject={(id) => {
                approveMutation.mutate({ id, status: "rejected" });
              }}
              swap={swap}
              t={t}
            />
          ))}
        </ApprovalListLayout>
      )}
    />
  );
}

function useShiftSwaps(wsId: string, activeTab: TabType, t: TFunction, options: { onSuccessCreate: () => void }) {
  const queryClient = useQueryClient();
  const toast = useToast();

  const { data: swaps = [], isLoading } = useQuery<ShiftSwapRequest[]>({
    queryKey: ["shift-swaps", wsId, activeTab],
    queryFn: () => fetchShiftSwaps(wsId, activeTab),
    enabled: !!wsId,
  });

  const { data: stats } = useQuery({
    queryKey: ["shift-swap-stats", wsId],
    queryFn: () => fetchShiftSwapStats(wsId),
    enabled: !!wsId,
  });

  const createMutation = useMutation({
    mutationFn: (data: { requester_id: string; target_id: string; swap_date: string; reason?: string }) => createShiftSwap(wsId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["shift-swaps"] });
      toast.success(t("shift_swaps.messages.create_success"));
      options.onSuccessCreate();
    },
    onError: () => {
      toast.error(t("shift_swaps.messages.create_error"));
    },
  });

  const approveMutation = useMutation({
    mutationFn: (data: { id: string; status: ShiftSwapStatus }) => approveShiftSwap(wsId, data.id, { status: data.status }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["shift-swaps"] });
      void queryClient.invalidateQueries({ queryKey: ["employee-rosters"] });
      void queryClient.invalidateQueries({ queryKey: ["workspace-schedules"] });
      toast.success(t("common.success"));
    },
  });

  return { approveMutation, createMutation, isLoading, stats, swaps };
}
