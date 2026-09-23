import { NavLink, useLocation } from "react-router-dom";
import { cn } from "@/shared/lib/cn";
import { useChromeValue } from "@/shared/lib/chrome";
import { currentPlatform } from "@/shared/lib/platform";
import { useUiStore } from "@/shared/lib/ui-store";
import { activeTab, destinationForPath } from "@/app/router/routes";
import { SETTINGS_LABELS } from "@/app/overlays/settingsSections";

/**
 * Barre de titre de 40 px (`reference_design/PLATFORM.md` C1) : fil d'Ariane à gauche,
 * onglets de vue et mention d'écran à droite.
 *
 * Sous macOS, la fenêtre est en `titleBarStyle: Overlay` : les feux natifs se posent sur
 * cette barre, qui leur réserve 68 px à gauche. Sous Windows et Linux, la barre système est
 * conservée (décision D5) et cette barre se place dessous, sans réserve. Toute la barre sert
 * de zone de glissement, sauf les onglets.
 */
export function TitleBar() {
  const { pathname } = useLocation();
  const destination = destinationForPath(pathname);
  const tab = activeTab(destination, pathname);
  const chrome = useChromeValue();
  const settings = useUiStore((state) => state.settings);
  const mac = currentPlatform() === "mac";

  const root = settings ? "Réglages" : destination.label;
  const leaf = settings ? SETTINGS_LABELS[settings] : (chrome.crumb ?? tab?.label);

  return (
    <header
      data-tauri-drag-region
      className={cn(
        "flex h-titlebar flex-none items-center gap-3 pr-[13px] select-none",
        mac ? "pl-[76px]" : "pl-[13px]",
      )}
    >
      <nav aria-label="Fil d'Ariane" data-tauri-drag-region className="flex min-w-0 items-center gap-2 text-ui">
        <span data-tauri-drag-region className="whitespace-nowrap text-tx-4">
          {root}
        </span>
        {leaf ? (
          <>
            <span aria-hidden data-tauri-drag-region className="text-tx-7">
              ›
            </span>
            <span data-tauri-drag-region aria-current="page" className="truncate font-medium text-tx">
              {leaf}
            </span>
          </>
        ) : null}
      </nav>
      <span data-tauri-drag-region className="h-full min-w-4 flex-1" />
      {!settings && destination.tabs ? (
        <div role="tablist" aria-label="Vues" className="flex flex-none items-center gap-px">
          {destination.tabs.map((item) => {
            const active = item.path === tab?.path;
            return (
              <NavLink
                key={item.path}
                to={item.path}
                role="tab"
                aria-selected={active}
                className={cn(
                  "rounded-r5 px-[9px] py-1 text-small transition-color hover:bg-elev",
                  active ? "bg-elev font-medium text-tx" : "text-tx-4",
                )}
              >
                {item.label}
              </NavLink>
            );
          })}
        </div>
      ) : null}
      {chrome.aside && !settings ? (
        <span className="inline-flex h-6 flex-none items-center rounded-r6 bg-elev px-[9px] text-small whitespace-nowrap text-tx-4">
          {chrome.aside}
        </span>
      ) : null}
    </header>
  );
}
