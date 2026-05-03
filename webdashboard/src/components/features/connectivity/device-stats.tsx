import { User, ClipboardList, Terminal, Globe, Clock } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { useTickingDeviceTime } from "@/hooks/use-ticking-device-time";
import { DeviceInfo } from "@/lib/api";

interface DeviceTimeTickerProps {
  deviceTime?: string;
  lastSync?: string;
}

function DeviceTimeTicker({ deviceTime, lastSync }: DeviceTimeTickerProps) {
  const { t } = useTranslation();
  const tickingTime = useTickingDeviceTime(deviceTime, lastSync);

  return (
    <div className="flex flex-col gap-1 min-w-[150px]">
      <span className="text-[8px] font-bold text-slate-400 uppercase tracking-widest">{t("devices.info.device_time")}</span>
      <div className="flex items-center gap-1.5 text-primary-600 dark:text-primary-400">
        <Clock size={12} />
        <span className="text-[10px] font-mono font-black tracking-tighter">{tickingTime ?? "--:--:--"}</span>
      </div>
    </div>
  );
}

interface DeviceStatsProps {
  displayInfo: DeviceInfo | null;
}

interface StatBlockProps {
  icon: React.ReactNode;
  label: string;
  value: React.ReactNode;
}

function StatBlock({ icon, label, value }: StatBlockProps) {
  return (
    <div className="flex flex-col gap-1 min-w-[80px]">
      <span className="text-[8px] font-bold text-slate-400 uppercase tracking-widest">{label}</span>
      <div className="flex items-center gap-1.5">
        {icon}
        <span className="text-sm font-black text-slate-700 dark:text-slate-200">
          {value}
        </span>
      </div>
    </div>
  );
}

function CapacityBlock({ capacity, count, icon, label, valueColorClass }: { capacity?: number; count?: number; icon: React.ReactNode; label: string; valueColorClass?: string }) {
  const value = count !== undefined ? (
    <>
      <span className={valueColorClass}>{count.toLocaleString()}</span>
      {typeof capacity === 'number' && capacity > 0 && (
        <span className="text-slate-400 font-bold text-[10px]"> / {capacity.toLocaleString()}</span>
      )}
    </>
  ) : "--";

  return (
    <div className="flex flex-col gap-1 min-w-[80px]">
      <span className="text-[8px] font-bold text-slate-400 uppercase tracking-widest">{label}</span>
      <div className="flex items-center gap-1.5">
        {icon}
        <span className="text-sm font-black text-slate-700 dark:text-slate-200">
          {value}
        </span>
      </div>
    </div>
  );
}

export function DeviceStats({ displayInfo }: DeviceStatsProps) {
  const { t } = useTranslation();

  if (!displayInfo) {
    return (
      <div className="flex flex-wrap gap-y-6 gap-x-8">
        <CapacityBlock icon={<User className="text-slate-300" size={12} />} label={t("devices.info.users_capacity")} />
        <CapacityBlock icon={<ClipboardList className="text-amber-400" size={12} />} label={t("devices.info.logs_capacity")} valueColorClass="text-amber-600" />
        <DeviceTimeTicker />
        <StatBlock icon={<Terminal className="text-slate-300" size={12} />} label={t("devices.info.serial")} value="N/A" />
        <StatBlock icon={<Globe className="text-slate-300" size={12} />} label={t("devices.info.mac")} value="N/A" />
      </div>
    );
  }

  const { users_capacity, users_count, logs_capacity, logs_count, device_time, last_sync, serial, mac, hw_name, firmware } = displayInfo;

  return (
    <div className="flex flex-wrap gap-y-6 gap-x-8">
      <CapacityBlock
        capacity={users_capacity}
        count={users_count}
        icon={<User className="text-slate-300" size={12} />}
        label={t("devices.info.users_capacity")}
      />

      <CapacityBlock
        capacity={logs_capacity}
        count={logs_count}
        icon={<ClipboardList className="text-amber-400" size={12} />}
        label={t("devices.info.logs_capacity")}
        valueColorClass="text-amber-600"
      />

      <DeviceTimeTicker deviceTime={device_time} lastSync={last_sync} />

      <StatBlock
        icon={<Terminal className="text-slate-300" size={12} />}
        label={t("devices.info.serial")}
        value={<span className="text-[10px] font-mono font-bold text-slate-600 dark:text-slate-400 truncate">{serial ?? "N/A"}</span>}
      />

      <StatBlock
        icon={<Globe className="text-slate-300" size={12} />}
        label={t("devices.info.mac")}
        value={<span className="text-[10px] font-mono font-bold text-slate-600 dark:text-slate-400 truncate">{mac ?? "N/A"}</span>}
      />

      <div className="flex flex-col gap-1 min-w-[120px]">
        <span className="text-[8px] font-bold text-slate-400 uppercase tracking-widest">{t("devices.info.hw_fw")}</span>
        <div className="flex flex-wrap gap-1">
          <span className="text-[9px] font-black px-1.5 py-0.5 bg-slate-100 dark:bg-slate-800 rounded text-slate-500 border border-slate-200/50 dark:border-slate-700/50">{hw_name ?? "N/A"}</span>
          <span className="text-[9px] font-black px-1.5 py-0.5 bg-primary-50 dark:bg-primary-950 text-primary-600 rounded border border-primary-100/50">v{firmware ?? "?.?"}</span>
        </div>
      </div>
    </div>
  );
}
