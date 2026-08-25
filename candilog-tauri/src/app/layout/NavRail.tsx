import { NavLink, useLocation } from "react-router-dom";
import { SECTIONS, sectionForPath } from "@/app/router/routes";
import { Icon } from "@/shared/ui/Icon";

/**
 * Rail de navigation de premier niveau.
 *
 * Se replie sous 1200 px de large (guide SPECDESIGN, section 7) : le libellé disparaît,
 * l'infobulle prend le relais, et la cible reste à 44 px de haut.
 */
export function NavRail() {
  const { pathname } = useLocation();
  const active = sectionForPath(pathname);

  return (
    <nav
      aria-label="Navigation principale"
      // Seuil 1200 px et non le `xl` de Tailwind (1280 px) : c'est la valeur fixée par le
      // guide pour le repli du rail.
      className="flex w-[76px] flex-none flex-col gap-1 border-r border-line bg-surface-alt px-2 py-3 min-[1200px]:w-[92px]"
    >
      {SECTIONS.map((section) => {
        const isActive = section.key === active.key;
        return (
          <NavLink
            key={section.key}
            to={section.routes[0]!.path}
            title={section.longLabel}
            aria-current={isActive ? "page" : undefined}
            className={[
              "flex min-h-11 flex-col items-center justify-center gap-1 rounded-button px-1 py-2",
              "transition-[background-color,color] duration-150",
              isActive
                ? "bg-accent-tint text-accent"
                : "text-ink-muted hover:bg-neutral-tint hover:text-ink",
            ].join(" ")}
          >
            <Icon name={section.icon} size={20} filled={isActive} />
            <span className="text-[10px] font-medium">{section.shortLabel}</span>
          </NavLink>
        );
      })}
    </nav>
  );
}
