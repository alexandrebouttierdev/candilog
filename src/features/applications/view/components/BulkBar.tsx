import { Kbd } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";

/**
 * Barre des actions groupées (`INTERACTIONS.md` §3.2), 40 px au-dessus de la barre d'état :
 * fond de sélection, filet `ac`. Elle agit sur les candidatures **cochées**, jamais sur la
 * seule candidature ouverte dans l'inspecteur.
 */
export function BulkBar({
  count,
  busy,
  onStatus,
  onExport,
  onDelete,
  onClear,
}: {
  count: number;
  busy: boolean;
  onStatus: (anchor: DOMRect) => void;
  onExport: () => void;
  onDelete: () => void;
  onClear: () => void;
}) {
  const action = "inline-flex h-[25px] flex-none items-center gap-[7px] rounded-r7 px-2.5 text-small whitespace-nowrap disabled:opacity-55";
  return (
    <div
      role="toolbar"
      aria-label="Actions sur les candidatures cochées"
      className="flex h-10 flex-none items-center gap-[9px] overflow-x-auto border-t border-ac bg-sel px-3.5"
    >
      <span
        aria-hidden
        className="inline-flex size-[17px] flex-none items-center justify-center rounded-r4 bg-ac font-mono text-[10px] text-white"
      >
        {count}
      </span>
      <span className="text-ui font-medium whitespace-nowrap text-tx">
        {count} candidature{count > 1 ? "s" : ""} sélectionnée{count > 1 ? "s" : ""}
      </span>
      <span aria-hidden className="mx-[3px] h-[18px] w-px flex-none bg-bd-menu" />
      <button
        type="button"
        disabled={busy}
        aria-keyshortcuts="S"
        onClick={(event) => onStatus(event.currentTarget.getBoundingClientRect())}
        className={cn(action, "bg-ac text-white")}
      >
        Changer le statut
        <Kbd shortcut="s" tone="on-accent" decorative />
      </button>
      <button
        type="button"
        disabled={busy}
        aria-keyshortcuts="Meta+E"
        onClick={onExport}
        className={cn(action, "bg-chip text-tx-2 hover:text-tx")}
      >
        Exporter en CSV
        <Kbd shortcut="mod+e" tone="ghost" decorative />
      </button>
      <button
        type="button"
        disabled={busy}
        aria-keyshortcuts="Meta+Backspace"
        onClick={onDelete}
        className={cn(action, "text-st-c")}
      >
        Supprimer
        <Kbd shortcut="mod+backspace" tone="ghost" decorative />
      </button>
      <button
        type="button"
        aria-keyshortcuts="Escape"
        onClick={onClear}
        className={cn(action, "ml-auto text-tx-5 hover:text-tx-3")}
      >
        Désélectionner
        <Kbd shortcut="escape" tone="ghost" decorative />
      </button>
    </div>
  );
}
