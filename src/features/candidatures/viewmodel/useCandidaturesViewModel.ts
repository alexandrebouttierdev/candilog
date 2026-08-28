import { useCallback, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { candidatureService } from "../services/candidature.service";
import type {
  Candidature,
  FiltreCandidatures,
  NouvelleCandidature,
  StatutCandidature,
} from "../services/candidature.service";
import type { CandidatureFilterValues } from "../model/schemas/candidature-filter.schema";
import type { TriCandidature } from "@/shared/types/generated/candidatures";
import { PAGE_SIZE } from "@/shared/types/page";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

/** Racine des clés de cache de la feature. */
export const CANDIDATURES_KEY = ["candidatures"] as const;

/** Mode d'affichage du suivi. */
export type VueSuivi = "kanban" | "liste";

/** Filtres appliqués, dans leur forme validée. */
const FILTRE_INITIAL: CandidatureFilterValues = {
  statut: null,
  contrat: null,
  entrepriseId: null,
  ville: "",
  poste: "",
  dateDebut: null,
  dateFin: null,
};

/**
 * Orchestration de l'écran Suivi → Candidatures.
 *
 * Sert les deux vues, Kanban et Liste, sur la même requête : elles n'affichent pas les
 * mêmes formes mais interrogent le même filtre, et les séparer aurait dupliqué l'état des
 * filtres, du tri et de la pagination.
 */
export function useCandidaturesViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);

  const [vue, setVue] = useState<VueSuivi>("kanban");
  const [page, setPage] = useState(1);
  const [taillePage, setTaillePage] = useState<number>(PAGE_SIZE);
  const [search, setSearch] = useState("");
  const [filtres, setFiltres] = useState<CandidatureFilterValues>(FILTRE_INITIAL);
  const [tri, setTri] = useState<TriCandidature>("date");
  const [descendant, setDescendant] = useState(true);
  const [selectedId, setSelectedId] = useState<string | null>(null);

  /** Filtre tel qu'envoyé au backend, recherche et tri compris. */
  const filtre = useMemo<FiltreCandidatures>(
    () => ({
      search,
      statut: filtres.statut,
      contrat: filtres.contrat,
      entrepriseId: filtres.entrepriseId,
      ville: filtres.ville,
      poste: filtres.poste,
      dateDebut: filtres.dateDebut,
      dateFin: filtres.dateFin,
      tri,
      descendant,
    }),
    [search, filtres, tri, descendant],
  );

  // Le Kanban affiche les quatre colonnes d'un coup : il demande une page assez large pour
  // les remplir, là où la Liste s'en tient à la densité choisie dans son pied.
  const pageSize = vue === "kanban" ? PAGE_SIZE * 4 : taillePage;

  const liste = useQuery({
    queryKey: [...CANDIDATURES_KEY, "page", { page, pageSize, filtre }],
    queryFn: () => candidatureService.listerPage({ page, pageSize, filtre }),
  });

  // Compteurs des en-têtes de colonnes : calculés par SQLite sur tout le filtre, pas sur la
  // page affichée — une colonne annoncerait sinon « 3 » en contenant tout le pipeline.
  const repartition = useQuery({
    queryKey: [...CANDIDATURES_KEY, "repartition", { filtre }],
    queryFn: () => candidatureService.repartition(filtre),
  });

  const invalider = useCallback(
    () => queryClient.invalidateQueries({ queryKey: CANDIDATURES_KEY }),
    [queryClient],
  );

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
    mutationFn: (input: NouvelleCandidature) => candidatureService.creer(input),
    onSuccess: async (candidature) => {
      await invalider();
      setSelectedId(candidature.id);
      notify({
        tone: "success",
        title: "Candidature enregistrée",
        detail: `${candidature.poste} — ${candidature.entrepriseNom ?? ""}`,
      });
    },
    onError: signalerEchec("Enregistrement impossible"),
  });

  const modification = useMutation({
    mutationFn: (params: { id: string; input: NouvelleCandidature }) =>
      candidatureService.modifier(params.id, params.input),
    onSuccess: async (candidature) => {
      await invalider();
      notify({ tone: "success", title: "Candidature modifiée", detail: candidature.poste });
    },
    onError: signalerEchec("Modification impossible"),
  });

  const changementStatut = useMutation({
    mutationFn: (params: { id: string; statut: StatutCandidature }) =>
      candidatureService.changerStatut(params.id, params.statut),
    onSuccess: invalider,
    // Pas de toast en cas de succès : le déplacement de la carte est déjà la confirmation
    // visible du geste. Un échec, lui, doit être annoncé — la carte reviendra à sa place.
    onError: signalerEchec("Changement de statut impossible"),
  });

  const suppression = useMutation({
    mutationFn: (id: string) => candidatureService.supprimer(id),
    onSuccess: async (_result, id) => {
      await invalider();
      if (selectedId === id) setSelectedId(null);
      notify({ tone: "success", title: "Candidature supprimée" });
    },
    onError: signalerEchec("Suppression impossible"),
  });

  const rechercher = useCallback((valeur: string) => {
    setSearch(valeur);
    setPage(1);
  }, []);

  const appliquerFiltres = useCallback((valeurs: CandidatureFilterValues) => {
    setFiltres(valeurs);
    setPage(1);
  }, []);

  const reinitialiserFiltres = useCallback(() => {
    setFiltres(FILTRE_INITIAL);
    setPage(1);
  }, []);

  /** Nombre de critères actifs, hors recherche libre, pour la pastille du bouton Filtres. */
  const filtresActifs = useMemo(
    () =>
      [
        filtres.statut,
        filtres.contrat,
        filtres.entrepriseId,
        filtres.ville || null,
        filtres.poste || null,
        filtres.dateDebut,
        filtres.dateFin,
      ].filter(Boolean).length,
    [filtres],
  );

  /** Bascule la direction si l'on retrie la colonne courante, sinon trie la nouvelle. */
  const trierPar = useCallback(
    (colonne: TriCandidature) => {
      if (colonne === tri) {
        setDescendant((valeur) => !valeur);
      } else {
        setTri(colonne);
        setDescendant(true);
      }
      setPage(1);
    },
    [tri],
  );

  const items: Candidature[] = liste.data?.items ?? [];
  const selection = items.find((item) => item.id === selectedId) ?? null;

  return {
    vue,
    items,
    repartition: repartition.data ?? { enAttente: 0, relancee: 0, entretien: 0, refus: 0 },
    total: liste.data?.total ?? 0,
    page,
    pageSize,
    search,
    filtres,
    filtresActifs,
    filtre,
    tri,
    descendant,
    selection,
    selectedId,
    isLoading: liste.isPending,
    error: liste.error,
    isSaving: creation.isPending || modification.isPending,
    isDeleting: suppression.isPending,

    setVue,
    setPage,
    /** Change la densité de la vue Liste et revient à la première page. */
    setPageSize: useCallback((taille: number) => {
      setTaillePage(taille);
      setPage(1);
    }, []),
    rechercher,
    appliquerFiltres,
    reinitialiserFiltres,
    trierPar,
    selectionner: setSelectedId,
    recharger: () => void liste.refetch(),
    creer: creation.mutateAsync,
    modifier: modification.mutateAsync,
    changerStatut: changementStatut.mutateAsync,
    supprimer: suppression.mutateAsync,
  };
}
