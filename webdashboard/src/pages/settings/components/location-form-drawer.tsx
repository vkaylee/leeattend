import { useState } from "react";
import { useTranslation } from "react-i18next";

import type { WorkLocation, CreateWorkLocationRequest, UpdateWorkLocationRequest } from "@/lib/api/locations";

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

  return (
    <Sheet 
      isOpen={isOpen} 
      onClose={onClose} 
      title={editingLocation ? t("settings.locations.edit", { defaultValue: "Edit Location" }) : t("settings.locations.add", { defaultValue: "Add Location" })}
    >
      {isOpen ? (
        <LocationFormContent 
          editingLocation={editingLocation}
          onClose={onClose}
          wsId={wsId}
        />
      ) : null}
    </Sheet>
  );
}

function getInitialValues(editingLocation: WorkLocation | null) {
  return {
    name: editingLocation?.name ?? "",
    address: editingLocation?.address ?? "",
    timezone: editingLocation?.timezone ?? (Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC"),
    isHeadquarters: editingLocation?.is_headquarters ?? false,
  };
}

interface ContentProps {
  editingLocation: WorkLocation | null;
  onClose: () => void;
  wsId: string;
}

function LocationFormContent({ editingLocation, onClose, wsId }: ContentProps) {
  const { t } = useTranslation();
  const createMutation = useCreateLocation(wsId);
  const updateMutation = useUpdateLocation(wsId);
  const toast = useToast();
  
  const initialValues = getInitialValues(editingLocation);
  const [name, setName] = useState(initialValues.name);
  const [address, setAddress] = useState(initialValues.address);
  const [timezone, setTimezone] = useState(initialValues.timezone);
  const [isHeadquarters, setIsHeadquarters] = useState(initialValues.isHeadquarters);

  const handleSave = () => {
    if (!name.trim()) {
      toast.error(t("settings.locations.error_name_required", { defaultValue: "Name is required" }));
      return;
    }

    if (editingLocation) {
      const payload: UpdateWorkLocationRequest = {
        name,
        address: address.trim() !== "" ? address.trim() : undefined,
        timezone,
      };
      
      if (isHeadquarters && !editingLocation.is_headquarters) {
        payload.is_headquarters = true;
      }

      updateMutation.mutate({ id: editingLocation.id, data: payload }, {
        onSuccess: () => {
          toast.success(t("settings.locations.updated", { defaultValue: "Location updated" }));
          onClose();
        },
        onError: (err: Error) => {
          toast.error(err.message);
        }
      });
    } else {
      const payload: CreateWorkLocationRequest = {
        name,
        address: address.trim() !== "" ? address.trim() : undefined,
        timezone,
        is_headquarters: isHeadquarters,
      };

      createMutation.mutate(payload, {
        onSuccess: () => {
          toast.success(t("settings.locations.created", { defaultValue: "Location created" }));
          onClose();
        },
        onError: (err: Error) => {
          toast.error(err.message);
        }
      });
    }
  };

  const isPending = createMutation.isPending || updateMutation.isPending;

  return (
    <div className="space-y-6">
      <LocationFormFields 
        address={address}
        editingLocation={editingLocation}
        isHeadquarters={isHeadquarters}
        name={name}
        setAddress={setAddress}
        setIsHeadquarters={setIsHeadquarters}
        setName={setName}
        setTimezone={setTimezone}
        timezone={timezone}
      />

      <div className="pt-6 border-t border-slate-100 dark:border-slate-800 flex justify-end gap-3">
        <Button disabled={isPending} onClick={onClose} variant="ghost">
          {t("common.cancel")}
        </Button>
        <Button loading={isPending} onClick={handleSave} variant="primary">
          {t("common.save")}
        </Button>
      </div>
    </div>
  );
}

interface FieldsProps {
  address: string;
  editingLocation: WorkLocation | null;
  isHeadquarters: boolean;
  name: string;
  setAddress: (v: string) => void;
  setIsHeadquarters: (v: boolean) => void;
  setName: (v: string) => void;
  setTimezone: (v: string) => void;
  timezone: string;
}

function LocationFormFields({
  name,
  setName,
  address,
  setAddress,
  timezone,
  setTimezone,
  isHeadquarters,
  setIsHeadquarters,
  editingLocation,
}: FieldsProps) {
  const { t } = useTranslation();
  const LABEL_BASE = "block text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-2";

  return (
    <>
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

      {(!editingLocation?.is_headquarters) ? (
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
      ) : null}
    </>
  );
}
