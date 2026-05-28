import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import type { WorkLocation, CreateWorkLocationRequest, UpdateWorkLocationRequest } from "@/lib/api-generated/models";
import { useCreateLocation, useUpdateLocation } from "../hooks/use-locations";
import Sheet from "@/components/ui/sheet";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useToast } from "@/components/ui/toast";
import { Combobox } from "@/components/ui/combobox";

// Simple list of timezones for the combobox
const TIMEZONES = Intl.supportedValuesOf("timeZone").map(tz => ({
  value: tz,
  label: tz,
}));

interface Props {
  isOpen: boolean;
  onClose: () => void;
  editingLocation: WorkLocation | null;
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
            placeholder="e.g. New York Office"
            value={name}
            onChange={e => setName(e.target.value)}
          />
        </div>

        <div className="space-y-2">
          <label className={LABEL_BASE}>{t("settings.locations.address", { defaultValue: "Address" })} (Optional)</label>
          <textarea 
            className="w-full px-6 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl focus:ring-4 focus:ring-primary-500/10 outline-none transition-all font-medium text-sm shadow-sm min-h-[100px]"
            placeholder="Full address details"
            value={address}
            onChange={e => setAddress(e.target.value)}
          />
        </div>

        <div className="space-y-2">
          <label className={LABEL_BASE}>{t("settings.locations.timezone", { defaultValue: "Timezone" })}</label>
          <Combobox 
            className="bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 h-14"
            options={TIMEZONES}
            value={timezone}
            onChange={setTimezone}
            placeholder="Search timezone..."
          />
        </div>

        {(!editingLocation?.is_headquarters) && (
          <div className="flex items-center gap-4 p-4 bg-slate-50 dark:bg-slate-800/50 rounded-2xl border border-slate-100 dark:border-slate-800">
            <input 
              type="checkbox" 
              id="is-hq"
              checked={isHeadquarters}
              onChange={e => setIsHeadquarters(e.target.checked)}
              className="w-5 h-5 rounded border-slate-300 text-primary-600 focus:ring-primary-500"
            />
            <div className="flex flex-col">
              <label htmlFor="is-hq" className="font-bold text-sm text-slate-700 dark:text-slate-300">
                {t("settings.locations.set_as_hq", { defaultValue: "Set as Headquarters" })}
              </label>
              <Typography variant="p" className="text-xs text-slate-500">
                {t("settings.locations.set_as_hq_desc", { defaultValue: "This will demote the current headquarters." })}
              </Typography>
            </div>
          </div>
        )}

        <div className="pt-6 border-t border-slate-100 dark:border-slate-800 flex justify-end gap-3">
          <Button variant="ghost" onClick={onClose} disabled={isPending}>
            {t("common.cancel")}
          </Button>
          <Button variant="primary" onClick={handleSave} loading={isPending}>
            {t("common.save")}
          </Button>
        </div>
      </div>
    </Sheet>
  );
}
