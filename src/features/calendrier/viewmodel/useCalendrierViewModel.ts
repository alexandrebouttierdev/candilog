import { useCallback, useMemo, useState } from "react";
import { useMutation, useQueries, useQueryClient } from "@tanstack/react-query";
import { entretienService } from "@/features/entretiens/services/entretien.service";
import type { Entretien, NouvelEntretien } from "@/features/entretiens/services/entretien.service";
import { relanceService } from "@/features/relances/services/relance.service";
import type { NouvelleRelance, Relance } from "@/features/relances/services/relance.service";
import { CANDIDATURES_KEY } from "@/features/candidatures";
import {
  bornesDeLaGrille,
  decalerMois,
  grilleDuMois,
  libelleMois,
} from "../model/mois";
import {
  depuisEntretien,
  depuisRelance,
  grouperParJour,
  type EvenementCalendrier,
} from "../model/evenement";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

/** Racines de cache des deux entités affichées. */
export const ENTRETIENS_KEY = ["entretiens"] as const;
export const RELANCES_KEY = ["relances"] as const;

/**
 * Orchestration de l'écran Suivi → Calendrier.
 *
 * Les deux entités sont chargées par `useQueries` sur **les bornes de la grille** et non du
 * mois : la grille déborde sur les mois voisins, et interroger le seul mois laisserait ces
 * cases vides alors qu'elles portent des événements.
 */
export function useCalendrierViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);

  // Année et mois dans un seul état : ils changent toujours ensemble, et les séparer
  // obligerait à lire l'un pour calculer l'autre, ce qui fige leurs valeurs dans la
  // fermeture du gestionnaire.
  const [periode, setPeriode] = useState(() => {
    const maintenant = new Date();
    return { annee: maintenant.getFullYear(), mois: maintenant.getMonth() };
  });
  const { annee, mois } = periode;

  const bornes = useMemo(() => bornesDeLaGrille(annee, mois), [annee, mois]);
  const cases = useMemo(() => grilleDuMois(annee, mois), [annee, mois]);

  const [entretiens, relances] = useQueries({
    queries: [
      {
        queryKey: [...ENTRETIENS_KEY, "entre", bornes],
        // Les entretiens portent une heure : les bornes doivent couvrir la journée entière,
        // sans quoi un entretien de 14 h le dernier jour affiché serait exclu.
        queryFn: () =>
          entretienService.listerEntre(`${bornes.from}T00:00:00`, `${bornes.to}T23:59:59`),
      },
      {
        queryKey: [...RELANCES_KEY, "entre", bornes],
        queryFn: () => relanceService.listerEntre(bornes.from, bornes.to),
      },
    ],
  });

  const evenements = useMemo<EvenementCalendrier[]>(
    () => [
      ...(entretiens.data ?? []).map(depuisEntretien),
      ...(relances.data ?? []).map(depuisRelance),
    ],
    [entretiens.data, relances.data],
  );

  const parJour = useMemo(() => grouperParJour(evenements), [evenements]);

  // Les compteurs d'en-tête ne comptent que le mois affiché, pas la grille entière : ils
  // sont posés à côté de « août 2026 », et y inclure les débordements sur juillet et
  // septembre ferait mentir le chiffre sur le mois qu'il annonce.
  const prefixeDuMois = `${annee}-${String(mois + 1).padStart(2, "0")}`;
  const dansLeMois = useMemo(
    () => evenements.filter((evenement) => evenement.jour.startsWith(prefixeDuMois)),
    [evenements, prefixeDuMois],
  );

  const invalider = useCallback(async () => {
    await queryClient.invalidateQueries({ queryKey: ENTRETIENS_KEY });
    await queryClient.invalidateQueries({ queryKey: RELANCES_KEY });
    // Enregistrer un entretien fait avancer la candidature : sans cette invalidation, le
    // Kanban resterait sur le statut précédent jusqu'au prochain rechargement.
    await queryClient.invalidateQueries({ queryKey: CANDIDATURES_KEY });
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

  const enregistrerEntretien = useMutation({
    mutationFn: (params: { id: string | null; input: NouvelEntretien }) =>
      entretienService.enregistrer(params.id, params.input),
    onSuccess: async (entretien) => {
      await invalider();
      notify({
        tone: "success",
        title: "Entretien enregistré",
        detail: `${entretien.candidaturePoste ?? ""} — la candidature passe en « Entretien ».`,
      });
    },
    onError: signalerEchec("Enregistrement impossible"),
  });

  const supprimerEntretien = useMutation({
    mutationFn: (id: string) => entretienService.supprimer(id),
    onSuccess: async () => {
      await invalider();
      notify({ tone: "success", title: "Entretien supprimé" });
    },
    onError: signalerEchec("Suppression impossible"),
  });

  const enregistrerRelance = useMutation({
    mutationFn: (params: { id: string | null; input: NouvelleRelance }) =>
      params.id === null
        ? relanceService.creer(params.input)
        : relanceService.modifier(params.id, params.input),
    onSuccess: async () => {
      await invalider();
      notify({ tone: "success", title: "Relance enregistrée" });
    },
    onError: signalerEchec("Enregistrement impossible"),
  });

  const supprimerRelance = useMutation({
    mutationFn: (id: string) => relanceService.supprimer(id),
    onSuccess: async () => {
      await invalider();
      notify({ tone: "success", title: "Relance supprimée" });
    },
    onError: signalerEchec("Suppression impossible"),
  });

  const naviguer = useCallback((pas: number) => {
    setPeriode((courante) => decalerMois(courante.annee, courante.mois, pas));
  }, []);

  const revenirAujourdhui = useCallback(() => {
    const aujourdhui = new Date();
    setPeriode({ annee: aujourdhui.getFullYear(), mois: aujourdhui.getMonth() });
  }, []);

  const allerA = useCallback((cibleAnnee: number, cibleMois: number) => {
    setPeriode({ annee: cibleAnnee, mois: cibleMois });
  }, []);

  /** Entretien ou relance derrière un événement, pour rouvrir la bonne modale. */
  const entretienDe = useCallback(
    (id: string): Entretien | null => entretiens.data?.find((item) => item.id === id) ?? null,
    [entretiens.data],
  );
  const relanceDe = useCallback(
    (id: string): Relance | null => relances.data?.find((item) => item.id === id) ?? null,
    [relances.data],
  );

  return {
    annee,
    mois,
    libelle: libelleMois(annee, mois),
    cases,
    parJour,
    nombreEntretiens: dansLeMois.filter((e) => e.genre === "entretien").length,
    nombreRelances: dansLeMois.filter((e) => e.genre === "relance").length,
    isLoading: entretiens.isPending || relances.isPending,
    error: entretiens.error ?? relances.error,
    isSaving: enregistrerEntretien.isPending || enregistrerRelance.isPending,
    isDeleting: supprimerEntretien.isPending || supprimerRelance.isPending,

    naviguer,
    allerA,
    revenirAujourdhui,
    recharger: () => {
      void entretiens.refetch();
      void relances.refetch();
    },
    entretienDe,
    relanceDe,
    enregistrerEntretien: enregistrerEntretien.mutateAsync,
    supprimerEntretien: supprimerEntretien.mutateAsync,
    enregistrerRelance: enregistrerRelance.mutateAsync,
    supprimerRelance: supprimerRelance.mutateAsync,
  };
}
