import { NavLink, useLocation } from "react-router-dom";
import { sectionForPath } from "@/app/router/routes";
import { Icon } from "@/shared/ui/Icon";

/**
 * Onglets contextuels de la section active.
 *
 * Masqués lorsque la section n'a qu'un seul écran : une barre d'onglets à un onglet
 * n'apporte rien et vole 40 px de hauteur utile.
 */
export function ContextTabs() {
  const { pathname } = useLocation();
  const section = sectionForPath(pathname);

  if (section.routes.length < 2) return null;

  return (
    <div
      role="tablist"
      aria-label={section.longLabel}
      className="flex flex-none items-center gap-1 border-b border-line bg-surface-alt px-7"
    >
      {section.routes.map((route) => (
        <NavLink
          key={route.path}
          to={route.path}
          role="tab"
          className={({ isActive }) =>
            [
              "flex min-h-10 items-center gap-1.5 border-b-2 px-3 text-body",
              "transition-[color,border-color] duration-150",
              isActive
                ? "border-accent font-medium text-accent"
                : "border-transparent text-ink-muted hover:text-ink",
            ].join(" ")
          }
        >
          <Icon name={route.icon} size={16} />
          {route.label}
        </NavLink>
      ))}
    </div>
  );
}
