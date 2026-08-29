import { useCallback, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { contactService } from "../services/contactService";
import type { Contact, NewContact } from "../services/contactService";
import { PAGE_SIZE } from "@/shared/types/page";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { COMPANIES_KEY } from "@/features/companies/viewmodel/useCompaniesViewModel";

/** Root des clés de cache de la feature. */
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
  const [tracking_role, setTrackingRole] = useState<string | null>(null);
  const [selected_id, setSelectedId] = useState<string | null>(null);

  const list = useQuery({
    queryKey: [...CONTACTS_KEY, "page", { page, search, tracking_role }],
    queryFn: () =>
      contactService.listPage({ page, page_size: PAGE_SIZE, search, tracking_role }),
  });

  const invalider = useCallback(async () => {
    await queryClient.invalidateQueries({ queryKey: CONTACTS_KEY });
    // Un contact rattaché change ce que la fiche entreprise affiche de son réseau : sans
    // cette seconde invalidation, la fiche resterait sur son décompte précédent.
    await queryClient.invalidateQueries({ queryKey: COMPANIES_KEY });
  }, [queryClient]);

  const signalerEchec = useCallback(
    (title: string) => (error: unknown) => {
      notify({
        tone: "error",
        title: title,
        detail: error instanceof AppError ? error.message : undefined,
      });
    },
    [notify],
  );

  const creation = useMutation({
    mutationFn: (input: NewContact) => contactService.create(input),
    onSuccess: async (contact) => {
      await invalider();
      setSelectedId(contact.id);
      notify({
        tone: "success",
        title: "Contact enregistré",
        detail: `${contact.first_name} ${contact.name}`,
      });
    },
    onError: signalerEchec("Enregistrement impossible"),
  });

  const modification = useMutation({
    mutationFn: (params: { id: string; input: NewContact }) =>
      contactService.update(params.id, params.input),
    onSuccess: async (contact) => {
      await invalider();
      notify({
        tone: "success",
        title: "Contact modifié",
        detail: `${contact.first_name} ${contact.name}`,
      });
    },
    onError: signalerEchec("Modification impossible"),
  });

  const suppression = useMutation({
    mutationFn: (id: string) => contactService.delete(id),
    onSuccess: async (_result, id) => {
      await invalider();
      if (selected_id === id) setSelectedId(null);
      notify({ tone: "success", title: "Contact supprimé" });
    },
    onError: signalerEchec("Suppression impossible"),
  });

  const rechercher = useCallback((value: string) => {
    setSearch(value);
    setPage(1);
  }, []);

  const filtrerParRole = useCallback((value: string | null) => {
    setTrackingRole(value);
    setPage(1);
  }, []);

  const resetFilters = useCallback(() => {
    setTrackingRole(null);
    setPage(1);
  }, []);

  /** Count de critères actifs, hors recherche libre, pour la pastille du bouton Filtres. */
  const filtersActifs = tracking_role ? 1 : 0;

  const items: Contact[] = list.data?.items ?? [];
  // Comme pour les entreprises, la fiche de droite n'est jamais vide tant que la page
  // contient au moins un contact.
  const selection = items.find((item) => item.id === selected_id) ?? items[0] ?? null;

  return {
    items,
    total: list.data?.total ?? 0,
    page,
    page_size: PAGE_SIZE,
    search,
    tracking_role,
    filtersActifs,
    selection,
    selected_id: selection?.id ?? null,
    isLoading: list.isPending,
    error: list.error,
    isSaving: creation.isPending || modification.isPending,
    isDeleting: suppression.isPending,

    setPage,
    rechercher,
    filtrerParRole,
    resetFilters,
    selectionner: setSelectedId,
    recharger: () => void list.refetch(),
    create: creation.mutateAsync,
    update: modification.mutateAsync,
    delete: suppression.mutateAsync,
  };
}
