import { useLocation } from "react-router-dom";
import { sectionForPath } from "@/app/router/routes";
import { CommandPaletteTrigger } from "@/shared/ui/CommandPalette";
import { Icon } from "@/shared/ui/Icon";

/** Topbar 46 px : icône de section, titre, recherche globale. */
export function TopBar({
  slotRef,
  onOpenPalette,
}: {
  slotRef: (node: HTMLElement | null) => void;
  onOpenPalette: () => void;
}) {
  const { pathname } = useLocation();
  const section = sectionForPath(pathname);
  const route =
    section.routes.find((r) =>
      r.path === "/" ? pathname === "/" : pathname.startsWith(r.path),
    ) ?? section.routes[0]!;

  return (
    <header className="glass-topbar flex h-topbar flex-none items-center gap-2.5 border-b border-glass-topbar pr-3 pl-3.5">
      <Icon name={route.icon} size={17} className="flex-none text-ink-disabled" />
      <h1 className="min-w-0 truncate text-section text-ink">{route.label}</h1>
      <CommandPaletteTrigger onClick={onOpenPalette} />
      <span className="flex-1" />
      <div ref={slotRef} className="flex flex-none items-center gap-2" />
    </header>
  );
}
