import { formatElapsed, formatTokens } from "@/shared/lib/duration";
import type { ImportJournalEntry } from "../../viewmodel/useProfileImportProgress";
import { ImportJournal } from "./ImportJournal";

/** Progression indéterminée : aucune valeur chiffrée n'est affichée. */
export function ImportAnalysisPanel({
  step,
  elapsedMs,
  entries,
  tokens_used,
}: {
  step: string | null;
  elapsedMs: number;
  entries: ImportJournalEntry[];
  tokens_used?: number | null;
}) {
  return (
    <div className="space-y-4 pt-3">
      <div>
        <p className="text-item font-semibold text-ink">Analyse du CV en cours…</p>
        <div className="mt-3 h-1 overflow-hidden rounded-full bg-fill">
          <div className="import-indeterminate h-full w-1/3 rounded-full bg-accent" />
        </div>
      </div>
      {step ? <p className="text-body text-ink-muted">{step}</p> : null}
      <p className="tabular text-meta text-ink-faint">
        Temps écoulé : {formatElapsed(elapsedMs)}
        {tokens_used !== null && tokens_used !== undefined
          ? ` · ${formatTokens(tokens_used)} tokens`
          : null}
      </p>
      <ImportJournal entries={entries} />
    </div>
  );
}
