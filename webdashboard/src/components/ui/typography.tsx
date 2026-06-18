import * as React from "react";

import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "../../lib/utils";

const typographyVariants = cva("text-slate-900 dark:text-slate-100", {
  variants: {
    variant: {
      h1: "text-2xl font-bold tracking-tight",
      h2: "text-3xl font-semibold tracking-tight transition-colors first:mt-0",
      h3: "text-2xl font-semibold tracking-tight",
      h4: "text-xl font-semibold tracking-tight",
      h5: "text-lg font-semibold tracking-tight",
      h6: "text-base font-semibold tracking-tight",
      p: "leading-7 [&:not(:first-child)]:mt-6",
      small: "text-sm font-medium leading-none",
      overline: "text-xs font-bold uppercase tracking-[0.25em]",
      label: "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
    },
    color: {
      default: "",
      primary: "text-primary-700 dark:text-primary-400",
      secondary: "text-secondary-700 dark:text-secondary-400",
      muted: "text-slate-500 dark:text-slate-400",
      destructive: "text-rose-600 dark:text-rose-400",
      success: "text-emerald-600 dark:text-emerald-400",
    },
    align: {
      left: "text-left",
      center: "text-center",
      right: "text-right",
    },
    noWrap: {
      true: "truncate",
    },
  },
  defaultVariants: {
    variant: "p",
    color: "default",
    align: "left",
  },
});

export interface TypographyProps
  extends Omit<React.HTMLAttributes<HTMLElement>, "color">,
    VariantProps<typeof typographyVariants> {
  as?: React.ElementType;
}

const variantElementMap: Record<NonNullable<VariantProps<typeof typographyVariants>["variant"]>, React.ElementType> = {
  h1: "h1",
  h2: "h2",
  h3: "h3",
  h4: "h4",
  h5: "h5",
  h6: "h6",
  p: "p",
  small: "small",
  overline: "span",
  label: "label",
};

export const Typography = React.forwardRef<HTMLElement, TypographyProps>(
  ({ className, variant, color, align, noWrap, as, ...props }, ref) => {
    const Comp = as ?? (variant ? variantElementMap[variant] : "p");
    
    return (
      <Comp
        className={cn(typographyVariants({ variant, color, align, noWrap, className }))}
        ref={ref}
        {...props}
      />
    );
  }
);
Typography.displayName = "Typography";
