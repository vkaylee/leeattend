import * as Dialog from "@radix-ui/react-dialog";
import { useNavigate } from "@tanstack/react-router";
import {
  CornerDownLeft,
  Search,
  Sparkles,
  X,
} from "lucide-react";
import React, { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { Command } from "cmdk";

import Button from "@/components/ui/button";
import { useMyPermissions } from "@/hooks/use-permissions";
import { useFeatureFlagVersion } from "@/lib/feature-flags";
import {
  buildFeatureSearchIndex,
  filterFeatures,
  getAllFeatures,
  type FeatureItem,
} from "@/lib/feature-registry";
import { removeDiacritics } from "@/lib/utils";

export interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
}

export function openCommandPalette(): void {
  if (typeof window !== "undefined") {
    window.dispatchEvent(new CustomEvent("open-command-palette"));
  }
}

export function CommandPalette({ isOpen, onClose }: CommandPaletteProps) {
  useFeatureFlagVersion();
  const { t } = useTranslation(["common", "dashboard"]);
  const navigate = useNavigate();
  const { hasAnyPermission } = useMyPermissions();
  const [search, setSearch] = useState("");

  // Listen to Cmd+K / Ctrl+K keyboard shortcut
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        if (isOpen) {
          onClose();
        }
      }
      if (e.key === "Escape" && isOpen) {
        e.preventDefault();
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => { window.removeEventListener("keydown", handleKeyDown); };
  }, [isOpen, onClose]);

  // Reset search when dialog opens/closes
  useEffect(() => {
    if (!isOpen) {
      setSearch("");
    }
  }, [isOpen]);

  const allFeatures = useMemo(() => {
    const raw = getAllFeatures();
    return filterFeatures(raw, hasAnyPermission);
  }, [hasAnyPermission]);

  const apps = useMemo(
    () => allFeatures.filter((f) => f.category === "app"),
    [allFeatures]
  );
  const actions = useMemo(
    () => allFeatures.filter((f) => f.category === "action"),
    [allFeatures]
  );

  const handleSelectItem = (item: FeatureItem) => {
    onClose();
    void navigate({ to: item.to });
  };

  // Custom filter for cmdk with diacritics support
  const filterCommandItem = (value: string, searchVal: string): number => {
    const normalizedSearch = removeDiacritics(searchVal.toLowerCase().trim());
    const normalizedValue = removeDiacritics(value.toLowerCase());
    return normalizedValue.includes(normalizedSearch) ? 1 : 0;
  };

  return (
    <Dialog.Root onOpenChange={(open) => { if (!open) onClose(); }} open={isOpen}>
      <Dialog.Portal>
        <Dialog.Overlay
          className="fixed inset-0 z-50 bg-black/60 backdrop-blur-xs transition-opacity animate-in fade-in duration-150"
          data-testid="command-palette-overlay"
        />
        <Dialog.Content
          aria-label={t("common.search", "Command Palette")}
           className="fixed top-[15%] left-1/2 z-50 w-full max-w-xl -translate-x-1/2 overflow-hidden rounded-xl border border-border bg-card p-0 shadow-2xl animate-in zoom-in-95 duration-150 outline-none"
           data-testid="command-palette-modal"
         >
           <Dialog.Title className="sr-only">
            {t("common.search", "Command Palette")}
           </Dialog.Title>
          <Dialog.Description className="sr-only">
            Quick navigation and feature launcher
          </Dialog.Description>

          <Command
            className="flex flex-col overflow-hidden"
            filter={filterCommandItem}
          >
            {/* Search Input Bar */}
            <div className="flex items-center border-b border-border bg-slate-50/50 dark:bg-slate-900/40 px-4 py-3 gap-3">
              <Search className="h-4 w-4 shrink-0 text-slate-400 dark:text-slate-500" />
              <Command.Input
                className="flex-1 bg-transparent text-sm font-medium text-foreground placeholder:text-muted-foreground border-none outline-none focus:outline-none focus:ring-0 focus-visible:outline-none shadow-none"
                data-testid="command-palette-input"
                onValueChange={setSearch}
                placeholder={t(
                  "common.search_placeholder",
                  "Tìm kiếm nhanh tính năng, thao tác (vd: 'nghỉ phép', 'wifi', 'ot')..."
                )}
                value={search}
              />
              {!!search && (
                <Button
                  aria-label="Clear search"
                  className="h-6 w-6 p-0 text-muted-foreground hover:text-foreground"
                  onClick={() => { setSearch(""); }}
                  size="sm"
                  variant="ghost"
                >
                  <X className="h-3.5 w-3.5" />
                </Button>
              )}
              <kbd className="rounded border border-border bg-muted px-1.5 py-0.5 text-[10px] font-mono text-muted-foreground">
                ESC
              </kbd>
            </div>

            {/* Results List */}
            <Command.List
              className="max-h-[360px] overflow-y-auto p-2 scrollbar-thin scrollbar-thumb-muted-foreground/20"
              data-testid="command-palette-list"
            >
              <Command.Empty
                className="py-8 text-center text-xs text-muted-foreground"
                data-testid="command-palette-empty"
              >
                {t(
                  "common.no_results",
                  "Không tìm thấy kết quả phù hợp với từ khóa."
                )}
              </Command.Empty>

              {/* Applications & Modules Group */}
              <Command.Group
                className="px-1 py-1.5 [&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1 [&_[cmdk-group-heading]]:text-[11px] [&_[cmdk-group-heading]]:font-semibold [&_[cmdk-group-heading]]:tracking-wider [&_[cmdk-group-heading]]:text-muted-foreground/80 [&_[cmdk-group-heading]]:uppercase"
                heading={t("common.sections.organization", "Phân hệ & Ứng dụng")}
              >
                {apps.map((item) => {
                  const ItemIcon = item.icon;
                  const itemSearchIndex = buildFeatureSearchIndex(item, t);
                  return (
                    <Command.Item
                      className="group flex cursor-pointer select-none items-center justify-between rounded-lg px-2.5 py-2 text-sm text-foreground transition-colors hover:bg-primary/10 hover:text-primary data-[selected=true]:bg-primary/10 data-[selected=true]:text-primary outline-none"
                      data-testid={`command-item-${item.id}`}
                      key={item.id}
                      onSelect={() => { handleSelectItem(item); }}
                      value={itemSearchIndex}
                    >
                      <div className="flex items-center gap-2.5 min-w-0">
                        <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-muted text-foreground group-hover:bg-primary group-hover:text-primary-foreground group-data-[selected=true]:bg-primary group-data-[selected=true]:text-primary-foreground transition-colors">
                          <ItemIcon className="h-4 w-4" />
                        </div>
                        <div className="flex flex-col truncate text-left">
                          <span className="font-medium truncate text-xs sm:text-sm">
                            {item.titleKey
                              ? t(item.titleKey, {
                                  defaultValue: item.titleFallback,
                                })
                              : item.titleFallback}
                          </span>
                        </div>
                      </div>
                      <CornerDownLeft className="h-3.5 w-3.5 opacity-0 group-hover:opacity-100 group-data-[selected=true]:opacity-100 text-muted-foreground transition-opacity" />
                    </Command.Item>
                  );
                })}
              </Command.Group>

              {/* Quick Actions Group */}
              <Command.Group
                className="mt-1 px-1 py-1.5 [&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1 [&_[cmdk-group-heading]]:text-[11px] [&_[cmdk-group-heading]]:font-semibold [&_[cmdk-group-heading]]:tracking-wider [&_[cmdk-group-heading]]:text-muted-foreground/80 [&_[cmdk-group-heading]]:uppercase"
                heading={t("common.actions", "Thao tác nhanh")}
              >
                {actions.map((item) => {
                  const ItemIcon = item.icon;
                  const itemSearchIndex = buildFeatureSearchIndex(item, t);
                  return (
                    <Command.Item
                      className="group flex cursor-pointer select-none items-center justify-between rounded-lg px-2.5 py-2 text-sm text-foreground transition-colors hover:bg-emerald-500/10 hover:text-emerald-600 dark:hover:text-emerald-400 data-[selected=true]:bg-emerald-500/10 data-[selected=true]:text-emerald-600 dark:data-[selected=true]:text-emerald-400 outline-none"
                      data-testid={`command-item-${item.id}`}
                      key={item.id}
                      onSelect={() => { handleSelectItem(item); }}
                      value={itemSearchIndex}
                    >
                      <div className="flex items-center gap-2.5 min-w-0">
                        <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 group-hover:bg-emerald-500 group-hover:text-white group-data-[selected=true]:bg-emerald-500 group-data-[selected=true]:text-white transition-colors">
                          <ItemIcon className="h-4 w-4" />
                        </div>
                        <div className="flex flex-col truncate text-left">
                          <span className="font-medium truncate text-xs sm:text-sm">
                            {item.titleKey
                              ? t(item.titleKey, {
                                  defaultValue: item.titleFallback,
                                })
                              : item.titleFallback}
                          </span>
                        </div>
                      </div>
                      <div className="flex items-center gap-1.5">
                        <Sparkles className="h-3 w-3 text-emerald-500/60" />
                        <CornerDownLeft className="h-3.5 w-3.5 opacity-0 group-hover:opacity-100 group-data-[selected=true]:opacity-100 text-muted-foreground transition-opacity" />
                      </div>
                    </Command.Item>
                  );
                })}
              </Command.Group>
            </Command.List>

            {/* Footer hints */}
            <div className="flex items-center justify-between border-t border-border px-3 py-2 text-[11px] text-muted-foreground bg-muted/30 select-none">
              <span className="flex items-center gap-1.5">
                <span>Dùng</span>
                <kbd className="rounded border border-border bg-background px-1 py-0.5 font-mono text-[10px]">
                  ↑
                </kbd>
                <kbd className="rounded border border-border bg-background px-1 py-0.5 font-mono text-[10px]">
                  ↓
                </kbd>
                <span>để di chuyển</span>
              </span>
              <span className="flex items-center gap-1">
                <span>Nhấn</span>
                <kbd className="rounded border border-border bg-background px-1 py-0.5 font-mono text-[10px]">
                  ↵ Enter
                </kbd>
                <span>để chọn</span>
              </span>
            </div>
          </Command>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
export default CommandPalette;
