import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { APPLICATIONS_KEY } from "@/features/applications";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import type { RelationRef } from "@/shared/types/generated/relations";
import { historyLines } from "../model/history";
import { relationsService } from "../services/relationsService";

/**
 * Historique de la fiche ouverte dans l'inspecteur de Relations, et ses notes.
 *
 * Rangé sous la clé racine des candidatures : l'historique se lit dans les candidatures,
 * leurs statuts, entretiens et relances, et toute écriture sur l'une d'elles le remet à jour.
 */
export function useRelationHistory(relation: RelationRef | null) {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const key = [...APPLICATIONS_KEY, "historique-relation", relation?.kind, relation?.id] as const;
  const query = useQuery({
    queryKey: key,
    queryFn: () => {
      // `enabled` garantit la fiche : la requête ne part pas sans elle.
      if (relation === null) throw new Error("Aucune fiche ouverte");
      return relationsService.history(relation);
    },
    enabled: relation !== null,
  });
  const refresh = () => queryClient.invalidateQueries({ queryKey: key });

  const add = useMutation({
    mutationFn: (note: { body: string; noted_on: string }) => {
      if (relation === null) throw new Error("Aucune fiche ouverte");
      return relationsService.addNote({ relation, ...note });
    },
    onSuccess: refresh,
  });
  const remove = useMutation({
    mutationFn: (id: string) => relationsService.deleteNote(id),
    onSuccess: async () => {
      await refresh();
      notify({ tone: "success", title: "Note supprimée" });
    },
    onError: (error: unknown) =>
      notify({
        tone: "error",
        title: "Suppression impossible",
        detail: error instanceof AppError ? error.message : undefined,
      }),
  });

  return {
    lines: relation && query.data ? historyLines(query.data, relation.kind) : [],
    loading: query.isPending && relation !== null,
    error: query.error ? (query.error instanceof AppError ? query.error.message : "L'historique n'a pas pu être lu.") : null,
    /** Résout à `true` si la note est enregistrée ; l'erreur reste affichée dans le dialogue. */
    addNote: async (body: string, notedOn: string) => {
      try {
        await add.mutateAsync({ body, noted_on: notedOn });
        return true;
      } catch {
        return false;
      }
    },
    addError: add.error ? (add.error instanceof AppError ? add.error.message : "La note n'a pas pu être enregistrée.") : null,
    resetAdd: add.reset,
    saving: add.isPending,
    deleteNote: (id: string) => remove.mutate(id),
    deleting: remove.isPending,
  };
}
