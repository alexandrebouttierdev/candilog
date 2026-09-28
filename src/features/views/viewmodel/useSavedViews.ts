import { useMutation, useQueries, useQuery, useQueryClient } from "@tanstack/react-query";
import { APPLICATIONS_KEY, applicationService } from "@/features/applications";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { viewsService } from "../services/viewsService";
import type { NewSavedView, SavedView } from "../services/viewsService";

/** Racine des clés de cache des vues. */
export const VIEWS_KEY = ["vues"] as const;

function detail(error: unknown): string | undefined {
  return error instanceof AppError ? error.message : undefined;
}

/**
 * Vues enregistrées : liste, décompte de chacune (le filtre rejoué par SQLite, comme la
 * liste le ferait) et écritures. Le décompte suit les candidatures : il est rangé sous leur
 * clé racine, et toute écriture sur une candidature le remet à jour.
 */
export function useSavedViews() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const list = useQuery({ queryKey: VIEWS_KEY, queryFn: viewsService.list });
  const views = list.data ?? [];

  const counts = useQueries({
    queries: views.map((view) => ({
      queryKey: [...APPLICATIONS_KEY, "vue", view.id, view.filter],
      queryFn: async () => {
        const breakdown = await applicationService.breakdown(view.filter);
        // La répartition ignore le filtre de statut : on ne garde que les statuts retenus.
        const byStatus = {
          EN_ATTENTE: breakdown.pending,
          RELANCEE: breakdown.followed_up,
          ENTRETIEN: breakdown.interview,
          REFUS: breakdown.rejected,
        } as const;
        const statuses = view.filter.status.length > 0 ? view.filter.status : (Object.keys(byStatus) as Array<keyof typeof byStatus>);
        const excluded = view.filter.excluded.includes("status");
        return (Object.keys(byStatus) as Array<keyof typeof byStatus>)
          .filter((status) => (excluded ? !statuses.includes(status) : statuses.includes(status)))
          .reduce((sum, status) => sum + byStatus[status], 0);
      },
    })),
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey: VIEWS_KEY });
  const fail = (title: string) => (error: unknown) => notify({ tone: "error", title, detail: detail(error) });

  const create = useMutation({
    mutationFn: (input: NewSavedView) => viewsService.create(input),
    onSuccess: async (view) => {
      await invalidate();
      notify({ tone: "success", title: `Vue « ${view.name} » enregistrée` });
    },
    onError: fail("Enregistrement impossible"),
  });
  const update = useMutation({
    mutationFn: (params: { id: string; input: NewSavedView }) => viewsService.update(params.id, params.input),
    onSuccess: async (view) => {
      await invalidate();
      notify({ tone: "success", title: `Vue « ${view.name} » mise à jour` });
    },
    onError: fail("Mise à jour impossible"),
  });
  const duplicate = useMutation({
    mutationFn: (id: string) => viewsService.duplicate(id),
    onSuccess: async (view) => {
      await invalidate();
      notify({ tone: "success", title: `Vue « ${view.name} » créée` });
    },
    onError: fail("Duplication impossible"),
  });
  const remove = useMutation({
    mutationFn: (view: SavedView) => viewsService.delete(view.id),
    onSuccess: async (_result, view) => {
      await invalidate();
      notify({ tone: "success", title: `Vue « ${view.name} » supprimée` });
    },
    onError: fail("Suppression impossible"),
  });

  return {
    views,
    /** La liste est connue : avant, « aucune vue » serait une affirmation fausse. */
    isLoaded: list.isSuccess,
    countOf: (id: string): number | undefined => {
      const index = views.findIndex((view) => view.id === id);
      return index >= 0 ? counts[index]?.data : undefined;
    },
    create: create.mutateAsync,
    update: update.mutateAsync,
    duplicate: duplicate.mutateAsync,
    remove: remove.mutateAsync,
    isSaving: create.isPending || update.isPending,
    isDeleting: remove.isPending,
  };
}
