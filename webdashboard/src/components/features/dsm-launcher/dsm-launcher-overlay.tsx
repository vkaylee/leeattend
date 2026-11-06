/* eslint-disable max-lines-per-function */

import * as Dialog from "@radix-ui/react-dialog";
import { useNavigate } from "@tanstack/react-router";
import { Command } from "cmdk";
import {
  Clock,
  Compass,
  CornerDownLeft,
  Layers,
  LayoutGrid,
  Search,
  SearchX,
  ShieldCheck,
  Sparkles,
  X,
} from "lucide-react";
import React, { useCallback, useEffect, useId, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useMyPermissions } from "@/hooks/use-permissions";
import { useQuickAccess } from "@/hooks/use-quick-access";
import {
  buildFeatureSearchIndex,
  filterFeatures,
  getAllFeatures,
  type FeatureItem,
} from "@/lib/feature-registry";
import { cn, removeDiacritics } from "@/lib/utils";

import { DsmSearchItem } from "./dsm-search-item";
import { DsmSquircleIcon } from "./dsm-squircle-icon";

interface DsmLauncherOverlayProps {
  isOpen: boolean;
  onClose: () => void;
}

function getGridColumnCount(): number {
  if (typeof window === "undefined") return 8;
  const width = window.innerWidth;
  if (width >= 1280) return 8;
  if (width >= 1024) return 7;
  if (width >= 768) return 6;
  if (width >= 640) return 4;
  return 3;
}

export function DsmLauncherOverlay({ isOpen, onClose }: DsmLauncherOverlayProps) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const titleId = useId();

  const [search, setSearch] = useState("");
  const [focusedGridIndex, setFocusedGridIndex] = useState<number | null>(null);
  const [activeSearchId, setActiveSearchId] = useState<string | null>(null);

  const inputRef = useRef<HTMLInputElement>(null);
  const gridButtonRefs = useRef<(HTMLButtonElement | null)[]>([]);

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

  const gridFeatures = appFeatures;

  // Quick Access apps (reactive with top3 changes)
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

  const allSearchResults = useMemo(
    () => [...displayedApps, ...displayedActions],
    [displayedApps, displayedActions],
  );

  const totalResults = allSearchResults.length;

  // Sync active search item when results change
  useEffect(() => {
    if (isSearching && allSearchResults.length > 0) {
      const exists = allSearchResults.some((item) => item.id === activeSearchId);
      if (!exists) {
        setActiveSearchId(allSearchResults[0].id);
      }
    } else {
      setActiveSearchId(null);
    }
  }, [isSearching, allSearchResults, activeSearchId]);

  // Active item object for side preview
  const activePreviewItem = useMemo(() => {
    if (!activeSearchId) return allSearchResults[0] ?? null;
    return allSearchResults.find((item) => item.id === activeSearchId) ?? allSearchResults[0] ?? null;
  }, [activeSearchId, allSearchResults]);

  // Handle selecting an item
  const handleSelectItem = useCallback(
    (item: FeatureItem) => {
      if (item.category === "app") {
        const rawAppId = item.id.replace(/^app-/, "");
        recordAccess(rawAppId);
      }
      onClose();
      setSearch("");
      setFocusedGridIndex(null);
      void navigate({ to: item.to as "/" });
    },
    [navigate, onClose, recordAccess],
  );

  // Keyboard navigation when inside the Search Input
  const handleInputKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (!isSearching && e.key === "ArrowDown") {
        e.preventDefault();
        if (gridFeatures.length > 0) {
          setFocusedGridIndex(0);
          gridButtonRefs.current[0]?.focus();
        }
      }
    },
    [isSearching, gridFeatures.length],
  );

  // Global Grid Keyboard Navigation handler when focused on an app tile
  const handleGridKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLDivElement>) => {
      if (isSearching || focusedGridIndex === null) return;

      const cols = getGridColumnCount();
      const maxIndex = gridFeatures.length - 1;

      switch (e.key) {
        case "ArrowRight": {
          e.preventDefault();
          const next = Math.min(maxIndex, focusedGridIndex + 1);
          setFocusedGridIndex(next);
          gridButtonRefs.current[next]?.focus();
          break;
        }
        case "ArrowLeft": {
          e.preventDefault();
          const prev = Math.max(0, focusedGridIndex - 1);
          setFocusedGridIndex(prev);
          gridButtonRefs.current[prev]?.focus();
          break;
        }
        case "ArrowDown": {
          e.preventDefault();
          const next = focusedGridIndex + cols;
          if (next <= maxIndex) {
            setFocusedGridIndex(next);
            gridButtonRefs.current[next]?.focus();
          }
          break;
        }
        case "ArrowUp": {
          e.preventDefault();
          const prev = focusedGridIndex - cols;
          if (prev >= 0) {
            setFocusedGridIndex(prev);
            gridButtonRefs.current[prev]?.focus();
          } else {
            setFocusedGridIndex(null);
            inputRef.current?.focus();
          }
          break;
        }
        case "Enter":
        case " ": {
          e.preventDefault();
          const targetItem = gridFeatures[focusedGridIndex];
          if (targetItem) {
            handleSelectItem(targetItem);
          }
          break;
        }
        default: {
          // If typing a character, return focus to search input and initiate search
          if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
            setFocusedGridIndex(null);
            setSearch(e.key);
            inputRef.current?.focus();
          }
          break;
        }
      }
    },
    [isSearching, focusedGridIndex, gridFeatures, handleSelectItem],
  );

  return (
    <Dialog.Root
      open={isOpen}
      onOpenChange={(open) => {
        if (!open) {
          onClose();
          setSearch("");
          setFocusedGridIndex(null);
        }
      }}
    >
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
          data-testid="dsm-launcher-modal"
          onKeyDown={handleGridKeyDown}
          className={cn(
            "fixed inset-0 z-50 w-screen h-screen flex flex-col",
            "bg-slate-950/85 backdrop-blur-3xl overflow-hidden text-white outline-none animate-in fade-in duration-200",
          )}
        >
          {/* Accessible Dialog Title & Description */}
          <Dialog.Title className="sr-only">
            {t("dsmLauncher.title", "DSM 7 Main Menu & Feature Launcher")}
          </Dialog.Title>
          <Dialog.Description className="sr-only">
            {t("dsmLauncher.desc", "Khởi chạy ứng dụng và tìm kiếm tính năng")}
          </Dialog.Description>

          <Command
            shouldFilter={false}
            value={activeSearchId ?? undefined}
            onValueChange={(val) => {
              if (val) setActiveSearchId(val);
            }}
            className="flex flex-col h-full overflow-hidden"
          >
            {/* Header: DSM Brand + Center Search Bar + Close */}
            <div className="flex items-center justify-between gap-3 sm:gap-4 px-4 sm:px-12 py-3.5 sm:py-5 border-b border-white/10 bg-white/5 shrink-0">
              {/* Brand Logo & Title */}
              <div className="flex items-center gap-2.5 sm:gap-3 text-primary-400 shrink-0">
                <div className="w-8 h-8 sm:w-9 sm:h-9 rounded-2xl bg-gradient-to-br from-primary-500 to-indigo-600 flex items-center justify-center shadow-lg">
                  <LayoutGrid className="w-4 h-4 sm:w-5 sm:h-5 text-white" />
                </div>
                <div className="flex flex-col hidden sm:flex">
                  <span className="font-bold text-sm tracking-wide text-white leading-none">
                    LeeAttend OS
                  </span>
                  <span className="text-[10px] text-slate-400 tracking-wider uppercase mt-1">
                    Main Menu
                  </span>
                </div>
              </div>

              {/* Center Spotlight Search Bar */}
              <div className="relative flex-1 max-w-xl md:max-w-2xl mx-auto flex items-center">
                <Search className="absolute left-3.5 sm:left-4 w-4 h-4 text-slate-400 pointer-events-none" />
                <Command.Input
                  ref={inputRef}
                  autoFocus
                  data-testid="dsm-launcher-search-input"
                  placeholder={t(
                    "dsmLauncher.searchPlaceholder",
                    "Tìm kiếm ứng dụng, tính năng, cài đặt (ví dụ: 'nghỉ phép', 'wifi', 'ot')...",
                  )}
                  value={search}
                  onValueChange={(val) => {
                    setSearch(val);
                    setFocusedGridIndex(null);
                  }}
                  onKeyDown={handleInputKeyDown}
                  className={cn(
                    "w-full h-11 sm:h-12 pl-10 sm:pl-11 pr-10 sm:pr-11 rounded-2xl bg-white/10 dark:bg-black/40 border border-white/20",
                    "text-xs sm:text-sm text-white placeholder-slate-400 outline-none shadow-inner",
                    "focus:border-primary-400/80 focus:ring-2 focus:ring-primary-500/40 transition-all",
                  )}
                />
                {search && (
                  <Button
                    aria-label={t("dsmLauncher.aria.clearSearch", "Xóa từ khóa tìm kiếm")}
                    className="absolute right-2.5 sm:right-3.5 text-slate-400 hover:text-white p-1 h-auto"
                    onClick={() => {
                      setSearch("");
                      inputRef.current?.focus();
                    }}
                    size="sm"
                    variant="ghost"
                  >
                    <X className="w-4 h-4" />
                  </Button>
                )}
              </div>

              {/* Hotkey Badge / Close button */}
              <div className="flex items-center gap-2 sm:gap-3 shrink-0">
                <kbd className="hidden md:inline-flex items-center px-2.5 py-1 text-xs font-semibold text-slate-400 bg-white/10 rounded-xl border border-white/10">
                  ESC
                </kbd>
                <Button
                  aria-label={t("dsmLauncher.aria.close", "Đóng launcher")}
                  className="p-2 sm:p-2.5 rounded-2xl text-slate-400 hover:text-white hover:bg-white/10 transition-colors h-auto"
                  data-testid="dsm-launcher-close-btn"
                  onClick={() => {
                    onClose();
                    setSearch("");
                    setFocusedGridIndex(null);
                  }}
                  size="sm"
                  variant="ghost"
                >
                  <X className="w-5 h-5" />
                </Button>
              </div>
            </div>

            {/* Body: Fullscreen Grid View vs Dual-Column Search Results with Preview */}
            <div className="flex-1 overflow-y-auto px-4 sm:px-12 py-6 sm:py-8 scrollbar-thin">
              {!isSearching ? (
                /* ── Default Grid View (DSM 7 Main Menu) ── */
                <div
                  key="dsm-grid-view"
                  data-testid="dsm-grid-view"
                  className="max-w-7xl mx-auto w-full space-y-8 sm:space-y-10 animate-in fade-in duration-200"
                >
                  {/* Quick Access Section */}
                  {quickAccessApps.length > 0 && (
                    <div data-testid="dsm-quick-access-section" className="space-y-4">
                      <div className="flex items-center gap-2 text-xs font-bold tracking-wider text-slate-400 uppercase">
                        <Clock className="w-4 h-4 text-primary-400" />
                        <span>{t("dsmLauncher.recent", "Truy cập gần đây")}</span>
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

                  {/* All Applications Grid */}
                  <div data-testid="dsm-all-apps-section" className="space-y-4">
                    <div className="flex items-center gap-2 text-xs font-bold tracking-wider text-slate-400 uppercase">
                      <Layers className="w-4 h-4 text-primary-400" />
                      <span>{t("dsmLauncher.allApps", "Tất cả ứng dụng")}</span>
                      <span className="text-xs font-normal text-slate-500">
                        ({gridFeatures.length})
                      </span>
                    </div>

                    <div className="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-7 xl:grid-cols-8 gap-4 sm:gap-8 lg:gap-10 justify-items-center py-4">
                      {gridFeatures.map((item, index) => (
                        <DsmSquircleIcon
                          key={item.id}
                          ref={(el) => {
                            gridButtonRefs.current[index] = el;
                          }}
                          item={item}
                          size="xl"
                          isFocused={focusedGridIndex === index}
                          onClick={handleSelectItem}
                        />
                      ))}
                    </div>
                  </div>
                </div>
              ) : (
                /* ── Search Mode: Dual-Column Results + Side Preview (Desktop) ── */
                <div
                  key="dsm-search-view"
                  data-testid="dsm-search-view"
                  className="max-w-6xl mx-auto w-full py-2 sm:py-4 animate-in fade-in duration-200"
                >
                  {totalResults === 0 ? (
                    <div
                      data-testid="dsm-search-empty"
                      className="flex flex-col items-center justify-center py-16 text-center text-slate-400 space-y-3"
                    >
                      <div className="w-14 h-14 rounded-3xl bg-white/5 flex items-center justify-center text-slate-500 shadow-inner">
                        <SearchX className="w-7 h-7" />
                      </div>
                      <p className="text-sm font-semibold text-slate-200">
                        {t("dsmLauncher.emptyTitle", "Không tìm thấy tính năng hoặc ứng dụng phù hợp")}
                      </p>
                      <p className="text-xs text-slate-400 max-w-sm">
                        {t(
                          "dsmLauncher.emptyDesc",
                          "Thử tìm kiếm với các từ khóa thông dụng như: 'nghỉ phép', 'chấm công', 'wifi', 'ot', 'bảng lương'...",
                        )}
                      </p>
                    </div>
                  ) : (
                    <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
                      {/* Left: Command Results List */}
                      <div className="lg:col-span-7 xl:col-span-7">
                        <Command.List className="space-y-6">
                          {/* Core Apps Group */}
                          {displayedApps.length > 0 && (
                            <Command.Group
                              heading={t("dsmLauncher.appsHeading", "Ứng dụng chính")}
                              className="[&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-bold [&_[cmdk-group-heading]]:text-slate-400 [&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-wider [&_[cmdk-group-heading]]:px-3 [&_[cmdk-group-heading]]:py-1.5"
                            >
                              <div className="space-y-1.5">
                                {displayedApps.map((item) => (
                                  <DsmSearchItem
                                    key={item.id}
                                    item={item}
                                    isSelected={activePreviewItem?.id === item.id}
                                    onHighlight={(it) => setActiveSearchId(it.id)}
                                    onSelect={handleSelectItem}
                                  />
                                ))}
                              </div>
                            </Command.Group>
                          )}

                          {/* Actions & Sub-Features Group */}
                          {displayedActions.length > 0 && (
                            <Command.Group
                              heading={t("dsmLauncher.actionsHeading", "Thao tác & Tính năng nhanh")}
                              className="[&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-bold [&_[cmdk-group-heading]]:text-emerald-400 [&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-wider [&_[cmdk-group-heading]]:px-3 [&_[cmdk-group-heading]]:py-1.5"
                            >
                              <div className="space-y-1.5">
                                {displayedActions.map((item) => (
                                  <DsmSearchItem
                                    key={item.id}
                                    item={item}
                                    isSelected={activePreviewItem?.id === item.id}
                                    onHighlight={(it) => setActiveSearchId(it.id)}
                                    onSelect={handleSelectItem}
                                  />
                                ))}
                              </div>
                            </Command.Group>
                          )}
                        </Command.List>
                      </div>

                      {/* Right: Side Preview Card (Desktop >= lg) */}
                      {activePreviewItem && (
                        <div
                          data-testid="dsm-side-preview"
                          className="hidden lg:flex lg:col-span-5 xl:col-span-5 flex-col p-6 rounded-3xl bg-white/5 border border-white/10 shadow-2xl backdrop-blur-2xl sticky top-4 space-y-6"
                        >
                          {/* Preview Header */}
                          <div className="flex items-center gap-4">
                            <div
                              className={cn(
                                "w-16 h-16 rounded-2xl flex items-center justify-center shrink-0 bg-gradient-to-br shadow-xl border border-white/20",
                                activePreviewItem.gradient,
                              )}
                            >
                              {React.createElement(activePreviewItem.icon, {
                                className: "w-8 h-8 text-white drop-shadow-md",
                              })}
                            </div>
                            <div className="flex flex-col min-w-0 flex-1">
                              <span className="text-base font-bold text-white truncate">
                                {activePreviewItem.titleKey
                                  ? t(activePreviewItem.titleKey, {
                                      defaultValue: activePreviewItem.titleFallback,
                                    })
                                  : activePreviewItem.titleFallback}
                              </span>
                              <span className="text-xs text-primary-300 font-medium capitalize mt-0.5">
                                {activePreviewItem.category === "app"
                                  ? t("dsmLauncher.badges.app", "App")
                                  : t("dsmLauncher.badges.action", "Thao tác")}
                              </span>
                            </div>
                          </div>

                          {/* Description */}
                          <p className="text-xs text-slate-300 leading-relaxed bg-black/20 p-3 rounded-2xl border border-white/5">
                            {(activePreviewItem.descriptionKey
                              ? t(activePreviewItem.descriptionKey, {
                                  defaultValue: activePreviewItem.descriptionFallback,
                                })
                              : activePreviewItem.descriptionFallback) ||
                              t(
                                "dsmLauncher.preview.selectHint",
                                "Chọn một mục từ danh sách kết quả để xem chi tiết",
                              )}
                          </p>

                          {/* Details Metadata: Route & Permissions */}
                          <div className="space-y-3 text-xs">
                            <div className="flex items-center gap-2 text-slate-300">
                              <Compass className="w-4 h-4 text-primary-400 shrink-0" />
                              <span className="text-slate-400">
                                {t("dsmLauncher.preview.route", "Đường dẫn")}:
                              </span>
                              <code className="text-[11px] font-mono text-primary-300 px-2 py-0.5 rounded-lg bg-white/10">
                                {activePreviewItem.to}
                              </code>
                            </div>

                            <div className="flex items-start gap-2 text-slate-300">
                              <ShieldCheck className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
                              <div className="flex flex-col gap-1 min-w-0 flex-1">
                                <span className="text-slate-400">
                                  {t("dsmLauncher.preview.permissions", "Quyền truy cập")}:
                                </span>
                                {activePreviewItem.permissions &&
                                activePreviewItem.permissions.length > 0 ? (
                                  <div className="flex flex-wrap gap-1">
                                    {activePreviewItem.permissions.map((p) => (
                                      <span
                                        key={p}
                                        className="text-[10px] px-1.5 py-0.5 rounded-md bg-emerald-500/15 text-emerald-300 border border-emerald-500/30 font-mono"
                                      >
                                        {p}
                                      </span>
                                    ))}
                                  </div>
                                ) : (
                                  <span className="text-[11px] text-slate-400 italic">
                                    {t(
                                      "dsmLauncher.preview.noPermissionRequired",
                                      "Không yêu cầu quyền đặc biệt",
                                    )}
                                  </span>
                                )}
                              </div>
                            </div>
                          </div>

                          {/* Open Action Button */}
                          <div className="space-y-2 pt-2">
                            <Button
                              onClick={() => handleSelectItem(activePreviewItem)}
                              className="w-full h-11 bg-gradient-to-r from-primary-600 to-indigo-600 hover:from-primary-500 hover:to-indigo-500 text-white rounded-xl font-semibold text-xs shadow-lg flex items-center justify-center gap-2 transition-all cursor-pointer"
                            >
                              <span>
                                {t("dsmLauncher.preview.openAction", "Mở tính năng")}
                              </span>
                              <CornerDownLeft className="w-4 h-4" />
                            </Button>
                            <p className="text-[10px] text-center text-slate-400">
                              {t("dsmLauncher.preview.pressEnter", "Nhấn ↵ để truy cập")}
                            </p>
                          </div>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              )}
            </div>

            {/* Footer status / navigation hints */}
            <div className="px-4 sm:px-12 py-3 sm:py-3.5 border-t border-white/10 bg-white/5 flex items-center justify-between text-xs text-slate-400 shrink-0">
              <div className="hidden sm:flex items-center gap-4">
                <span className="flex items-center gap-1.5">
                  <kbd className="px-2 py-0.5 rounded bg-white/10 border border-white/10 text-[10px]">
                    ↑
                  </kbd>
                  <kbd className="px-2 py-0.5 rounded bg-white/10 border border-white/10 text-[10px]">
                    ↓
                  </kbd>
                  <span className="hidden md:inline">
                    {t("dsmLauncher.hints.navigate", "di chuyển")}
                  </span>
                </span>
                <span className="flex items-center gap-1.5">
                  <kbd className="px-2 py-0.5 rounded bg-white/10 border border-white/10 text-[10px]">
                    ↵
                  </kbd>
                  <span className="hidden md:inline">
                    {t("dsmLauncher.hints.select", "chọn")}
                  </span>
                </span>
                <span className="flex items-center gap-1.5">
                  <kbd className="px-2 py-0.5 rounded bg-white/10 border border-white/10 text-[10px]">
                    ESC
                  </kbd>
                  <span className="hidden md:inline">
                    {t("dsmLauncher.hints.exit", "thoát")}
                  </span>
                </span>
              </div>

              {/* Mobile text-only indication */}
              <div className="sm:hidden text-[11px] text-slate-400">
                LeeAttend OS Launcher
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
