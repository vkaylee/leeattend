import { Cpu, Globe, MoreVertical, RefreshCw, Pencil, Trash2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Typography } from "@/components/ui/typography";
import { Device, DeviceStatus } from "@/lib/api";
import { cn } from "@/lib/utils";

interface DeviceTopBarProps {
  device: Device;
  handleRefresh: () => void;
  isRefreshing: boolean;
  onEdit?: (device: Device) => void;
  setShowDeleteModal: (show: boolean) => void;
  status?: DeviceStatus;
}

export function DeviceTopBar({
  device,
  status,
  isRefreshing,
  handleRefresh,
  onEdit,
  setShowDeleteModal,
}: DeviceTopBarProps) {
  const { t } = useTranslation();
  const isOnline = status?.status === "online";

  return (
    <div className="flex items-center justify-between mb-4 pb-4 border-b border-slate-50 dark:border-slate-800/50">
      <div className="flex items-center gap-3 min-w-0">
        <div className={cn(
          "w-10 h-10 rounded-xl flex items-center justify-center border shadow-inner shrink-0",
          isOnline
            ? "bg-emerald-50 dark:bg-emerald-950/50 text-emerald-600 border-emerald-100 dark:border-emerald-800"
            : "bg-slate-100 dark:bg-slate-800 text-slate-400 border-slate-200 dark:border-slate-700"
        )}>
          <Cpu className={cn(isOnline && "animate-pulse")} size={20} />
        </div>
        <div className="min-w-0">
          <Typography className="font-black text-slate-700 dark:text-slate-200 text-sm uppercase tracking-tight truncate" variant="h3">
            {device.name}
          </Typography>
          <div className="flex items-center gap-2 text-[10px] font-mono font-bold text-slate-400">
            <Globe size={10} /> {device.ip}
          </div>
        </div>
      </div>

      <div className="flex items-center gap-3">
        <span className={cn(
          "px-2.5 py-0.5 rounded-full text-[8px] font-black uppercase tracking-widest border",
          isOnline
            ? "bg-emerald-500 text-white border-transparent"
            : "bg-slate-100 dark:bg-slate-800 text-slate-500 border-slate-200 dark:border-slate-700"
        )}>
          {isOnline ? "Online" : "Offline"}
        </span>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              className="h-auto w-auto p-2 rounded-lg text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors shadow-none"
              data-testid={`device-menu-${device.id}`}
              icon={<MoreVertical size={16} />}
              size="icon"
              variant="ghost"
            />
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-52 p-2 rounded-xl shadow-2xl">
            <DropdownMenuItem className="gap-3 p-3 rounded-lg font-bold text-xs uppercase" data-testid={`device-refresh-${device.id}`} disabled={isRefreshing} onSelect={handleRefresh}>
              <RefreshCw className={cn(isRefreshing && "animate-spin")} size={14} />
              {isRefreshing ? t("common.loading") : t("common.refresh")}
            </DropdownMenuItem>
            {!!onEdit && (
              <DropdownMenuItem className="gap-3 p-3 rounded-lg font-bold text-xs uppercase" data-testid={`device-edit-${device.id}`} onSelect={() => { onEdit(device); }}>
                <Pencil size={14} /> {t("devices.table.edit")}
              </DropdownMenuItem>
            )}
            <DropdownMenuItem className="gap-3 p-3 rounded-lg font-bold text-xs uppercase text-rose-600 focus:text-rose-600" data-testid={`device-delete-${device.id}`} onSelect={() => { setShowDeleteModal(true); }}>
              <Trash2 size={14} /> {t("devices.table.delete")}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  );
}

