import { GripVertical, X, RotateCcw, Loader2 } from "lucide-react";
import React from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";

import {
    DndContext,
    closestCenter,
    KeyboardSensor,
    PointerSensor,
    useSensor,
    useSensors,
    type DragEndEvent,
} from "@dnd-kit/core";
import {
    arrayMove,
    SortableContext,
    sortableKeyboardCoordinates,
    useSortable,
    verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";

import type { WidgetId } from "@/hooks/use-dashboard-layout";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

// ── Types ──────────────────────────────────────────────────────────

interface CustomizePanelProps {
    allWidgets: { id: WidgetId; labelKey: string }[];
    isOpen: boolean;
    isSaving?: boolean;
    onClose: () => void;
    onReorder: (order: WidgetId[]) => void;
    onReset: () => void;
    onToggleWidget: (id: WidgetId) => void;
    visibleWidgets: Record<WidgetId, boolean>;
    widgetOrder: WidgetId[];
}

// ── Sortable Widget Row ────────────────────────────────────────────

function SortableWidgetRow({
    widget,
    isVisible,
    onToggle,
}: {
    widget: { id: WidgetId; labelKey: string };
    isVisible: boolean;
    onToggle: (id: WidgetId) => void;
}) {
    const { t } = useTranslation();
    const {
        attributes,
        listeners,
        setNodeRef,
        transform,
        transition,
        isDragging,
    } = useSortable({ id: widget.id });

    const style = {
        transform: CSS.Transform.toString(transform),
        transition,
        zIndex: isDragging ? 50 : undefined,
    };

    return (
        <div
            className={cn(
                "flex items-center gap-3 px-4 py-3 rounded-2xl border transition-all",
                isDragging
                    ? "bg-primary-50 dark:bg-primary-950 border-primary-200 dark:border-primary-800 shadow-xl"
                    : "bg-white dark:bg-slate-800 border-slate-100 dark:border-slate-700 hover:border-slate-200 dark:hover:border-slate-600"
            )}
            ref={setNodeRef}
            style={style}
        >
            {/* Drag handle */}
            <button
                {...attributes}
                {...listeners}
                aria-label="Drag to reorder"
                className="cursor-grab active:cursor-grabbing text-slate-300 dark:text-slate-600 hover:text-slate-500 dark:hover:text-slate-400 touch-none"
            >
                <GripVertical size={16} />
            </button>

            {/* Label */}
            <span className="flex-1 text-sm font-medium text-slate-700 dark:text-slate-200 truncate">
                {t(widget.labelKey)}
            </span>

            {/* Toggle */}
            <button
                aria-checked={isVisible}
                className={cn(
                    "relative w-10 h-6 rounded-full transition-colors shrink-0",
                    isVisible
                        ? "bg-primary-600 dark:bg-primary-500"
                        : "bg-slate-200 dark:bg-slate-600"
                )}
                onClick={() => { onToggle(widget.id); }}
                role="switch"
            >
                <span
                    className={cn(
                        "absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow-sm transition-transform",
                        isVisible && "translate-x-4"
                    )}
                />
            </button>
        </div>
    );
}

// ── Main Panel ─────────────────────────────────────────────────────

function CustomizePanelBackdrop({ onClose }: { onClose: () => void }) {
    return (
        <div
            className="fixed inset-0 bg-black/30 dark:bg-black/50 backdrop-blur-sm z-40 transition-opacity"
            onClick={onClose}
            onKeyDown={(e) => {
                if (e.key === 'Escape') onClose();
            }}
            role="presentation"
        />
    );
}

interface CustomizePanelHeaderProps {
    isSaving?: boolean;
    onClose: () => void;
    t: (key: string) => string;
}

function CustomizePanelHeader({ onClose, isSaving, t }: CustomizePanelHeaderProps) {
    return (
        <div className="flex items-center justify-between px-6 py-5 border-b border-slate-100 dark:border-slate-800">
            <div className="flex items-center gap-3">
                <Typography className="text-lg font-bold text-slate-700 dark:text-slate-200" variant="h2">
                    {t("dashboard.customize.title")}
                </Typography>
                {!!isSaving && <Loader2 className="animate-spin text-primary-500" size={16} />}
            </div>
            <button
                className="p-2 rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
                onClick={onClose}
            >
                <X className="text-slate-500" size={18} />
            </button>
        </div>
    );
}

interface CustomizePanelFooterProps {
    onReset: () => void;
    t: (key: string) => string;
}

function CustomizePanelFooter({ onReset, t }: CustomizePanelFooterProps) {
    return (
        <div className="px-6 py-5 border-t border-slate-100 dark:border-slate-800 space-y-3">
            <button
                className="w-full flex items-center justify-center gap-2 px-4 py-2.5 text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest hover:text-slate-700 dark:hover:text-slate-200 transition-colors"
                onClick={onReset}
            >
                <RotateCcw size={14} />
                {t("dashboard.customize.reset")}
            </button>
        </div>
    );
}

export default function CustomizePanel({
    isOpen,
    onClose,
    visibleWidgets,
    widgetOrder,
    allWidgets,
    onToggleWidget,
    onReorder,
    onReset,
    isSaving,
}: CustomizePanelProps) {
    const { t } = useTranslation();
    const sensors = useSensors(
        useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
        useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
    );

    // Get ordered items — include all widgets even if not in current order
    const orderedWidgets = (() => {
        const ordered = widgetOrder
            .map((id) => allWidgets.find((w) => w.id === id))
            .filter(Boolean) as typeof allWidgets;

        // Add any missing widgets at the end
        const missing = allWidgets.filter(
            (w) => !widgetOrder.includes(w.id)
        );
        return [...ordered, ...missing];
    })();

    const handleDragEnd = (event: DragEndEvent) => {
        const { active, over } = event;
        if (!over || active.id === over.id) return;

        const oldIndex = orderedWidgets.findIndex((w) => w.id === active.id);
        const newIndex = orderedWidgets.findIndex((w) => w.id === over.id);

        const newOrder = arrayMove(
            orderedWidgets.map((w) => w.id),
            oldIndex,
            newIndex
        );
        onReorder(newOrder);
    };

    if (!isOpen) return null;

    return createPortal(
        <>
            <CustomizePanelBackdrop onClose={onClose} />

            {/* Panel */}
            <aside
                className={cn(
                    "fixed right-0 top-0 h-full w-full sm:w-96 bg-white dark:bg-slate-900 border-l border-slate-200 dark:border-slate-800 shadow-2xl z-50",
                    "flex flex-col",
                    "animate-in slide-in-from-right duration-300"
                )}
            >
                <CustomizePanelHeader isSaving={isSaving} onClose={onClose} t={t} />

                {/* Widget list */}
                <div className="flex-1 overflow-y-auto px-6 py-5 space-y-3">
                    <Typography className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mb-4" variant="p">
                        {t("dashboard.customize.widgets")}
                    </Typography>

                    <DndContext
                        collisionDetection={closestCenter}
                        onDragEnd={handleDragEnd}
                        sensors={sensors}
                    >
                        <SortableContext
                            items={orderedWidgets.map((w) => w.id)}
                            strategy={verticalListSortingStrategy}
                        >
                            {orderedWidgets.map((widget) => (
                                <SortableWidgetRow
                                    isVisible={visibleWidgets[widget.id]}
                                    key={widget.id}
                                    onToggle={onToggleWidget}
                                    widget={widget}
                                />
                            ))}
                        </SortableContext>
                    </DndContext>
                </div>

                <CustomizePanelFooter onReset={onReset} t={t} />
            </aside>
        </>,
        document.body
    );
}
