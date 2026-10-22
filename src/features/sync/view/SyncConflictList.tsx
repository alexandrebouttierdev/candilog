import { Button, StatusGlyph } from "@/shared/ui";
import type { ConflictChoice, SyncConflict } from "@/shared/types/generated/sync";
import { formatMoment } from "../model/syncStatus";

const KIND_LABELS = {
  edit: "Modifié sur les deux ordinateurs",
  deleted_remotely: "Supprimé sur un autre ordinateur, modifié ici",
  deleted_locally: "Supprimé ici, modifié sur un autre ordinateur",
} as const;

/** Libellés des deux choix, selon la nature du conflit. */
function choiceLabels(conflict: SyncConflict): Record<ConflictChoice, string> {
  switch (conflict.kind) {
    case "deleted_remotely":
      return { keep_local: "Conserver la fiche", take_remote: "Supprimer partout" };
    case "deleted_locally":
      return { keep_local: "Confirmer la suppression", take_remote: "Restaurer la fiche" };
    default:
      return { keep_local: "Garder cet ordinateur", take_remote: "Prendre l'autre version" };
  }
}

/**
 * Conflits en attente. Chaque décision est publiée : les autres ordinateurs l'adoptent au
 * lieu de devoir trancher à leur tour.
 */
export function SyncConflictList({
  conflicts,
  busy,
  onResolve,
  onResolveAll,
}: {
  conflicts: readonly SyncConflict[];
  busy: boolean;
  onResolve: (id: string, choice: ConflictChoice) => void;
  onResolveAll: (choice: ConflictChoice) => void;
}) {
  if (conflicts.length === 0) return null;
  return (
    <section aria-labelledby="sync-conflits" className="mt-5">
      <div className="flex items-center gap-2">
        <h3 id="sync-conflits" className="caps">
          Conflits à arbitrer · {conflicts.length}
        </h3>
        {conflicts.length > 1 ? (
          <span className="ml-auto flex gap-1.5">
            <Button size="compact" disabled={busy} onClick={() => onResolveAll("keep_local")}>
              Tout garder d'ici
            </Button>
            <Button size="compact" disabled={busy} onClick={() => onResolveAll("take_remote")}>
              Tout prendre de l'autre
            </Button>
          </span>
        ) : null}
      </div>
      <ul className="mt-2 flex flex-col gap-2">
        {conflicts.map((conflict) => {
          const labels = choiceLabels(conflict);
          return (
            <li key={conflict.id} className="rounded-r9 border border-bd-soft bg-group px-3 py-2.5">
              <div className="flex items-start gap-2">
                <span className="mt-[3px]">
                  <StatusGlyph tone="a" />
                </span>
                <div className="min-w-0 flex-1">
                  <p className="text-row text-tx">
                    {conflict.entity_label} · <span className="font-medium">{conflict.record_label}</span>
                  </p>
                  <p className="text-sub text-tx-5">
                    {KIND_LABELS[conflict.kind]} · détecté le {formatMoment(conflict.detected_at)}
                  </p>
                </div>
              </div>
              {conflict.fields.length > 0 ? (
                <table className="mt-2 w-full table-fixed text-sub">
                  <thead>
                    <tr className="text-left text-tx-6">
                      <th className="w-[28%] pb-1 font-normal">Champ</th>
                      <th className="pb-1 font-normal">Sur cet ordinateur</th>
                      <th className="pb-1 font-normal">Sur l'autre ordinateur</th>
                    </tr>
                  </thead>
                  <tbody>
                    {conflict.fields.map((field) => (
                      <tr key={field.column} className="border-t border-bd-soft align-top">
                        <td className="py-1 pr-2 text-tx-4">{field.label}</td>
                        <td className="py-1 pr-2 break-words text-tx-2">{field.local ?? "—"}</td>
                        <td className="py-1 break-words text-tx-2">{field.remote ?? "—"}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              ) : null}
              <div className="mt-2 flex justify-end gap-1.5">
                <Button size="compact" disabled={busy} onClick={() => onResolve(conflict.id, "keep_local")}>
                  {labels.keep_local}
                </Button>
                <Button size="compact" disabled={busy} onClick={() => onResolve(conflict.id, "take_remote")}>
                  {labels.take_remote}
                </Button>
              </div>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
