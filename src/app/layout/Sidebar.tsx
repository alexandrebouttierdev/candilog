import { NavLink, useLocation } from "react-router-dom";
import { cn } from "@/shared/lib/cn";
import { useUiStore } from "@/shared/lib/ui-store";
import { BrandMark, LineIcon } from "@/shared/ui";
import { formatShortcut } from "@/shared/lib/platform";
import { DESTINATIONS, destinationForPath } from "@/app/router/routes";
import type { DestinationKey } from "@/app/router/routes";
import { PATHS } from "@/shared/lib/paths";
import { useNavCounts } from "./useNavCounts";
import { useAiIndicator } from "./useAiIndicator";

const PUCE = { local: "bg-st-g", remote: "bg-st-a", none: "bg-tx-7" } as const;

/**
 * Barre de navigation (`COMPONENTS.md` §8) : 202 px, réduite à 52 px — icônes seules, libellé
 * en infobulle — sous le palier de 1060 px. Marque et accès à la palette en tête, six
 * destinations avec leur décompte, puis Réglages et l'indicateur d'IA en pied.
 */
export function Sidebar() {
  const { pathname } = useLocation();
  const active = destinationForPath(pathname);
  const settingsOpen = useUiStore((state) => state.settings !== null);
  const openSettings = useUiStore((state) => state.openSettings);
  const setPalette = useUiStore((state) => state.setPalette);
  const counts = useNavCounts();
  const ai = useAiIndicator();

  const count = (key: DestinationKey): { value: string; tone: string } | null => {
    switch (key) {
      case "today":
        return counts.today ? { value: String(counts.today), tone: "text-ac-tx" } : null;
      case "applications":
        return counts.applications === undefined ? null : { value: String(counts.applications), tone: "text-tx-6" };
      case "relations":
        return counts.relations === undefined ? null : { value: String(counts.relations), tone: "text-tx-6" };
      case "documents":
        return counts.documents === undefined ? null : { value: String(counts.documents), tone: "text-tx-6" };
      case "profile":
        // Un profil incomplet n'est pas une erreur : ambre tant qu'il manque quelque chose,
        // neutre à 100 % (`DECISIONS.md` B11).
        return counts.profile === undefined
          ? null
          : { value: `${counts.profile} %`, tone: counts.profile < 100 ? "text-st-a" : "text-tx-6" };
      case "ai":
        return null;
    }
  };

  return (
    <nav
      aria-label="Navigation principale"
      className="flex w-nav-sm flex-none flex-col overflow-hidden px-[7px] pt-0.5 wide:w-nav"
    >
      <div className="mb-1.5 flex h-[31px] flex-none items-center justify-center gap-2 px-[7px] wide:justify-start">
        <BrandMark />
        <span className="hidden text-row font-medium tracking-[-0.01em] whitespace-nowrap text-tx wide:inline">
          Candilog
        </span>
        <button
          type="button"
          onClick={() => setPalette(true)}
          aria-label="Ouvrir la palette de commandes"
          title="Palette de commandes"
          className="ml-auto hidden h-[19px] min-w-5 items-center justify-center rounded-r5 bg-elev px-1 font-mono text-kbd text-tx-4 hover:text-tx-2 wide:inline-flex"
        >
          <span aria-hidden>{formatShortcut("mod+k")}</span>
        </button>
      </div>

      <ul className="flex flex-col gap-px">
        {DESTINATIONS.map((destination) => {
          const isActive = !settingsOpen && destination.key === active.key;
          const badge = count(destination.key);
          return (
            <li key={destination.key}>
              <NavLink
                to={destination.path}
                title={destination.label}
                aria-current={isActive ? "page" : undefined}
                className={cn(
                  "flex h-nav-item items-center justify-center gap-[9px] rounded-r6 px-2 text-ui transition-color wide:justify-start",
                  isActive ? "bg-sel font-medium text-tx" : "text-tx-3 hover:bg-elev hover:text-tx",
                )}
              >
                <LineIcon name={destination.icon} className={isActive ? "text-ac-tx" : undefined} />
                <span className="hidden truncate wide:inline">{destination.label}</span>
                {badge ? (
                  <span className={cn("ml-auto hidden font-mono text-mono-sm wide:inline", badge.tone)}>
                    {badge.value}
                  </span>
                ) : null}
              </NavLink>
            </li>
          );
        })}
      </ul>

      <div className="mt-auto flex-none pb-2">
        <button
          type="button"
          title="Réglages"
          onClick={() => openSettings()}
          className={cn(
            "flex h-btn w-full items-center justify-center gap-[9px] rounded-r6 px-2 transition-color wide:justify-start",
            settingsOpen ? "bg-sel text-tx" : "text-tx-3 hover:bg-elev hover:text-tx",
          )}
        >
          <LineIcon name="settings" />
          <span className="hidden text-ui whitespace-nowrap wide:inline">Réglages</span>
          <span aria-hidden className="ml-auto hidden font-mono text-kbd text-tx-6 wide:inline">
            {formatShortcut("mod+,")}
          </span>
        </button>
        <NavLink
          to={PATHS.ai}
          title={ai.routing ? `Routage IA · ${ai.routing}` : `${ai.label}${ai.model ? ` · ${ai.model}` : ""}`}
          className="flex h-6 items-center justify-center gap-[9px] rounded-r6 px-2 hover:bg-elev wide:justify-start"
        >
          <span className="flex w-[15px] flex-none justify-center">
            <span aria-hidden className={cn("size-1.5 rounded-full", PUCE[ai.locality])} />
          </span>
          {/* Dès qu'une tâche a son propre modèle, le pied résume le routage plutôt que le
              seul fournisseur principal, qui ne dit plus où vont toutes les données. */}
          <span className="hidden text-tiny whitespace-nowrap text-tx-4 wide:inline">
            {ai.routing ? "Routage IA" : ai.label}
          </span>
          {(ai.routing ?? ai.model) ? (
            <span className="ml-auto hidden truncate font-mono text-kbd text-tx-7 wide:inline">{ai.routing ?? ai.model}</span>
          ) : null}
        </NavLink>
      </div>
    </nav>
  );
}
