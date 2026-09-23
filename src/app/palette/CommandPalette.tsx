import { useMemo, useRef, useState } from "react";
import { useLocation } from "react-router-dom";
import { createPortal } from "react-dom";
import { cn } from "@/shared/lib/cn";
import { filterCommands, useCommands } from "@/shared/lib/commands";
import type { Command, CommandGroup } from "@/shared/lib/commands";
import { useUiStore } from "@/shared/lib/ui-store";
import { useDismissable } from "@/shared/hooks/useDismissable";
import { useFocusTrap } from "@/shared/hooks/useFocusTrap";
import { Kbd } from "@/shared/ui";
import { destinationForPath } from "@/app/router/routes";

const GROUPES: ReadonlyArray<{ key: CommandGroup; label: string }> = [
  { key: "selection", label: "Actions sur la sélection" },
  { key: "create", label: "Créer" },
  { key: "goto", label: "Aller à" },
  { key: "settings", label: "Réglages" },
];

/**
 * Palette de commandes `⌘K` (`screens/20-command-palette.png`) : 560 px, à 76 px du haut,
 * commandes groupées par objet. La saisie filtre sans tenir compte des accents ; `↑ ↓`
 * parcourent, `⏎` exécute, `Échap` ferme.
 */
export function CommandPalette() {
  const open = useUiStore((state) => state.palette);
  if (!open) return null;
  return <PaletteSurface />;
}

function PaletteSurface() {
  const setPalette = useUiStore((state) => state.setPalette);
  const commands = useCommands();
  const { pathname } = useLocation();
  const scope = destinationForPath(pathname).label;
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const surface = useRef<HTMLDivElement>(null);
  const close = () => setPalette(false);

  const ordered = useMemo(() => {
    const found = filterCommands(commands, query);
    return GROUPES.flatMap((group) => found.filter((command) => command.group === group.key));
  }, [commands, query]);
  const active = ordered[Math.min(activeIndex, ordered.length - 1)];

  const run = (command: Command | undefined) => {
    if (!command) return;
    close();
    command.run();
  };

  useDismissable({ open: true, onDismiss: close });
  useFocusTrap(surface, true);

  return createPortal(
    <div className="fixed inset-0 z-[75] flex justify-center bg-scrim/50 px-8 pt-[76px]" onPointerDown={(event) => {
      if (event.target === event.currentTarget) close();
    }}>
      <div
        ref={surface}
        role="dialog"
        aria-modal="true"
        aria-label="Palette de commandes"
        className="flex max-h-[min(520px,calc(100vh-110px))] w-[560px] max-w-full animate-pop-menu flex-col overflow-hidden rounded-r11 border border-bd-menu bg-menu shadow-pop"
      >
        <div className="flex h-11 flex-none items-center gap-2.5 border-b border-bd-soft px-3.5">
          <span aria-hidden className="text-row text-tx-5">
            ⌕
          </span>
          <input
            autoFocus
            role="combobox"
            aria-expanded
            aria-controls="palette-resultats"
            aria-activedescendant={active ? `palette-${active.id}` : undefined}
            aria-label="Chercher, créer, aller à…"
            placeholder="Chercher, créer, aller à…"
            value={query}
            onChange={(event) => {
              setQuery(event.target.value);
              setActiveIndex(0);
            }}
            onKeyDown={(event) => {
              if (event.key === "ArrowDown") {
                event.preventDefault();
                setActiveIndex((index) => Math.min(index + 1, ordered.length - 1));
              } else if (event.key === "ArrowUp") {
                event.preventDefault();
                setActiveIndex((index) => Math.max(index - 1, 0));
              } else if (event.key === "Enter") {
                event.preventDefault();
                run(active);
              }
            }}
            className="min-w-0 flex-1 bg-transparent text-lead text-tx outline-none placeholder:text-tx-6"
          />
          <span className="flex-none rounded-r5 bg-chip px-2 py-0.5 text-tiny text-tx-4">{scope}</span>
        </div>

        <div id="palette-resultats" role="listbox" aria-label="Commandes" className="min-h-0 flex-1 overflow-y-auto p-[5px]">
          {ordered.length === 0 ? (
            <p className="px-3 py-6 text-center text-ui text-tx-5">
              Aucune commande ne correspond à « {query} ».
            </p>
          ) : (
            GROUPES.map((group) => {
              const items = ordered.filter((command) => command.group === group.key);
              if (items.length === 0) return null;
              return (
                <div key={group.key} role="group" aria-label={group.label}>
                  <p className="px-2.5 pt-2 pb-1 text-tiny font-medium text-tx-5">{group.label}</p>
                  {items.map((command) => {
                    const isActive = command === active;
                    return (
                      <div
                        key={command.id}
                        id={`palette-${command.id}`}
                        role="option"
                        aria-selected={isActive}
                        onPointerMove={() => setActiveIndex(ordered.indexOf(command))}
                        onClick={() => run(command)}
                        className={cn(
                          "flex h-palette-row cursor-default items-center gap-2.5 rounded-r9 px-2.5",
                          isActive && "bg-elev",
                        )}
                      >
                        <span
                          aria-hidden
                          className={cn(
                            "flex size-5 flex-none items-center justify-center rounded-r5 text-mono-sm",
                            isActive ? "bg-ac text-white" : "bg-chip text-tx-5",
                          )}
                        >
                          {command.glyph ?? "›"}
                        </span>
                        <span className="min-w-0 truncate text-ui text-tx-2">{command.label}</span>
                        {command.detail ? (
                          <span className="min-w-0 truncate text-small text-tx-5">{command.detail}</span>
                        ) : null}
                        {command.shortcut ? (
                          <span className="ml-auto flex flex-none gap-1">
                            {command.shortcut.split(" ").map((key) => (
                              <Kbd key={key} shortcut={key} tone="ghost" decorative />
                            ))}
                          </span>
                        ) : null}
                      </div>
                    );
                  })}
                </div>
              );
            })
          )}
        </div>

        <footer className="flex h-8 flex-none items-center gap-3.5 border-t border-bd-soft px-3.5 text-tiny text-tx-4">
          <span className="flex items-center gap-1.5">Naviguer <Kbd shortcut="up" /><Kbd shortcut="down" /></span>
          <span className="flex items-center gap-1.5">Exécuter <Kbd shortcut="enter" /></span>
          <span className="ml-auto flex items-center gap-1.5">Fermer <Kbd shortcut="escape" /></span>
        </footer>
      </div>
    </div>,
    document.body,
  );
}
