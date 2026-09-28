import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { ApplicationsPage } from "@/features/applications";
import type { ActiveSavedView, ApplicationFilter } from "@/features/applications";
import { SaveViewDialog, useSavedViews } from "@/features/views";

/**
 * Candidatures, avec ses vues enregistrées (`?view=<id>`). La couche `app` fait le lien :
 * les deux features ne se connaissent pas. L'écran est remonté à chaque vue (`key`), pour
 * repartir exactement du filtre enregistré.
 */
export function ApplicationsRoute({ view }: { view: "list" | "kanban" }) {
  const [searchParams, setSearchParams] = useSearchParams();
  const views = useSavedViews();
  const [pendingSave, setPendingSave] = useState<ApplicationFilter | null>(null);
  const viewId = searchParams.get("view");
  const active = views.views.find((item) => item.id === viewId) ?? null;
  const savedView: ActiveSavedView | null = active ? { id: active.id, name: active.name, filter: active.filter } : null;

  // Une vue demandée mais pas encore chargée : on attend plutôt que de monter l'écran sans
  // filtre, puis de le remonter une seconde fois.
  if (viewId && !active && views.views.length === 0) return null;

  return (
    <>
      <ApplicationsPage
        key={savedView?.id ?? "toutes"}
        view={view}
        savedView={savedView}
        onSaveView={setPendingSave}
        onUpdateView={(filter) => {
          if (active) void views.update({ id: active.id, input: { name: active.name, filter } }).catch(() => undefined);
        }}
      />
      <SaveViewDialog
        open={pendingSave !== null}
        title="Enregistrer la vue"
        initialName=""
        submitLabel="Enregistrer"
        busy={views.isSaving}
        onClose={() => setPendingSave(null)}
        onSubmit={async (name) => {
          if (!pendingSave) return;
          const created = await views.create({ name, filter: pendingSave });
          setSearchParams((current) => {
            const next = new URLSearchParams(current);
            next.set("view", created.id);
            return next;
          });
        }}
      />
    </>
  );
}
