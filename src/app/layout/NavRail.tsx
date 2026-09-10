import { NavLink, useLocation } from "react-router-dom";
import { Sections, sectionForPath } from "@/app/router/routes";
import { applyTheme, useShellBrand, useUiStore } from "@/shared/lib/ui-store";
import type { ThemePref } from "@/shared/types/generated/settings";
import { settingsService } from "@/features/settings/services/settingsService";
import { Icon } from "@/shared/ui/Icon";
import { cn } from "@/shared/lib/cn";
import logoCandilog from "@/assets/logo-candilog.svg";
import logoCandilogDark from "@/assets/logo-candilog-dark.svg";

/** Rail compact : items 42×36, tooltip immédiat. */
export function NavRail() {
  const { pathname } = useLocation();
  const active = sectionForPath(pathname);
  const setTheme = useUiStore((state) => state.setTheme);
  const railBrand = useShellBrand();
  const sombre = !railBrand;

  return (
    <nav
      aria-label="Navigation principale"
      className="glass-rail relative z-20 flex w-rail flex-none flex-col items-center overflow-visible border-r border-glass-rail pt-3 pb-2.5"
    >
      <span
        className={cn(
          "mb-3 flex size-9 items-center justify-center",
          railBrand && "rounded-tile bg-white/95 p-1 shadow-sm",
        )}
      >
        <img
          src={sombre ? logoCandilogDark : logoCandilog}
          alt="Candilog"
          width={36}
          height={36}
          className={cn(railBrand ? "size-7" : "size-9")}
        />
      </span>
      <span
        aria-hidden
        className={cn("mb-2.5 h-px w-6", railBrand ? "bg-rail-divider" : "bg-line")}
      />
      <div className="flex min-h-0 flex-1 flex-col gap-1 self-stretch overflow-visible px-[13px]">
        {Sections.map((section) => {
          const isActive = section.key === active.key;
          return (
            <div key={section.key} className="group relative flex justify-center">
              <NavLink
                to={section.routes[0]!.path}
                title={section.long_label}
                aria-label={section.long_label}
                aria-current={isActive ? "page" : undefined}
                className={cn(
                  "flex h-9 w-[42px] flex-none items-center justify-center rounded-tile",
                  "transition-colors duration-hover ease-in-out",
                  railBrand
                    ? "focus-visible:outline-1 focus-visible:outline-rail-focus"
                    : "focus-visible:outline-1 focus-visible:outline-accent-focus",
                  isActive
                    ? railBrand
                      ? "border border-rail-item-active-border bg-rail-item-active-bg text-rail-ink-hover"
                      : "border border-accent-border bg-accent-tint text-accent-hover"
                    : railBrand
                      ? "text-rail-ink hover:bg-rail-item-hover hover:text-rail-ink-hover"
                      : "text-ink-subtle hover:bg-surface-hover hover:text-ink-muted",
                )}
              >
                <Icon name={section.icon} size={20} />
              </NavLink>
              <span className="pointer-events-none absolute top-1/2 left-full z-[60] ml-2 flex -translate-y-1/2 items-center gap-2.5 rounded-button border border-overlay bg-[var(--candilog-glass-menu)] px-2.5 py-1.5 whitespace-nowrap opacity-0 shadow-menu backdrop-blur-[14px] group-hover:opacity-100">
                <span className="text-note whitespace-nowrap text-ink">{section.long_label}</span>
              </span>
            </div>
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
        className={cn(
          "mt-1 flex h-9 w-[42px] flex-none items-center justify-center rounded-tile transition-colors duration-hover",
          railBrand
            ? "text-rail-ink hover:bg-rail-item-hover hover:text-rail-ink-hover"
            : "text-ink-subtle hover:bg-surface-hover hover:text-ink-muted",
        )}
      >
        <Icon name={sombre ? "dark_mode" : "light_mode"} size={20} />
      </button>
    </nav>
  );
}
