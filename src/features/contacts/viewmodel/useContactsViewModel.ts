import { useCallback, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { contactService } from "../services/contact.service";
import type { Contact, NouveauContact } from "../services/contact.service";
import { PAGE_SIZE } from "@/shared/types/page";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { ENTREPRISES_KEY } from "@/features/entreprises/viewmodel/useEntreprisesViewModel";

/** Racine des clés de cache de la feature. */
export const CONTACTS_KEY = ["contacts"] as const;

/**
 * Orchestration de l'écran Réseau : liste paginée, recherche, sélection et écritures.
 *
 * Même structure que le ViewModel des entreprises : la pagination et la recherche sont des
 * paramètres de requête, jamais un filtrage local.
 */
export function useContactsViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);

  const [page, setPage] = useState(1);
  const [search, setSearch] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const liste = useQuery({
    queryKey: [...CONTACTS_KEY, "page", { page, search }],
    queryFn: () => contactService.listerPage({ page, pageSize: PAGE_SIZE, search }),
  });

  const invalider = useCallback(async () => {
    await queryClient.invalidateQueries({ queryKey: CONTACTS_KEY });
    // Un contact rattaché change ce que la fiche entreprise affiche de son réseau : sans
    // cette seconde invalidation, la fiche resterait sur son décompte précédent.
    await queryClient.invalidateQueries({ queryKey: ENTREPRISES_KEY });
  }, [queryClient]);

  const signalerEchec = useCallback(
    (titre: string) => (error: unknown) => {
      notify({
        tone: "error",
        title: titre,
        detail: error instanceof AppError ? error.message : undefined,
      });
    },
    [notify],
  );

  const creation = useMutation({
    mutationFn: (input: NouveauContact) => contactService.creer(input),
    onSuccess: async (contact) => {
      await invalider();
      setSelectedId(contact.id);
      notify({
        tone: "success",
        title: "Contact enregistré",
        detail: `${contact.prenom} ${contact.nom}`,
      });
    },
    onError: signalerEchec("Enregistrement impossible"),
  });

  const modification = useMutation({
    mutationFn: (params: { id: string; input: NouveauContact }) =>
      contactService.modifier(params.id, params.input),
    onSuccess: async (contact) => {
      await invalider();
      notify({
        tone: "success",
        title: "Contact modifié",
        detail: `${contact.prenom} ${contact.nom}`,
      });
    },
    onError: signalerEchec("Modification impossible"),
  });

  const suppression = useMutation({
    mutationFn: (id: string) => contactService.supprimer(id),
    onSuccess: async (_result, id) => {
      await invalider();
      if (selectedId === id) setSelectedId(null);
      notify({ tone: "success", title: "Contact supprimé" });
    },
    onError: signalerEchec("Suppression impossible"),
  });

  const rechercher = useCallback((valeur: string) => {
    setSearch(valeur);
    setPage(1);
  }, []);

  const items: Contact[] = liste.data?.items ?? [];
  // Comme pour les entreprises, la fiche de droite n'est jamais vide tant que la page
  // contient au moins un contact.
  const selection = items.find((item) => item.id === selectedId) ?? items[0] ?? null;

  return {
    items,
    total: liste.data?.total ?? 0,
    page,
    pageSize: PAGE_SIZE,
    search,
    selection,
    selectedId: selection?.id ?? null,
    isLoading: liste.isPending,
    error: liste.error,
    isSaving: creation.isPending || modification.isPending,
    isDeleting: suppression.isPending,

    setPage,
    rechercher,
    selectionner: setSelectedId,
    recharger: () => void liste.refetch(),
    creer: creation.mutateAsync,
    modifier: modification.mutateAsync,
    supprimer: suppression.mutateAsync,
  };
}
