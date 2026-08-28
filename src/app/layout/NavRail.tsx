import { NavLink, useLocation } from "react-router-dom";
import { SECTIONS, sectionForPath } from "@/app/router/routes";
import { applyTheme, useUiStore } from "@/shared/lib/ui-store";
import { Icon } from "@/shared/ui/Icon";

/**
 * Rail de navigation de premier niveau.
 *
 * Largeur et pastilles des maquettes (86 px). Sous 1200 px le libellé disparaît
 * (guide SPECDESIGN, section 7) ; l'infobulle et `aria-label` restent.
 * Le basculeur clair/sombre reprend le pied de rail des maquettes.
 */
export function NavRail() {
  const { pathname } = useLocation();
  const active = sectionForPath(pathname);
  const theme = useUiStore((state) => state.theme);
  const setTheme = useUiStore((state) => state.setTheme);
  const sombre =
    theme === "dark" ||
    (theme === "system" &&
      typeof window !== "undefined" &&
      typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches);

  return (
    <nav
      aria-label="Navigation principale"
      className="flex w-[72px] flex-none flex-col items-center border-r border-line bg-surface-alt py-3.5 min-[1200px]:w-[86px]"
    >
      <Icon name="workspace_premium" size={22} className="mb-4 text-accent" />
      <div className="flex min-h-0 flex-1 flex-col gap-[3px] self-stretch px-2">
        {SECTIONS.map((section) => {
          const isActive = section.key === active.key;
          return (
            <NavLink
              key={section.key}
              to={section.routes[0]!.path}
              title={section.longLabel}
              aria-label={section.longLabel}
              aria-current={isActive ? "page" : undefined}
              className={[
                "flex min-h-11 flex-col items-center justify-center gap-1 rounded-[10px] px-1 py-2",
                "transition-[background-color,color] duration-150",
                "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
                isActive
                  ? "bg-accent-tint text-accent"
                  : "text-ink-faint hover:bg-neutral-tint hover:text-ink",
              ].join(" ")}
            >
              <Icon name={section.icon} size={21} filled={isActive} />
              <span className="hidden max-w-full truncate text-[10px] font-medium min-[1200px]:block">
                {section.shortLabel}
              </span>
            </NavLink>
          );
        })}
      </div>
      <button
        type="button"
        aria-label={sombre ? "Passer en thème clair" : "Passer en thème sombre"}
        onClick={() => {
          const suivant = sombre ? "light" : "dark";
          setTheme(suivant);
          applyTheme(suivant);
        }}
        className="mx-2 mt-1 flex min-h-11 w-[calc(100%-16px)] flex-col items-center justify-center gap-1.5 rounded-[10px] py-2 text-ink-faint transition-colors duration-150 hover:bg-neutral-tint hover:text-ink"
      >
        <Icon name={sombre ? "light_mode" : "dark_mode"} size={19} />
        <span className="hidden text-[10px] font-medium min-[1200px]:block">
          {sombre ? "Clair" : "Sombre"}
        </span>
      </button>
    </nav>
  );
}
