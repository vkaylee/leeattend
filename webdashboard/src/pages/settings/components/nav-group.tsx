import { Link } from "@tanstack/react-router";
import { ChevronRight } from "lucide-react";
import React from "react";

import { SettingItem } from "../hooks/use-settings-nav";

interface NavGroupProps {
  isActive: (path: string) => boolean;
  items: SettingItem[];
  title: string;
}

export function NavGroup({ title, items, isActive }: NavGroupProps) {
  return (
    <div className="mb-6">
      <h3 className="px-4 text-[10px] font-black text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] mb-2">{title}</h3>
      <div className="space-y-1">
        {items.map((item) => (
          <Link
            className={`flex items-start gap-3 px-4 py-3 rounded-2xl transition-all duration-300 group ${
              isActive(item.path)
                ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 shadow-sm shadow-primary-100/50 border border-primary-100 dark:border-primary-800/50"
                : "text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 hover:text-slate-700 dark:hover:text-slate-200 border border-transparent"
            }`}
            key={item.path}
            to={item.path}
          >
            <div
              className={`p-2 rounded-xl transition-colors ${
                isActive(item.path)
                  ? "bg-white dark:bg-slate-900 shadow-sm"
                  : "bg-slate-50 dark:bg-slate-800 group-hover:bg-white dark:hover:bg-slate-900"
              }`}
            >
              {item.icon}
            </div>
            <div className="flex flex-col min-w-0">
              <span className="font-bold text-sm leading-tight">{item.label}</span>
              <span className="text-[10px] text-slate-500 dark:text-slate-400 font-medium leading-tight hidden xl:block mt-0.5">
                {item.description}
              </span>
            </div>
            {isActive(item.path) ? (
              <ChevronRight className="w-4 h-4 ml-auto text-primary-400" />
            ) : null}
          </Link>
        ))}
      </div>
    </div>
  );
}
