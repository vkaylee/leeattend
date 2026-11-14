import { ChevronRight, Plus, Search } from "lucide-react";
import React, { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { SearchInput } from "@/components/ui/search-input";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

import { LEAVE_TEMPLATES, LeaveTemplate } from "../leave-templates";

interface TemplateGalleryProps {
  className?: string;
  onSelectTemplate: (template: LeaveTemplate) => void;
  onSkip: () => void;
}

// eslint-disable-next-line max-lines-per-function
export function TemplateGallery({ className, onSelectTemplate, onSkip }: TemplateGalleryProps) {
  const { t } = useTranslation();
  const [search, setSearch] = useState("");
  const [activeCategory, setActiveCategory] = useState<string>("all");

  const categories = [
    { id: "all", name: t("settings.leave.gallery.categories.all", "All") },
    { id: "standard", name: t("settings.leave.gallery.categories.standard", "Standard") },
    { id: "wellness", name: t("settings.leave.gallery.categories.wellness", "Wellness") },
    { id: "family", name: t("settings.leave.gallery.categories.family", "Family") },
    { id: "professional", name: t("settings.leave.gallery.categories.professional", "Career") },
    { id: "special", name: t("settings.leave.gallery.categories.special", "Special") },
  ];

  const filteredTemplates = useMemo(() => {
    return LEAVE_TEMPLATES.filter((tmpl: LeaveTemplate) => {
      const localizedName = t(`settings.leave.templates.${tmpl.id}.name`, tmpl.name);
      const matchesSearch =
        localizedName.toLowerCase().includes(search.toLowerCase()) ||
        tmpl.code.toLowerCase().includes(search.toLowerCase());
      const matchesCategory = activeCategory === "all" || tmpl.category === activeCategory;
      return matchesSearch && matchesCategory;
    });
  }, [search, activeCategory, t]);

  const getColorClasses = (color: string) => {
    const map: Record<string, string> = {
      blue: "bg-blue-50 dark:bg-blue-950/30 text-blue-600 border-blue-100 dark:border-blue-900/50",
      emerald: "bg-emerald-50 dark:bg-emerald-950/30 text-emerald-600 border-emerald-100 dark:border-emerald-900/50",
      rose: "bg-rose-50 dark:bg-rose-950/30 text-rose-600 border-rose-100 dark:border-rose-900/50",
      amber: "bg-amber-50 dark:bg-amber-950/30 text-amber-600 border-amber-100 dark:border-amber-900/50",
      indigo: "bg-indigo-50 dark:bg-indigo-950/30 text-indigo-600 border-indigo-100 dark:border-indigo-900/50",
      violet: "bg-violet-50 dark:bg-violet-950/30 text-violet-600 border-violet-100 dark:border-violet-900/50",
      pink: "bg-pink-50 dark:bg-pink-950/30 text-pink-600 border-pink-100 dark:border-pink-900/50",
      cyan: "bg-cyan-50 dark:bg-cyan-950/30 text-cyan-600 border-cyan-100 dark:border-cyan-900/50",
      teal: "bg-teal-50 dark:bg-teal-950/30 text-teal-600 border-teal-100 dark:border-teal-900/50",
      orange: "bg-orange-50 dark:bg-orange-950/30 text-orange-600 border-orange-100 dark:border-orange-900/50",
      slate: "bg-slate-50 dark:bg-slate-800/50 text-slate-600 border-slate-100 dark:border-slate-800",
      stone: "bg-stone-50 dark:bg-stone-900/50 text-stone-600 border-stone-100 dark:border-stone-800",
      brown: "bg-amber-100/50 dark:bg-amber-950/20 text-amber-800 dark:text-amber-400 border-amber-200 dark:border-amber-900",
      lime: "bg-lime-50 dark:bg-lime-950/30 text-lime-600 border-lime-100 dark:border-lime-900/50",
    };
    return map[color] || map.slate;
  };

  const isModal = !className;

  return (
    <div className={cn(isModal ? "flex flex-col h-[65dvh] max-h-[700px] -m-8" : "w-full space-y-6", className)}>
      {/* Header with Search */}
      <div className={cn(isModal ? "px-8 pt-6 pb-6 border-b border-slate-50 dark:border-slate-800 bg-white/50 dark:bg-slate-900/50 backdrop-blur-sm sticky top-0 z-10" : "space-y-4")}>
        <div className="flex items-center justify-between gap-4">
          <div className="flex-1">
            <SearchInput
              onChange={(e) => { setSearch(e.target.value); }}
              onClear={() => { setSearch(""); }}
              placeholder={t("settings.leave.gallery.search_placeholder", "Search templates (e.g. 'Annual', 'Sick')...")}
              size="sm"
              value={search}
            />
          </div>
          <Button
            className="text-xs text-primary-600 dark:text-primary-400 whitespace-nowrap hover:underline font-semibold"
            onClick={onSkip}
            size="sm"
            variant="ghost"
          >
            {t("settings.leave.gallery.skip", "Skip to manual")} →
          </Button>
        </div>

        <div className="flex flex-wrap gap-2 pt-1">
          {categories.map((cat) => (
            <Button
              className={cn(
                "px-4 py-1.5 rounded-xl text-[10px] font-bold uppercase tracking-widest whitespace-nowrap transition-all h-auto",
                activeCategory === cat.id
                  ? "bg-primary-600 text-white shadow-md shadow-primary-500/20"
                  : "bg-slate-100/80 dark:bg-slate-800/80 text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-slate-700 border border-slate-200/50 dark:border-slate-700/50"
              )}
              key={cat.id}
              onClick={() => {
                setActiveCategory(cat.id);
              }}
              variant="ghost"
            >
              {cat.name}
            </Button>
          ))}
        </div>
      </div>

      {/* Content */}
      <div className={cn(isModal ? "flex-1 overflow-y-auto p-8 custom-scrollbar" : "pt-2")}>
        {filteredTemplates.length > 0 ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {activeCategory === "all" && !search ? (
              <div
                className={cn(
                  "group relative flex flex-col justify-between p-6 rounded-3xl border-2 border-dashed border-slate-300 dark:border-slate-700",
                  "bg-slate-50/60 dark:bg-slate-900/40 hover:border-primary-500 dark:hover:border-primary-500 hover:bg-primary-50/20 dark:hover:bg-primary-950/20",
                  "hover:shadow-xl hover:shadow-primary-500/5 hover:-translate-y-1 transition-all duration-300",
                  "text-left h-full min-h-[220px] cursor-pointer focus-visible:ring-2 focus-visible:ring-primary-500 outline-none select-none overflow-hidden"
                )}
                onClick={onSkip}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    onSkip();
                  }
                }}
                role="button"
                tabIndex={0}
              >
                <div className="w-full">
                  <div className="h-12 w-12 rounded-2xl flex items-center justify-center border border-dashed border-primary-300 dark:border-primary-700 bg-primary-50 dark:bg-primary-950/60 text-primary-600 dark:text-primary-400 group-hover:scale-110 transition-transform shadow-xs">
                    <Plus size={22} />
                  </div>
                  <div className="mt-4">
                    <Typography className="text-base font-bold text-slate-900 dark:text-slate-100 tracking-tight group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors" variant="h4">
                      {t("settings.leave.gallery.custom_title", "Custom Leave Type")}
                    </Typography>
                    <Typography className="text-xs text-slate-500 dark:text-slate-400 line-clamp-2 leading-relaxed mt-2" variant="p">
                      {t("settings.leave.gallery.custom_desc", "Configure code, name, and entitlement rules from scratch without templates.")}
                    </Typography>
                  </div>
                </div>
                <div className="mt-6 pt-3.5 border-t border-dashed border-slate-200 dark:border-slate-800 flex items-center justify-between w-full">
                  <span className="text-xs font-semibold text-primary-600 dark:text-primary-400">
                    {t("settings.leave.gallery.custom_action", "Create blank")}
                  </span>
                  <div className="w-7 h-7 rounded-full bg-primary-100 dark:bg-primary-950 text-primary-600 dark:text-primary-400 group-hover:bg-primary-600 group-hover:text-white flex items-center justify-center transition-all duration-200 group-hover:translate-x-1">
                    <ChevronRight size={14} />
                  </div>
                </div>
              </div>
            ) : null}

            {filteredTemplates.map((template: LeaveTemplate) => {
              const Icon = template.icon;
              return (
                <div
                  className={cn(
                    "group relative flex flex-col justify-between p-6 rounded-3xl border border-slate-200/90 dark:border-slate-800",
                    "bg-white dark:bg-slate-900/90 hover:border-primary-500/80 dark:hover:border-primary-500/80",
                    "hover:shadow-xl hover:shadow-primary-500/5 hover:-translate-y-1 transition-all duration-300",
                    "text-left h-full min-h-[220px] cursor-pointer focus-visible:ring-2 focus-visible:ring-primary-500 outline-none select-none overflow-hidden"
                  )}
                  key={template.id}
                  onClick={() => {
                    onSelectTemplate(template);
                  }}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      onSelectTemplate(template);
                    }
                  }}
                  role="button"
                  tabIndex={0}
                >
                  <div className="absolute top-0 right-0 w-32 h-32 bg-primary-500/5 dark:bg-primary-500/10 rounded-full blur-2xl pointer-events-none group-hover:scale-150 transition-transform duration-500" />
                  <div className="w-full">
                    <div className="flex items-center justify-between gap-2">
                      <div
                        className={cn(
                          "h-12 w-12 shrink-0 rounded-2xl flex items-center justify-center border shadow-xs transition-transform duration-300 group-hover:scale-110 group-hover:rotate-1",
                          getColorClasses(template.color)
                        )}
                      >
                        <Icon size={20} />
                      </div>
                      <div className="flex items-center gap-1.5 flex-wrap justify-end">
                        <span className="px-2.5 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-widest bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border border-slate-200/50 dark:border-slate-700/50">
                          {t(`settings.leave.gallery.categories.${template.category}`, template.category)}
                        </span>
                        <span
                          className={cn(
                            "px-2.5 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-widest border flex items-center gap-1",
                            template.is_paid
                              ? "bg-emerald-50 dark:bg-emerald-950/70 text-emerald-700 dark:text-emerald-300 border-emerald-200/70 dark:border-emerald-800/70"
                              : "bg-amber-50 dark:bg-amber-950/70 text-amber-700 dark:text-amber-300 border-amber-200/70 dark:border-amber-800/70"
                          )}
                        >
                          <span className={cn("w-1.5 h-1.5 rounded-full", template.is_paid ? "bg-emerald-500" : "bg-amber-500")} />
                          {template.is_paid ? t("settings.leave.type_manager.paid", "Paid") : t("settings.leave.type_manager.unpaid", "Unpaid")}
                        </span>
                      </div>
                    </div>

                    <div className="mt-4">
                      <div className="flex items-center gap-2">
                        <Typography className="text-base font-bold text-slate-900 dark:text-slate-100 tracking-tight group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors truncate" variant="h4">
                          {t(`settings.leave.templates.${template.id}.name`, template.name)}
                        </Typography>
                        <span className="px-2 py-0.5 rounded-md font-mono text-[10px] font-bold uppercase tracking-wider bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300 border border-slate-200/60 dark:border-slate-700/60 shrink-0">
                          {template.code}
                        </span>
                      </div>
                      <Typography className="text-xs text-slate-500 dark:text-slate-400 line-clamp-2 leading-relaxed mt-2" variant="p">
                        {t(`settings.leave.templates.${template.id}.desc`, template.description)}
                      </Typography>
                    </div>
                  </div>

                  <div className="mt-6 pt-3.5 border-t border-slate-100 dark:border-slate-800/80 flex items-center justify-between w-full">
                    <span className="text-[11px] font-semibold text-slate-400 dark:text-slate-500 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors">
                      {t("settings.leave.gallery.select_action", "Use this template")}
                    </span>
                    <div className="w-7 h-7 rounded-full bg-slate-50 dark:bg-slate-800 border border-slate-200/60 dark:border-slate-700 group-hover:bg-primary-600 group-hover:border-primary-600 group-hover:text-white flex items-center justify-center transition-all duration-200 group-hover:translate-x-1 text-slate-400">
                      <ChevronRight size={14} />
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        ) : (
          <div className="h-full flex flex-col items-center justify-center py-12 text-center">
            <Search className="text-slate-200 dark:text-slate-800 mb-4" size={32} />
            <Typography className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h4">
              {t("settings.leave.gallery.empty_title", "No templates found")}
            </Typography>
            <Typography className="text-xs text-slate-400 dark:text-slate-500 mt-1 max-w-[200px]" variant="p">
              {t("settings.leave.gallery.empty_desc", "Try a different keyword or category")}
            </Typography>
          </div>
        )}
      </div>
    </div>
  );
}
