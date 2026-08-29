import { useState } from "react";
import { Icon } from "@/shared/ui";
import { formatJournalTime } from "../../model/formatElapsed";
import type { ImportJournalEntry } from "../../viewmodel/useProfileImportProgress";

/** Journal d'import replié par défaut, destiné à un utilisateur non technique. */
export function ImportJournal({
  entries,
  defaultOpen = false,
}: {
  entries: ImportJournalEntry[];
  defaultOpen?: boolean;
}) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <div className="border-t border-line pt-3">
      <button
        type="button"
        aria-expanded={open}
        onClick={() => setOpen((current) => !current)}
        className="inline-flex items-center gap-1 text-label font-medium text-ink-muted hover:text-ink"
      >
        <Icon name={open ? "expand_more" : "chevron_right"} size={16} />
        Journal d'import
      </button>
      {open ? (
        <ol className="mt-2 max-h-40 space-y-1 overflow-y-auto font-mono text-meta text-ink-muted">
          {entries.length === 0 ? (
            <li>Aucun événement pour le moment.</li>
          ) : (
            entries.map((entry, index) => (
              <li key={`${entry.at}-${index}`} className="flex gap-3">
                <span className="tabular text-ink-faint">{formatJournalTime(entry.at)}</span>
                <span>{entry.message}</span>
              </li>
            ))
          )}
        </ol>
      ) : null}
    </div>
  );
}
