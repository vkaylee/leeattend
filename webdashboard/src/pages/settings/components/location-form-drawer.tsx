import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import type { WorkLocation, CreateWorkLocationRequest, UpdateWorkLocationRequest } from "@/lib/api-generated/models";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import Sheet from "@/components/ui/sheet";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";

import { useCreateLocation, useUpdateLocation } from "../hooks/use-locations";

// Simple list of timezones for the combobox
const TIMEZONES = Intl.supportedValuesOf("timeZone").map(tz => ({
  value: tz,
  label: tz,
}));

interface Props {
  editingLocation: WorkLocation | null;
  isOpen: boolean;
  onClose: () => void;
  wsId: string;
}

export default function LocationFormDrawer({ isOpen, onClose, editingLocation, wsId }: Props) {
  const { t } = useTranslation();
  const createMutation = useCreateLocation(wsId);
  const updateMutation = useUpdateLocation(wsId);
  const toast = useToast();
  
  const [name, setName] = useState("");
  const [address, setAddress] = useState("");
  const [timezone, setTimezone] = useState("UTC");
  const [isHeadquarters, setIsHeadquarters] = useState(false);

  useEffect(() => {
    if (isOpen) {
      if (editingLocation) {
        setName(editingLocation.name);
        setAddress(editingLocation.address || "");
        setTimezone(editingLocation.timezone);
        setIsHeadquarters(editingLocation.is_headquarters);
      } else {
        setName("");
        setAddress("");
        setTimezone(Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC");
        setIsHeadquarters(false);
      }
    }
  }, [isOpen, editingLocation]);

  const handleSave = () => {
    if (!name.trim()) {
      toast.error(t("settings.locations.error_name_required", { defaultValue: "Name is required" }));
      return;
    }

    if (editingLocation) {
      const payload: UpdateWorkLocationRequest = {
        name,
        address: address || undefined,
        timezone,
      };
      
      // Only send is_headquarters if we're making it HQ (backend handles demoting others)
      // If it's already HQ, we can't demote it directly (someone else must be promoted)
      if (isHeadquarters && !editingLocation.is_headquarters) {
        payload.is_headquarters = true;
      }

      updateMutation.mutate({ id: editingLocation.id, data: payload }, {
        onSuccess: () => {
          toast.success(t("settings.locations.updated", { defaultValue: "Location updated" }));
          onClose();
        },
        onError: (err: any) => {
          toast.error(err.message || "Failed to update location");
        }
      });
    } else {
      const payload: CreateWorkLocationRequest = {
        name,
        address: address || undefined,
        timezone,
        is_headquarters: isHeadquarters,
      };

      createMutation.mutate(payload, {
        onSuccess: () => {
          toast.success(t("settings.locations.created", { defaultValue: "Location created" }));
          onClose();
        },
        onError: (err: any) => {
          toast.error(err.message || "Failed to create location");
        }
      });
    }
  };

  const isPending = createMutation.isPending || updateMutation.isPending;
  const LABEL_BASE = "block text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-2";

  return (
    <Sheet 
      isOpen={isOpen} 
      onClose={onClose} 
      title={editingLocation ? t("settings.locations.edit", { defaultValue: "Edit Location" }) : t("settings.locations.add", { defaultValue: "Add Location" })}
    >
      <div className="space-y-6">
        <div className="space-y-2">
          <label className={LABEL_BASE}>{t("settings.locations.name", { defaultValue: "Name" })}</label>
          <input 
            className="w-full px-6 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl focus:ring-4 focus:ring-primary-500/10 outline-none transition-all font-bold text-sm tracking-tight shadow-sm"
            onChange={e => { setName(e.target.value); }}
            placeholder="e.g. New York Office"
            value={name}
          />
        </div>

        <div className="space-y-2">
          <label className={LABEL_BASE}>{t("settings.locations.address", { defaultValue: "Address" })} (Optional)</label>
          <textarea 
            className="w-full px-6 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl focus:ring-4 focus:ring-primary-500/10 outline-none transition-all font-medium text-sm shadow-sm min-h-[100px]"
            onChange={e => { setAddress(e.target.value); }}
            placeholder="Full address details"
            value={address}
          />
        </div>

        <div className="space-y-2">
          <label className={LABEL_BASE}>{t("settings.locations.timezone", { defaultValue: "Timezone" })}</label>
          <Combobox 
            className="bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 h-14"
            onChange={setTimezone}
            options={TIMEZONES}
            placeholder="Search timezone..."
            value={timezone}
          />
        </div>

        {(!editingLocation?.is_headquarters) && (
          <div className="flex items-center gap-4 p-4 bg-slate-50 dark:bg-slate-800/50 rounded-2xl border border-slate-100 dark:border-slate-800">
            <input 
              checked={isHeadquarters} 
              className="w-5 h-5 rounded border-slate-300 text-primary-600 focus:ring-primary-500"
              id="is-hq"
              onChange={e => { setIsHeadquarters(e.target.checked); }}
              type="checkbox"
            />
            <div className="flex flex-col">
              <label className="font-bold text-sm text-slate-700 dark:text-slate-300" htmlFor="is-hq">
                {t("settings.locations.set_as_hq", { defaultValue: "Set as Headquarters" })}
              </label>
              <Typography className="text-xs text-slate-500" variant="p">
                {t("settings.locations.set_as_hq_desc", { defaultValue: "This will demote the current headquarters." })}
              </Typography>
            </div>
          </div>
        )}

        <div className="pt-6 border-t border-slate-100 dark:border-slate-800 flex justify-end gap-3">
          <Button disabled={isPending} onClick={onClose} variant="ghost">
            {t("common.cancel")}
          </Button>
          <Button loading={isPending} onClick={handleSave} variant="primary">
            {t("common.save")}
          </Button>
        </div>
      </div>
    </Sheet>
  );
}
