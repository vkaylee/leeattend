import { MapPin, Plus, Trash2, Edit2, Globe2 } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { getCurrentWorkspaceId } from "@/lib/api";
import { WorkLocation } from "@/lib/api-generated/models";
import { cn } from "@/lib/utils";

import LocationFormDrawer from "./components/location-form-drawer";
import { useLocations, useDeleteLocation } from "./hooks/use-locations";

export default function LocationsSettings() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() ?? "";
  const { data: locations, isLoading } = useLocations(wsId);
  const deleteLocation = useDeleteLocation(wsId);
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
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50">
            <MapPin size={24} />
          </div>
          <div>
            <Typography className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200 leading-tight" variant="h1">
              {t("settings.categories.locations", { defaultValue: "Locations" })}
            </Typography>
            <Typography className="text-slate-500 dark:text-slate-400 mt-1 font-medium" variant="p">
              {t("settings.categories.locations_desc", { defaultValue: "Manage work locations and timezones" })}
            </Typography>
          </div>
        </div>
        
        <Button className="rounded-2xl px-6" onClick={handleAdd} variant="primary">
          <Plus className="mr-2" size={16} />
          {t("settings.locations.add", { defaultValue: "Add Location" })}
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 animate-in slide-in-from-bottom-4 duration-500">
        {isLoading ? (
          <div className="col-span-full py-12 text-center text-slate-500">Loading...</div>
        ) : locations?.length === 0 ? (
          <div className="col-span-full py-12 text-center text-slate-500">
            No locations found. Add your first work location.
          </div>
        ) : (
          locations?.map((location) => (
            <div 
              className={cn(
                "group relative bg-white dark:bg-slate-900 rounded-[2.5rem] border p-6 shadow-sm hover:shadow-md transition-all duration-300",
                location.is_headquarters 
                  ? "border-primary-200 dark:border-primary-800" 
                  : "border-slate-100 dark:border-slate-800"
              )} 
              key={location.id}
            >
              <div className="flex justify-between items-start mb-4">
                <div className="flex items-center gap-3">
                  <div className={cn(
                    "w-10 h-10 rounded-2xl flex items-center justify-center",
                    location.is_headquarters 
                      ? "bg-primary-50 text-primary-600 dark:bg-primary-900/30 dark:text-primary-400"
                      : "bg-slate-50 text-slate-500 dark:bg-slate-800 dark:text-slate-400"
                  )}>
                    {location.is_headquarters ? <Globe2 size={20} /> : <MapPin size={20} />}
                  </div>
                  <div>
                    <Typography className="font-bold text-slate-800 dark:text-slate-200" variant="h4">
                      {location.name}
                    </Typography>
                    {location.is_headquarters ? (
                      <span className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest px-2 py-0.5 bg-primary-50 dark:bg-primary-900/30 rounded-full">
                        {t("settings.locations.hq", { defaultValue: "Headquarters" })}
                      </span>
                    ) : null}
                  </div>
                </div>
                
                <div className="opacity-0 group-hover:opacity-100 transition-opacity flex gap-1">
                  <Button 
                    className="w-8 h-8 rounded-xl text-slate-400 hover:text-primary-600" 
                    onClick={() => { handleEdit(location); }} 
                    size="icon"
                    variant="ghost"
                  >
                    <Edit2 size={14} />
                  </Button>
                  {!location.is_headquarters && (
                    <Button 
                      className="w-8 h-8 rounded-xl text-slate-400 hover:text-red-600" 
                      onClick={() => { handleDelete(location.id, location.is_headquarters); }} 
                      size="icon"
                      variant="ghost"
                    >
                      <Trash2 size={14} />
                    </Button>
                  )}
                </div>
              </div>
              
              <div className="space-y-3 mt-6">
                <div className="text-sm">
                  <Typography className="text-xs font-bold text-slate-400 uppercase tracking-widest mb-1" variant="p">
                    {t("settings.locations.address", { defaultValue: "Address" })}
                  </Typography>
                  <span className="text-slate-600 dark:text-slate-300 font-medium">
                    {location.address ?? "—"}
                  </span>
                </div>
                <div className="text-sm">
                  <Typography className="text-xs font-bold text-slate-400 uppercase tracking-widest mb-1" variant="p">
                    {t("settings.locations.timezone", { defaultValue: "Timezone" })}
                  </Typography>
                  <span className="text-slate-600 dark:text-slate-300 font-medium">
                    {location.timezone}
                  </span>
                </div>
              </div>
            </div>
          ))
        )}
      </div>

      <LocationFormDrawer 
        editingLocation={editingLocation} 
        isOpen={isDrawerOpen} 
        onClose={() => { setIsDrawerOpen(false); }}
        wsId={wsId}
      />
    </div>
  );
}
