import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { relationsService } from "../services/relationsService";

/** Date du jour `AAAA-MM-JJ`, telle qu'elle figurera dans les noms de fichiers. */
function today(): string {
  const now = new Date();
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

/**
 * Export des relations (`states/dialog-csv-rel.png`) : un dialogue d'information annonce
 * les deux fichiers et leurs lignes, puis le dialogue natif choisit l'emplacement.
 */
export function useRelationsExport() {
  const notify = useUiStore((state) => state.notify);
  const [open, setOpen] = useState(false);
  const [date] = useState(today);
  const run = useMutation({
    mutationFn: relationsService.exportCsv,
    onSuccess: (result) => {
      setOpen(false);
      if (!result) return;
      notify({
        tone: "success",
        title: "Relations exportées",
        detail: `${result.companies} entreprise${result.companies > 1 ? "s" : ""} dans ${result.companies_file}, ${result.contacts} contact${result.contacts > 1 ? "s" : ""} dans ${result.contacts_file}.`,
      });
    },
    onError: (error: unknown) => {
      notify({
        tone: "error",
        title: "Export impossible",
        detail: error instanceof AppError ? error.message : undefined,
      });
    },
  });

  return {
    open,
    date,
    isExporting: run.isPending,
    ask: () => setOpen(true),
    cancel: () => setOpen(false),
    confirm: () => run.mutate(),
  };
}
