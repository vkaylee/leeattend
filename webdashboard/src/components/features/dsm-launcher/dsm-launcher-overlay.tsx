/* eslint-disable max-lines-per-function, react/forbid-elements */

import * as Dialog from "@radix-ui/react-dialog";
import { useNavigate } from "@tanstack/react-router";
import { Command } from "cmdk";
import {
  Clock,
  Layers,
  LayoutGrid,
  Search,
  SearchX,
  Sparkles,
  X,
} from "lucide-react";
import React, { useCallback, useId, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { useMyPermissions } from "@/hooks/use-permissions";
import { useQuickAccess } from "@/hooks/use-quick-access";
import {
  buildFeatureSearchIndex,
  filterFeatures,
  getAllFeatures,
  type FeatureItem,
} from "@/lib/feature-registry";
import { cn, removeDiacritics } from "@/lib/utils";
import Button from "@/components/ui/button";

import { DsmSearchItem } from "./dsm-search-item";
import { DsmSquircleIcon } from "./dsm-squircle-icon";
interface DsmLauncherOverlayProps {
  isOpen: boolean;
  onClose: () => void;
}

export function DsmLauncherOverlay({ isOpen, onClose }: DsmLauncherOverlayProps) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const titleId = useId();
  const [search, setSearch] = useState("");

  const { hasAnyPermission, isLoading: permsLoading } = useMyPermissions();
  const { top3, recordAccess } = useQuickAccess();

  // Filter all features according to standalone permissions.
  const authorizedFeatures = useMemo(() => {
    if (permsLoading) return [];
    return filterFeatures(getAllFeatures(), hasAnyPermission);
  }, [permsLoading, hasAnyPermission]);

  // App-category features
  const appFeatures = useMemo(
    () => authorizedFeatures.filter((f) => f.category === "app"),
    [authorizedFeatures],
  );

  // Action-category features
  const actionFeatures = useMemo(
    () => authorizedFeatures.filter((f) => f.category === "action"),
    [authorizedFeatures],
  );

  // Launcher owns app and action shortcuts; Settings Hub owns settings navigation.
  const gridFeatures = appFeatures;

  // Quick Access apps
  const quickAccessApps = useMemo(() => {
    return top3
      .map((id) => appFeatures.find((f) => f.id === `app-${id}`))
      .filter(Boolean) as FeatureItem[];
  }, [top3, appFeatures]);

  // Search state and diacritic-normalized filtering
  const isSearching = search.trim().length > 0;

  const searchFilter = useCallback(
    (item: FeatureItem): boolean => {
      if (!isSearching) return true;
      const cleanSearch = removeDiacritics(search).toLowerCase().trim();
      const index = buildFeatureSearchIndex(item, t);
      return index.includes(cleanSearch);
    },
    [isSearching, search, t],
  );

  const displayedApps = useMemo(
    () => appFeatures.filter(searchFilter),
    [appFeatures, searchFilter],
  );

  const displayedActions = useMemo(
    () => actionFeatures.filter(searchFilter),
    [actionFeatures, searchFilter],
  );

  const totalResults = displayedApps.length + displayedActions.length;
  // Handle selecting an item
  const handleSelectItem = useCallback(
    (item: FeatureItem) => {
      if (item.category === "app") {
        const rawAppId = item.id.replace(/^app-/, "");
        recordAccess(rawAppId);
      }
      onClose();
      setSearch("");
      void navigate({ to: item.to as "/" });
    },
    [navigate, onClose, recordAccess],
  );



  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => { if (!open) { onClose(); setSearch(""); } }}>
      <Dialog.Portal>
        {/* Backdrop with frosted glass & blur */}
        <Dialog.Overlay
          data-testid="dsm-launcher-overlay"
          className={cn(
            "fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-3xl transition-opacity duration-300",
          )}
        />

        {/* Main Launcher Window (Fullscreen Desktop OS View) */}
        <Dialog.Content
          aria-labelledby={titleId}
          data-testid="dsm-launcher-modal"
          className={cn(
            "fixed inset-0 z-50 w-screen h-screen flex flex-col",
            "bg-slate-950/85 backdrop-blur-3xl overflow-hidden text-white outline-none animate-in fade-in duration-200",
          )}
        >
            {/* Accessible Dialog Title */}
            <Dialog.Title id={titleId} className="sr-only">
              {t("appLauncher.dsmTitle", "DSM 7 Main Menu & Feature Launcher")}
            </Dialog.Title>
            <Dialog.Description className="sr-only">
              {t("appLauncher.dsmDesc", "Khởi chạy ứng dụng và tìm kiếm tính năng")}
            </Dialog.Description>

            <Command
              shouldFilter={false}
              className="flex flex-col h-full overflow-hidden"
            >
              {/* Header: DSM Brand + Center Search Bar + Close */}
              <div className="flex items-center justify-between gap-4 px-6 sm:px-12 py-4 sm:py-5 border-b border-white/10 bg-white/5 shrink-0">
                <div className="flex items-center gap-3 text-primary-400 shrink-0">
                  <div className="w-9 h-9 rounded-2xl bg-gradient-to-br from-primary-500 to-indigo-600 flex items-center justify-center shadow-lg">
                    <LayoutGrid className="w-5 h-5 text-white" />
                  </div>
                  <div className="flex flex-col">
                    <span className="font-bold text-sm tracking-wide text-white leading-none">
                      LeeAttend OS
                    </span>
                    <span className="text-[10px] text-slate-400 tracking-wider uppercase mt-1 hidden sm:inline">
                      Main Menu
                    </span>
                  </div>
                </div>

                {/* Center Search Input */}
                <div className="relative flex-1 max-w-xl md:max-w-2xl mx-auto flex items-center">
                  <Search className="absolute left-4 w-4 h-4 text-slate-400 pointer-events-none" />
                  <input
                    autoFocus
                    type="text"
                    data-testid="dsm-launcher-search-input"
                    placeholder={t(
                      "appLauncher.searchPlaceholder",
                      "Tìm kiếm ứng dụng, tính năng, cài đặt (ví dụ: 'nghỉ phép', 'wifi', 'ot')...",
                    )}
                    value={search}
                    onChange={(e: React.ChangeEvent<HTMLInputElement>) => setSearch(e.target.value)}
                    className={cn(
                      "w-full h-12 pl-11 pr-11 rounded-2xl bg-white/10 dark:bg-black/40 border border-white/20",
                      "text-sm text-white placeholder-slate-400 outline-none shadow-inner",
                      "focus:border-primary-400/80 focus:ring-2 focus:ring-primary-500/40 transition-all",
                    )}
                  />
                  {search && (
                    <Button
                      aria-label="Xóa từ khóa tìm kiếm"
                      className="absolute right-3.5 text-slate-400 hover:text-white p-1 h-auto"
                      onClick={() => setSearch("")}
                      size="sm"
                      variant="ghost"
                    >
                      <X className="w-4 h-4" />
                    </Button>
                  )}
                </div>

                {/* Hotkey Badge / Close button */}
                <div className="flex items-center gap-3 shrink-0">
                  <kbd className="hidden md:inline-flex items-center px-2.5 py-1 text-xs font-semibold text-slate-400 bg-white/10 rounded-xl border border-white/10">
                    ESC
                  </kbd>
                  <Button
                    aria-label="Đóng launcher"
                    className="p-2.5 rounded-2xl text-slate-400 hover:text-white hover:bg-white/10 transition-colors h-auto"
                    data-testid="dsm-launcher-close-btn"
                    onClick={() => { onClose(); setSearch(""); }}
                    size="sm"
                    variant="ghost"
                  >
                    <X className="w-5 h-5" />
                  </Button>
                </div>
              </div>
              {/* Body: Fullscreen Grid View vs Centered Search Results */}
              <div className="flex-1 overflow-y-auto px-6 sm:px-12 py-8 scrollbar-thin">
                {!isSearching ? (
                  /* ── Default Grid View (DSM 7 Main Menu) ── */
                  <div
                    key="dsm-grid-view"
                    data-testid="dsm-grid-view"
                    className="max-w-7xl mx-auto w-full space-y-10 animate-in fade-in duration-200"
                  >
                    {/* Quick Access Section */}
                    {quickAccessApps.length > 0 && (
                      <div data-testid="dsm-quick-access-section" className="space-y-4">
                        <div className="flex items-center gap-2 text-xs font-bold tracking-wider text-slate-400 uppercase">
                          <Clock className="w-4 h-4 text-primary-400" />
                          <span>{t("appLauncher.recent", "Truy cập gần đây")}</span>
                        </div>
                        <div className="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-8 gap-4 sm:gap-6 justify-items-center">
                          {quickAccessApps.map((item) => (
                            <DsmSquircleIcon
                              key={item.id}
                              item={item}
                              size="md"
                              testId={`dsm-quick-${item.id}`}
                              onClick={handleSelectItem}
                            />
                          ))}
                        </div>
                        <div className="border-b border-white/10 pt-4" />
                      </div>
                    )}

                    {/* Applications and the Settings Hub entry */}
                    <div data-testid="dsm-all-apps-section" className="space-y-4">
                      <div className="flex items-center gap-2 text-xs font-bold tracking-wider text-slate-400 uppercase">
                        <Layers className="w-4 h-4 text-primary-400" />
                        <span>{t("appLauncher.allApps", "Tất cả ứng dụng")}</span>
                        <span className="text-xs font-normal text-slate-500">
                          ({gridFeatures.length})
                        </span>
                      </div>

                      <div className="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-7 xl:grid-cols-8 gap-6 sm:gap-8 lg:gap-10 justify-items-center py-4">
                        {gridFeatures.map((item) => (
                          <DsmSquircleIcon
                            key={item.id}
                            item={item}
                            size="xl"
                            onClick={handleSelectItem}
                          />
                        ))}
                      </div>
                    </div>
                  </div>
                ) : (
                  /* ── Search Mode: Centered Card ── */
                  <div
                    key="dsm-search-view"
                    data-testid="dsm-search-view"
                    className="max-w-3xl mx-auto w-full py-4 animate-in fade-in duration-200"
                  >
                    <Command.List className="space-y-4">
                        {/* Empty state */}
                        {totalResults === 0 ? (
                          <div
                            data-testid="dsm-search-empty"
                            className="flex flex-col items-center justify-center py-12 text-center text-slate-400 space-y-3"
                          >
                            <div className="w-12 h-12 rounded-2xl bg-white/5 flex items-center justify-center text-slate-500">
                              <SearchX className="w-6 h-6" />
                            </div>
                            <p className="text-sm font-semibold text-slate-200">
                              Không tìm thấy tính năng hoặc ứng dụng phù hợp
                            </p>
                            <p className="text-xs text-slate-400 max-w-sm">
                              Thử tìm kiếm với các từ khóa thông dụng như: "nghỉ phép", "chấm công", "wifi", "ot", "bảng lương"...
                            </p>
                          </div>
                        ) : (
                          <>
                            {/* Apps Group */}
                            {displayedApps.length > 0 && (
                              <Command.Group
                                heading="Ứng dụng chính"
                                className="[&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-bold [&_[cmdk-group-heading]]:text-slate-400 [&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-wider [&_[cmdk-group-heading]]:px-3 [&_[cmdk-group-heading]]:py-1.5"
                              >
                                <div className="space-y-1">
                                  {displayedApps.map((item) => (
                                    <DsmSearchItem
                                      key={item.id}
                                      item={item}
                                      onSelect={handleSelectItem}
                                    />
                                  ))}
                                </div>
                              </Command.Group>
                            )}

                            {/* Actions & Sub-Features Group */}
                            {displayedActions.length > 0 && (
                              <Command.Group
                                heading="Thao tác & Tính năng nhanh"
                                className="[&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-bold [&_[cmdk-group-heading]]:text-emerald-400 [&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-wider [&_[cmdk-group-heading]]:px-3 [&_[cmdk-group-heading]]:py-1.5"
                              >
                                <div className="space-y-1">
                                  {displayedActions.map((item) => (
                                    <DsmSearchItem
                                      key={item.id}
                                      item={item}
                                      onSelect={handleSelectItem}
                                    />
                                  ))}
                                </div>
                              </Command.Group>
                            )}
                          </>
                        )}
                      </Command.List>
                    </div>
                )}
              </div>

              {/* Footer status / navigation hints */}
              <div className="px-6 sm:px-12 py-3.5 border-t border-white/10 bg-white/5 flex items-center justify-between text-xs text-slate-400 shrink-0">
                <div className="flex items-center gap-4">
                  <span className="flex items-center gap-1.5">
                    <kbd className="px-2 py-0.5 rounded bg-white/10 border border-white/10 text-[10px]">↑</kbd>
                    <kbd className="px-2 py-0.5 rounded bg-white/10 border border-white/10 text-[10px]">↓</kbd>
                    <span className="hidden sm:inline">di chuyển</span>
                  </span>
                  <span className="flex items-center gap-1.5">
                    <kbd className="px-2 py-0.5 rounded bg-white/10 border border-white/10 text-[10px]">↵</kbd>
                    <span className="hidden sm:inline">chọn</span>
                  </span>
                  <span className="flex items-center gap-1.5">
                    <kbd className="px-2 py-0.5 rounded bg-white/10 border border-white/10 text-[10px]">ESC</kbd>
                    <span className="hidden sm:inline">thoát</span>
                  </span>
                </div>
                <div className="flex items-center gap-1.5 text-primary-400">
                  <Sparkles className="w-4 h-4" />
                  <span className="hidden sm:inline">Synology DSM 7 Fullscreen Engine</span>
                </div>
              </div>
            </Command>
          </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
