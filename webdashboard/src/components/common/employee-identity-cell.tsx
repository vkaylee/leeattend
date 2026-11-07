import { Briefcase } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

export interface EmployeeIdentityCellProps {
  avatarSize?: "sm" | "md";
  className?: string;
  code?: string | null;
  employerName?: string | null;
  name: string;
  onClick?: () => void;
  showAvatar?: boolean;
}

export function getEmployeeInitials(name: string): string {
  if (!name.trim()) return "?";
  const parts = name.trim().split(/\s+/);
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
}

export const EmployeeIdentityCell: React.FC<EmployeeIdentityCellProps> = ({
  avatarSize = "md",
  className,
  code,
  employerName,
  name,
  onClick,
  showAvatar = true,
}) => {
  const initials = getEmployeeInitials(name);
  const sizeClasses =
    avatarSize === "sm" ? "h-7 w-7 text-[11px] rounded-lg" : "h-8 w-8 text-xs rounded-xl";

  const content = (
    <div className={cn("flex items-center gap-2.5 min-w-0 text-left", className)}>
      {showAvatar && (
        <div
          aria-hidden="true"
          className={cn(
            "flex shrink-0 items-center justify-center font-bold uppercase select-none border border-primary-200/80 bg-primary-50 text-primary-700 dark:border-primary-800 dark:bg-primary-950/80 dark:text-primary-300",
            sizeClasses
          )}
        >
          {initials}
        </div>
      )}
      <div className="min-w-0 flex-1">
        <div className="truncate text-xs font-bold text-slate-900 dark:text-slate-100 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors">
          {name}
        </div>
        <div className="flex items-center gap-2 text-[10px] text-slate-500 dark:text-slate-400 font-mono">
          {code && <span className="font-semibold text-slate-600 dark:text-slate-300">{code}</span>}
          {employerName && (
            <span className="flex items-center gap-1 font-sans truncate text-slate-400 dark:text-slate-500">
              <Briefcase size={10} />
              {employerName}
            </span>
          )}
        </div>
      </div>
    </div>
  );

  if (onClick) {
    return (
      <Button
        aria-label={`Chi tiết nhân sự: ${name}`}
        className="text-left w-full h-auto p-0 justify-start focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 rounded-lg group"
        onClick={onClick}
        type="button"
        variant="ghost"
      >
        {content}
      </Button>
    );
  }

  return content;
};
