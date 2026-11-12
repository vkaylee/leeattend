import * as React from "react";

import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "../../lib/utils";

const typographyVariants = cva("text-slate-900 dark:text-slate-100", {
  variants: {
    variant: {
      hero: "text-3xl sm:text-4xl lg:text-5xl font-black tracking-tight leading-tight",
      tagline: "text-base sm:text-lg font-medium text-slate-500 dark:text-slate-400 leading-relaxed",
      h1: "text-3xl font-bold tracking-tight",
      h2: "text-2xl font-semibold tracking-tight transition-colors first:mt-0",
      h3: "text-xl font-semibold tracking-tight",
      h4: "text-xl font-semibold tracking-tight",
      h5: "text-lg font-semibold tracking-tight",
      h6: "text-base font-semibold tracking-tight",
      p: "leading-7 [&:not(:first-child)]:mt-6",
      body: "text-base leading-7",
      helper: "text-sm leading-5 text-slate-500 dark:text-slate-400",
      error: "text-sm leading-5 text-rose-600 dark:text-rose-400",
      small: "text-sm font-medium leading-none",
      overline: "text-xs font-bold uppercase tracking-[0.25em]",
      label: "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
    },
    color: {
      default: "",
      gradient: "bg-gradient-to-r from-violet-600 via-indigo-500 to-sky-400 bg-clip-text text-transparent font-extrabold",
      gradientEmerald: "bg-gradient-to-r from-emerald-600 via-teal-500 to-cyan-400 bg-clip-text text-transparent font-extrabold",
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
  hero: "h1",
  tagline: "p",
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
  body: "p",
  helper: "p",
  error: "p",
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

export interface GradientTextProps extends React.HTMLAttributes<HTMLSpanElement> {
  from?: string;
  via?: string;
  to?: string;
}

/**
 * Clean VitePress-inspired gradient text clip component
 * Replicates the hero name clip styling from nginxproxymanager.com
 */
export const GradientText = React.forwardRef<HTMLSpanElement, GradientTextProps>(
  (
    {
      className,
      from = "from-violet-600",
      via = "via-indigo-500",
      to = "to-sky-400",
      children,
      ...props
    },
    ref
  ) => (
    <span
      className={cn(
        "bg-gradient-to-r bg-clip-text text-transparent font-extrabold",
        from,
        via,
        to,
        className
      )}
      ref={ref}
      {...props}
    >
      {children}
    </span>
  )
);
GradientText.displayName = "GradientText";
