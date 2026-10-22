import { useUiStore } from "@/shared/lib/ui-store";
import { StatusGlyph } from "@/shared/ui";
import { PHASE_LABELS, phaseTone } from "../model/syncStatus";
import { useSyncStatus } from "../viewmodel/useSyncStatus";

/**
 * Indicateur de la barre d'état : visible seulement quand la synchronisation est en
 * service. Un clic ouvre la section des Réglages qui explique l'état et l'action attendue.
 */
export function SyncStatusIndicator() {
  const status = useSyncStatus().data;
  const openSettings = useUiStore((state) => state.openSettings);
  if (!status || status.provider === "local") return null;
  const label = PHASE_LABELS[status.phase];
  return (
    <button
      type="button"
      onClick={() => openSettings("data")}
      title={status.message ?? label}
      className="flex items-center gap-1.5 whitespace-nowrap text-sub text-tx-3 hover:text-tx"
    >
      <StatusGlyph tone={phaseTone(status.phase)} />
      {label}
      {status.pending_changes > 0 && status.phase !== "syncing" ? (
        <span className="font-mono text-caps text-tx-5">· {status.pending_changes}</span>
      ) : null}
    </button>
  );
}
