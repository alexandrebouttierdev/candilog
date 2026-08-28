import { NavLink, useLocation } from "react-router-dom";
import { Sections, sectionForPath } from "@/app/router/routes";
import { applyTheme, useUiStore } from "@/shared/lib/ui-store";
import type { ThemePref } from "@/shared/types/generated/settings";
import { settingsService } from "@/features/settings/services/settingsService";
import { Icon } from "@/shared/ui/Icon";
import logoCandilog from "@/assets/logo-candilog.png";

/**
 * Rail de navigation de premier niveau.
 *
 * Géométrie des maquettes : 86 px de large, marque en 28 px, tuiles de 10 px de rayon
 * espacées de 3 px dans une gouttière de 9 px, basculeur de thème collé en bas. Sous
 * 1200 px la largeur tombe à 72 px (guide SPECDESIGN, section 7) mais les libellés
 * restent : ils tiennent, et un rail d'icônes muettes est illisible.
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
      className="flex w-[72px] flex-none flex-col items-center overflow-hidden border-r border-line bg-surface-alt pt-[18px] pb-3.5 min-[1200px]:w-[86px]"
    >
      <img src={logoCandilog} alt="Candilog" width={28} height={28} className="mb-4 size-7" />
      <div className="flex min-h-0 flex-1 flex-col gap-[3px] self-stretch px-[9px]">
        {Sections.map((section) => {
          const isActive = section.key === active.key;
          return (
            <NavLink
              key={section.key}
              to={section.routes[0]!.path}
              title={section.long_label}
              aria-label={section.long_label}
              aria-current={isActive ? "page" : undefined}
              className={[
                "flex min-w-0 flex-col items-center gap-[5px] rounded-tile px-1 pt-[9px] pb-[7px]",
                "transition-colors duration-[120ms]",
                "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
                isActive ? "bg-accent-tint text-accent" : "text-ink-faint hover:bg-neutral-tint",
              ].join(" ")}
            >
              <Icon name={section.icon} size={21} />
              <span className="max-w-full truncate text-micro font-mid">{section.short_label}</span>
            </NavLink>
          );
        })}
      </div>
      <button
        type="button"
        title={sombre ? "Passer en thème clair" : "Passer en thème sombre"}
        aria-label={sombre ? "Passer en thème clair" : "Passer en thème sombre"}
        onClick={() => {
          const suivant: ThemePref = sombre ? "light" : "dark";
          setTheme(suivant);
          applyTheme(suivant);
          void settingsService
            .load()
            .then((settings) => settingsService.save({ ...settings, theme: suivant }))
            .catch(() => {
              /* Revue navigateur sans backend : le thème reste en session. */
            });
        }}
        className="mx-[9px] flex flex-col items-center gap-1.5 self-stretch rounded-tile py-[9px] text-ink-faint transition-colors duration-[120ms] hover:bg-neutral-tint"
      >
        <Icon name={sombre ? "dark_mode" : "light_mode"} size={19} />
        <span className="max-w-full truncate text-micro font-mid">
          {sombre ? "Sombre" : "Clair"}
        </span>
      </button>
    </nav>
  );
}
