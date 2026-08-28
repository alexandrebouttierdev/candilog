import { NavLink, useLocation } from "react-router-dom";
import { sectionForPath } from "@/app/router/routes";
import { Icon } from "@/shared/ui/Icon";

/**
 * Onglets contextuels de la section active.
 *
 * Barre de 46 px, pastilles de 30 px, gouttière de 3 px : la géométrie des maquettes
 * SPECDESIGN, identique sur tous les écrans. La barre est rendue même lorsque la section
 * n'a qu'un écran — c'est le chrome constant des maquettes, et son emplacement droit
 * porte la recherche ou la note de contexte de chaque écran.
 */
export function ContextTabs({ slotRef }: { slotRef: (node: HTMLElement | null) => void }) {
  const { pathname } = useLocation();
  const section = sectionForPath(pathname);

  return (
    <div className="flex h-[46px] flex-none items-center gap-[3px] border-b border-line bg-surface px-5">
      <div role="tablist" aria-label={section.longLabel} className="flex items-center gap-[3px]">
        {section.routes.map((route) => {
          const selected =
            route.path === "/" ? pathname === "/" : pathname.startsWith(route.path);
          return (
            <NavLink
              key={route.path}
              to={route.path}
              role="tab"
              aria-selected={selected}
              className={[
                "flex h-tab items-center gap-[7px] rounded-button px-3 text-body font-medium",
                "transition-colors duration-[120ms]",
                "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
                selected ? "bg-accent-tint text-accent" : "text-ink-muted hover:bg-neutral-tint",
              ].join(" ")}
            >
              <Icon name={route.icon} size={17} />
              {route.label}
            </NavLink>
          );
        })}
      </div>
      <span className="flex-1" />
      <div ref={slotRef} className="flex flex-none items-center gap-2" />
    </div>
  );
}
