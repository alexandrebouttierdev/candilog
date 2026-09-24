import { useCallback, useState } from "react";
import type { MouseEvent } from "react";
import { NavLink, useLocation } from "react-router-dom";
import { cn } from "@/shared/lib/cn";
import { PATHS } from "@/shared/lib/paths";
import { ConfirmDialog, LineIcon, Menu } from "@/shared/ui";
import type { MenuAnchor } from "@/shared/ui";
import { SaveViewDialog, useSavedViews } from "@/features/views";
import type { SavedView } from "@/features/views";

/**
 * Section « Vues » de la navigation (`INTERACTIONS.md` §2) : chaque vue ouvre Candidatures
 * avec son filtre, décompte à droite. Clic droit ou `⋯` : renommer, dupliquer, supprimer.
 * Visible dès 1060 px, comme les libellés de la navigation.
 */
export function SavedViewsNav() {
  const { pathname, search } = useLocation();
  const views = useSavedViews();
  const [menu, setMenu] = useState<{ view: SavedView; anchor: MenuAnchor } | null>(null);
  const [renaming, setRenaming] = useState<SavedView | null>(null);
  const [deleting, setDeleting] = useState<SavedView | null>(null);
  const closeMenu = useCallback(() => setMenu(null), []);
  const activeId = pathname.startsWith(PATHS.applications) ? new URLSearchParams(search).get("view") : null;

  const openMenu = (view: SavedView, event: MouseEvent<HTMLElement>, atPointer: boolean) => {
    event.preventDefault();
    event.stopPropagation();
    setMenu({ view, anchor: atPointer ? { x: event.clientX, y: event.clientY } : event.currentTarget.getBoundingClientRect() });
  };

  return (
    <section aria-label="Vues enregistrées" className="mt-4 hidden wide:block">
      <h2 className="caps flex items-center px-2 pb-1">Vues</h2>
      {!views.isLoaded ? null : views.views.length === 0 ? (
        <p className="px-2 text-tiny leading-[1.45] text-tx-6">
          Filtrez Candidatures, puis « Enregistrer la vue » pour la retrouver ici.
        </p>
      ) : (
        <ul className="flex flex-col gap-px">
          {views.views.map((view) => {
            const count = views.countOf(view.id);
            const active = view.id === activeId;
            return (
              <li key={view.id} className="group relative">
                <NavLink
                  to={`${PATHS.applications}?view=${encodeURIComponent(view.id)}`}
                  aria-current={active ? "page" : undefined}
                  onContextMenu={(event) => openMenu(view, event, true)}
                  className={cn(
                    "flex h-nav-item items-center gap-[9px] rounded-r6 px-2 text-ui",
                    active ? "bg-sel font-medium text-tx" : "text-tx-3 hover:bg-elev hover:text-tx",
                  )}
                >
                  <LineIcon name="bookmark" className={active ? "text-ac-tx" : undefined} />
                  <span className="min-w-0 flex-1 truncate">{view.name}</span>
                  {count !== undefined ? (
                    <span className="font-mono text-mono-sm text-tx-6 group-hover:hidden">{count}</span>
                  ) : null}
                </NavLink>
                <button
                  type="button"
                  aria-label={`Actions sur la vue ${view.name}`}
                  onClick={(event) => openMenu(view, event, false)}
                  className="absolute top-1/2 right-1 hidden h-[18px] -translate-y-1/2 items-center rounded-r5 px-1 text-small text-tx-5 group-hover:flex hover:bg-hover hover:text-tx"
                >
                  ⋯
                </button>
              </li>
            );
          })}
        </ul>
      )}

      <Menu
        open={menu !== null}
        anchor={menu?.anchor ?? null}
        label={`Vue ${menu?.view.name ?? ""}`}
        onClose={closeMenu}
        entries={
          menu
            ? [
                { kind: "section", id: "titre", label: menu.view.name },
                { kind: "item", id: "renommer", label: "Renommer…", onSelect: () => setRenaming(menu.view) },
                {
                  kind: "item",
                  id: "dupliquer",
                  label: "Dupliquer",
                  onSelect: () => void views.duplicate(menu.view.id).catch(() => undefined),
                },
                { kind: "separator", id: "sep" },
                { kind: "item", id: "supprimer", label: "Supprimer la vue…", tone: "danger", onSelect: () => setDeleting(menu.view) },
              ]
            : []
        }
      />
      <SaveViewDialog
        open={renaming !== null}
        title="Renommer la vue"
        initialName={renaming?.name ?? ""}
        submitLabel="Renommer"
        busy={views.isSaving}
        onClose={() => setRenaming(null)}
        onSubmit={(name) =>
          renaming ? views.update({ id: renaming.id, input: { name, filter: renaming.filter } }) : Promise.resolve()
        }
      />
      <ConfirmDialog
        open={deleting !== null}
        title="Supprimer cette vue ?"
        description={`La vue « ${deleting?.name ?? ""} » disparaît de la navigation.`}
        note="Les candidatures qu'elle montrait ne sont pas touchées."
        confirmLabel="Supprimer la vue"
        busy={views.isDeleting}
        onCancel={() => setDeleting(null)}
        onConfirm={() => {
          const target = deleting;
          setDeleting(null);
          if (target) views.remove(target).catch(() => undefined);
        }}
      />
    </section>
  );
}
