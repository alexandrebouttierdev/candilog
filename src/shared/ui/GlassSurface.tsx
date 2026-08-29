import type { HTMLAttributes } from "react";
import { cn } from "@/shared/lib/cn";

type GlassVariant = "rail" | "subnav" | "topbar" | "inspector" | "popover" | "menu" | "palette" | "modal";

const VARIANTS: Record<GlassVariant, string> = {
  rail: "glass-rail border-r border-glass-rail",
  subnav: "glass-subnav border-r border-glass-subnav",
  topbar: "glass-topbar border-b border-glass-topbar",
  inspector: "glass-inspector border-l border-glass-inspector",
  popover: "glass-popover border border-overlay",
  menu: "glass-menu border border-overlay",
  palette: "glass-palette border border-overlay-strong",
  modal: "glass-modal border border-overlay",
};

/** Surface glass avec repli opaque si le blur est indisponible. */
export function GlassSurface({
  variant,
  className,
  children,
  ...props
}: HTMLAttributes<HTMLElement> & { variant: GlassVariant }) {
  return (
    <div className={cn(VARIANTS[variant], className)} {...props}>
      {children}
    </div>
  );
}
