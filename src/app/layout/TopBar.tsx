import { useShellBrand } from "@/shared/lib/ui-store";
import { cn } from "@/shared/lib/cn";
import { AiGlobalHeader } from "./AiGlobalHeader";

/** Topbar 46 px : actions IA et accessoire contextuel à droite. */
export function TopBar({ slotRef }: { slotRef: (node: HTMLElement | null) => void }) {
  const shellBrand = useShellBrand();

  return (
    <header
      data-shell-brand={shellBrand ? "" : undefined}
      className="glass-topbar flex h-topbar flex-none items-center justify-end gap-2 border-b border-glass-topbar pr-3 pl-3.5"
    >
      <AiGlobalHeader shellBrand={shellBrand} />
      <div
        ref={slotRef}
        className={cn(
          "flex min-w-0 items-center gap-2",
          shellBrand && "[&_.text-ink-faint]:text-rail-ink [&_.text-ink]:text-rail-ink-hover",
        )}
      />
    </header>
  );
}
