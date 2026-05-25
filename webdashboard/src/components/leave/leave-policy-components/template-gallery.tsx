import { Search, ChevronRight } from "lucide-react";
import React, { useState, useMemo } from "react";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

import { LEAVE_TEMPLATES, LeaveTemplate } from "../leave-templates";

interface TemplateGalleryProps {
  onSelectTemplate: (template: LeaveTemplate) => void;
  onSkip: () => void;
}

export function TemplateGallery({ onSelectTemplate, onSkip }: TemplateGalleryProps) {
  const [search, setSearch] = useState("");
  const [activeCategory, setActiveCategory] = useState<string>("all");

  const categories = [
    { id: "all", name: "All" },
    { id: "standard", name: "Standard" },
    { id: "wellness", name: "Wellness" },
    { id: "family", name: "Family" },
    { id: "professional", name: "Career" },
    { id: "special", name: "Special" },
  ];

  const filteredTemplates = useMemo(() => {
    return LEAVE_TEMPLATES.filter((t: LeaveTemplate) => {
      const matchesSearch =
        t.name.toLowerCase().includes(search.toLowerCase()) ||
        t.code.toLowerCase().includes(search.toLowerCase());
      const matchesCategory = activeCategory === "all" || t.category === activeCategory;
      return matchesSearch && matchesCategory;
    });
  }, [search, activeCategory]);

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

  return (
    <div className="flex flex-col h-[65vh] max-h-[700px] -m-8">
      {/* Header with Search */}
      <div className="px-8 pt-6 pb-6 border-b border-slate-50 dark:border-slate-800 bg-white/50 dark:bg-slate-900/50 backdrop-blur-sm sticky top-0 z-10">
        <div className="flex items-center justify-between gap-4">
          <div className="relative flex-1">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400" size={16} />
            <input
              className="w-full bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 rounded-2xl py-3 pl-11 pr-4 text-xs focus:ring-2 focus:ring-primary-500/20 outline-none transition-all"
              onChange={(e) => {
                setSearch(e.target.value);
              }}
              placeholder="Search templates (e.g. 'Annual', 'Sick')..."
              type="text"
              value={search}
            />
          </div>
          <button
            className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest hover:underline whitespace-nowrap"
            onClick={onSkip}
          >
            Skip to manual
          </button>
        </div>

        <div className="flex flex-wrap gap-2 mt-4 pb-2">
          {categories.map((cat) => (
            <button
              className={cn(
                "px-4 py-1.5 rounded-xl text-[9px] font-bold uppercase tracking-widest whitespace-nowrap transition-all",
                activeCategory === cat.id
                  ? "bg-primary-600 text-white shadow-lg shadow-primary-500/20"
                  : "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 border border-slate-100 dark:border-slate-700"
              )}
              key={cat.id}
              onClick={() => {
                setActiveCategory(cat.id);
              }}
            >
              {cat.name}
            </button>
          ))}
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-8 custom-scrollbar">
        {filteredTemplates.length > 0 ? (
          <div className="grid grid-cols-1 gap-4">
            {filteredTemplates.map((template: LeaveTemplate) => {
              const Icon = template.icon;
              return (
                <button
                  className="group relative flex items-start gap-3 p-4 rounded-2xl border border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900/50 hover:border-primary-200 dark:hover:border-primary-800 hover:bg-primary-50/30 dark:hover:bg-primary-950/10 transition-all text-left"
                  key={template.id}
                  onClick={() => {
                    onSelectTemplate(template);
                  }}
                >
                  <div
                    className={cn(
                      "h-10 w-10 shrink-0 rounded-xl flex items-center justify-center border transition-transform group-hover:scale-105",
                      getColorClasses(template.color)
                    )}
                  >
                    <Icon size={18} />
                  </div>
                  <div className="flex-1 min-w-0 pr-6">
                    <div className="flex items-center gap-2 mb-1">
                      <Typography className="text-sm font-bold text-slate-800 dark:text-slate-100 uppercase tracking-tight truncate" variant="h4">
                        {template.name}
                      </Typography>
                      <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 shrink-0">
                        {template.code}
                      </span>
                    </div>
                    <Typography className="text-xs text-slate-500 dark:text-slate-400 line-clamp-2 leading-relaxed" variant="p">
                      {template.description}
                    </Typography>
                    <div className="mt-2 flex items-center gap-2">
                      <span
                        className={cn(
                          "px-2 py-0.5 rounded-md text-[9px] font-bold uppercase tracking-widest",
                          template.is_paid
                            ? "bg-emerald-50 dark:bg-emerald-950/50 text-emerald-600"
                            : "bg-amber-50 dark:bg-amber-950/50 text-amber-600"
                        )}
                      >
                        {template.is_paid ? "Paid" : "Unpaid"}
                      </span>
                    </div>
                  </div>
                  <ChevronRight
                    className="absolute right-4 top-1/2 -translate-y-1/2 text-slate-300 dark:text-slate-700 group-hover:text-primary-400 transition-all group-hover:translate-x-1"
                    size={16}
                  />
                </button>
              );
            })}
          </div>
        ) : (
          <div className="h-full flex flex-col items-center justify-center py-12 text-center">
            <Search className="text-slate-200 dark:text-slate-800 mb-4" size={32} />
            <Typography className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h4">
              No templates found
            </Typography>
            <Typography className="text-xs text-slate-400 dark:text-slate-500 mt-1 max-w-[200px]" variant="p">
              Try a different keyword or category
            </Typography>
          </div>
        )}
      </div>
    </div>
  );
}
