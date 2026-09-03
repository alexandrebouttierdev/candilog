import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { ProfileImportProgress } from "@/features/ai/model/types";

export type ImportJournalEntry = { at: string; message: string };

/** Journal et étape d'analyse via événement Tauri, sans pourcentage. */
export function useProfileImportProgress(generation_id: string | null) {
  const [state, setState] = useState<{
    id: string | null;
    step: string | null;
    entries: ImportJournalEntry[];
    tokens_used: number | null;
  }>({ id: null, step: null, entries: [], tokens_used: null });

  useEffect(() => {
    if (!generation_id) return;
    let cancelled = false;
    let dispose: (() => void) | undefined;
    void listen<ProfileImportProgress>("profile_import_progress", (event) => {
      if (event.payload.generation_id !== generation_id) return;
      setState((current) => {
        const entries =
          current.id === generation_id
            ? current.entries
            : [];
        return {
          id: generation_id,
          step: event.payload.step ?? (current.id === generation_id ? current.step : null),
          entries: event.payload.message
            ? [...entries, { at: event.payload.at, message: event.payload.message }]
            : entries,
          tokens_used:
            event.payload.tokens_used ??
            (current.id === generation_id ? current.tokens_used : null),
        };
      });
    })
      .then((unlisten) => {
        if (cancelled) {
          unlisten();
          return;
        }
        dispose = unlisten;
      })
      .catch(() => {
        /* navigateur de revue sans runtime Tauri */
      });
    return () => {
      cancelled = true;
      dispose?.();
    };
  }, [generation_id]);

  const active = state.id === generation_id;
  return {
    step: active ? state.step : generation_id ? null : state.step,
    entries: generation_id && !active ? [] : state.entries,
    tokens_used: active ? state.tokens_used : generation_id ? null : state.tokens_used,
  };
}
