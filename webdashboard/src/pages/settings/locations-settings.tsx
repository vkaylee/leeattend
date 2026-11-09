/* eslint-disable @typescript-eslint/no-unnecessary-type-conversion, max-lines-per-function */
import { MapPin, Plus, Trash2, Edit2, Globe2 } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import { Skeleton } from "@/components/ui/skeleton";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { WorkLocation } from "@/lib/api/locations";
import { cn } from "@/lib/utils";

import { SettingsHeader } from "@/components/ui/settings/settings-header";
import LocationFormDrawer from "./components/location-form-drawer";
import { useLocations, useDeleteLocation } from "./hooks/use-locations";

export default function LocationsSettings() {
  const { t } = useTranslation();
  
  const { data: locations, isError, isLoading } = useLocations();
  const deleteLocation = useDeleteLocation();
  const toast = useToast();
  
  const [isDrawerOpen, setIsDrawerOpen] = useState(false);
  const [editingLocation, setEditingLocation] = useState<WorkLocation | null>(null);

  const handleDelete = (id: string, isHeadquarters: boolean) => {
    if (isHeadquarters) {
      toast.error(t("settings.locations.cannot_delete_hq", { defaultValue: "Cannot delete headquarters." }));
      return;
    }
    if (confirm(t("settings.locations.confirm_delete", { defaultValue: "Are you sure you want to delete this location?" }))) {
      deleteLocation.mutate(id, {
        onSuccess: () => { toast.success(t("settings.locations.deleted", { defaultValue: "Location deleted successfully" })); },
        onError: (err: Error) => { toast.error(err.message); },
      });
    }
  };

  const handleEdit = (location: WorkLocation) => {
    setEditingLocation(location);
    setIsDrawerOpen(true);
  };

  const handleAdd = () => {
    setEditingLocation(null);
    setIsDrawerOpen(true);
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <SettingsHeader
        actions={
          <Button className="rounded-xl px-4 py-2 text-xs font-semibold shadow-xs normal-case tracking-normal" onClick={handleAdd} variant="primary">
            <Plus className="mr-1.5" size={15} />
            {t("settings.locations.add", { defaultValue: "Thêm địa điểm" })}
          </Button>
        }
        description={t("settings.categories.locations_desc", { defaultValue: "Quản lý chi nhánh văn phòng và múi giờ" })}
        icon={<MapPin size={20} />}
        title={t("settings.categories.locations", { defaultValue: "Địa điểm làm việc" })}
      />

      <div className="flex flex-col gap-4 animate-in slide-in-from-bottom-4 duration-500">
        {isError ? (
          <p className="text-sm text-slate-500 dark:text-slate-400" role="status">
            {t("settings.locations.unavailable", { defaultValue: "Work locations are unavailable for your account." })}
          </p>
        ) : isLoading ? (
          Array.from({ length: 3 }).map((_, index) => (
            <div 
              className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 p-5 flex flex-col md:flex-row md:items-center justify-between gap-6 animate-pulse"
              key={index}
            >
              <div className="flex items-center gap-4 min-w-[240px]">
                <Skeleton className="w-11 h-11 rounded-2xl bg-slate-100 dark:bg-slate-800 shrink-0" />
                <div className="space-y-2">
                  <Skeleton className="h-5 w-32 bg-slate-100 dark:bg-slate-800" />
                  <Skeleton className="h-3.5 w-24 bg-slate-100 dark:bg-slate-800" />
                </div>
              </div>
              <div className="flex-1 min-w-0 space-y-2">
                <Skeleton className="h-3.5 w-16 bg-slate-100 dark:bg-slate-800" />
                <Skeleton className="h-4 w-48 bg-slate-100 dark:bg-slate-800" />
              </div>
              <div className="w-fit md:w-48 shrink-0 space-y-2">
                <Skeleton className="h-3.5 w-16 bg-slate-100 dark:bg-slate-800" />
                <Skeleton className="h-6 w-28 rounded-xl bg-slate-100 dark:bg-slate-800" />
              </div>
              <div className="flex items-center gap-2 justify-end shrink-0">
                <Skeleton className="w-9 h-9 rounded-xl bg-slate-100 dark:bg-slate-800" />
                <Skeleton className="w-9 h-9 rounded-xl bg-slate-100 dark:bg-slate-800" />
              </div>
            </div>
          ))
        ) : locations?.length === 0 ? (
          <EmptyState
            action={
              <Button className="rounded-2xl px-6 normal-case tracking-normal" onClick={handleAdd} variant="primary">
                <Plus className="mr-2" size={16} />
                {t("settings.locations.add", { defaultValue: "Thêm địa điểm" })}
              </Button>
            }
            description={t("settings.locations.empty_desc", { defaultValue: "Add your first work location to get started." })}
            icon={<MapPin className="w-12 h-12 text-slate-300 dark:text-slate-600" />}
            title={t("settings.locations.empty_title", { defaultValue: "No Locations" })}
          />
        ) : (
          locations?.map((location) => (
            <div 
              className={cn(
                "group relative bg-white dark:bg-slate-900 rounded-2xl border p-5 shadow-2xs transition-all duration-200",
                "flex flex-col md:flex-row md:items-center justify-between gap-6",
                location.is_headquarters 
                  ? "border-primary-200 dark:border-primary-800" 
                  : "border-slate-100 dark:border-slate-800"
              )} 
              key={location.id}
            >
              {/* Left Section: Icon and Location Name & Badge */}
              <div className="flex items-center gap-4 min-w-[240px]">
                <div className={cn(
                  "w-11 h-11 rounded-2xl flex items-center justify-center shrink-0 transition-colors duration-300",
                  location.is_headquarters 
                    ? "bg-primary-50 text-primary-600 dark:bg-primary-900/30 dark:text-primary-400"
                    : "bg-slate-50 text-slate-500 dark:bg-slate-800 dark:text-slate-400"
                )}>
                  {location.is_headquarters ? <Globe2 size={20} /> : <MapPin size={20} />}
                </div>
                <div className="space-y-1">
                  <Typography className="font-bold text-slate-800 dark:text-slate-200 text-lg leading-tight" variant="h4">
                    {location.name}
                  </Typography>
                  {!!location.is_headquarters && (
                    <span className="inline-flex text-[9px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest px-2 py-0.5 bg-primary-50 dark:bg-primary-900/30 rounded-full w-fit">
                      {t("settings.locations.hq", { defaultValue: "Headquarters" })}
                    </span>
                  )}
                </div>
              </div>

              {/* Middle Section: Address */}
              <div className="flex-1 min-w-0">
                <Typography className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-1" variant="p">
                  {t("settings.locations.address", { defaultValue: "Address" })}
                </Typography>
                <Typography className="text-sm text-slate-600 dark:text-slate-300 font-medium truncate" variant="p">
                  {location.address ?? "—"}
                </Typography>
              </div>

              {/* Middle Section: Timezone */}
              <div className="w-fit md:w-48 shrink-0">
                <Typography className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-1" variant="p">
                  {t("settings.locations.timezone", { defaultValue: "Timezone" })}
                </Typography>
                <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-xl text-xs font-semibold bg-slate-50 dark:bg-slate-800 text-slate-600 dark:text-slate-300 border border-slate-100 dark:border-slate-800/50">
                  {location.timezone}
                </span>
              </div>

              {/* Right Section: Actions */}
              <div className="flex items-center gap-2 md:opacity-0 group-hover:opacity-100 transition-opacity justify-end shrink-0">
                <Button 
                  className="w-9 h-9 rounded-xl text-slate-400 hover:text-primary-600 hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors" 
                  onClick={() => { handleEdit(location); }} 
                  size="icon"
                  variant="ghost"
                >
                  <Edit2 size={16} />
                </Button>
                {!location.is_headquarters && (
                  <Button 
                    className="w-9 h-9 rounded-xl text-slate-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-950/30 transition-colors" 
                    onClick={() => { handleDelete(location.id, location.is_headquarters); }} 
                    size="icon"
                    variant="ghost"
                  >
                    <Trash2 size={16} />
                  </Button>
                )}
              </div>
            </div>
          ))
        )}
      </div>

      <LocationFormDrawer editingLocation={editingLocation} 
      isOpen={isDrawerOpen} 
      key={isDrawerOpen ? (editingLocation?.id ?? 'new') : 'closed'}
      onClose={() => { setIsDrawerOpen(false); }}  />
    </div>
  );
}
