import { useLocation } from "react-router-dom";
import { sectionForPath } from "@/app/router/routes";
import { Icon } from "@/shared/ui/Icon";

/** Topbar 46 px : titre centré, actions contextuelles à droite. */
export function TopBar({ slotRef }: { slotRef: (node: HTMLElement | null) => void }) {
  const { pathname } = useLocation();
  const section = sectionForPath(pathname);
  const route =
    section.routes.find((r) =>
      r.path === "/" ? pathname === "/" : pathname.startsWith(r.path),
    ) ?? section.routes[0]!;

  return (
    <header className="glass-topbar grid h-topbar flex-none grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-2 border-b border-glass-topbar pr-3 pl-3.5">
      <div className="col-start-2 row-start-1 flex min-w-0 max-w-[min(36vw,16rem)] items-center justify-center gap-2">
        <Icon name={route.icon} size={17} className="flex-none text-ink-disabled" />
        <h1 className="truncate text-section text-ink">{route.label}</h1>
      </div>
      <div
        ref={slotRef}
        className="col-start-3 row-start-1 flex min-w-0 items-center justify-end justify-self-end gap-2"
      />
    </header>
  );
}
