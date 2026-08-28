import { NavLink, useLocation } from "react-router-dom";
import { sectionForPath } from "@/app/router/routes";
import { Icon } from "@/shared/ui/Icon";

/**
 * Onglets contextuels de la section active — pastilles 30 px, barre 46 px,
 * comme dans les maquettes SPECDESIGN (Dashboard, Suivi, Documents…).
 *
 * Masqués lorsque la section n'a qu'un seul écran : une barre à un onglet
 * n'apporte rien et vole de la hauteur utile.
 */
export function ContextTabs() {
  const { pathname } = useLocation();
  const section = sectionForPath(pathname);

  if (section.routes.length < 2) return null;

  return (
    <div
      role="tablist"
      aria-label={section.longLabel}
      className="flex h-[46px] flex-none items-center gap-[3px] border-b border-line bg-surface px-5"
    >
      {section.routes.map((route) => {
        const selected = pathname === route.path;
        return (
          <NavLink
            key={route.path}
            to={route.path}
            role="tab"
            aria-selected={selected}
            className={[
              "flex h-[30px] items-center gap-1.5 rounded-button px-3 text-body font-medium",
              "transition-[background-color,color] duration-150",
              "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
              selected ? "bg-accent-tint text-accent" : "text-ink-muted hover:bg-neutral-tint hover:text-ink",
            ].join(" ")}
          >
            <Icon name={route.icon} size={17} filled={selected} />
            {route.label}
          </NavLink>
        );
      })}
    </div>
  );
}
