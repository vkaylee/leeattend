import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Plus,
  Clock,
  CheckCircle,
  XCircle,
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { TFunction } from "i18next";

import ApprovalListLayout, { TabConfig } from "@/components/templates/approval-list-layout";
import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useAllEmployees } from "@/hooks/use-employees";

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

type TabType = "pending_target" | "pending_admin" | "approved" | "rejected" | "cancelled";

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
  const [activeTab, setActiveTab] = useState<TabType>("pending_admin");
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
        <Button
          icon={<Plus size={18} />}
          onClick={() => {
            setShowCreateForm(!showCreateForm);
          }}
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
