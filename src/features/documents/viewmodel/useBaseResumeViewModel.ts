import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { PROFILE_KEY, profileService } from "@/features/profile";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import type { ProfileSection } from "@/shared/types/generated/ai";
import type { BaseResume, ResumeDocument } from "@/shared/types/generated/documents";
import { profileGaps } from "../model/profileReadiness";
import { RESUME_SECTIONS, sectionOptions, toggleSection } from "../model/profileSections";
import { documentsService } from "../services/documentsService";
import { BASE_RESUME_KEY, RESUME_KEY } from "./documentKeys";

/** Jumeau frontend de `RESUME_WORKSPACE_VERSION` / `RESUME_BASE_KIND` (Rust). */
const BASE_RESUME_SCHEMA_VERSION = 1;
const BASE_RESUME_KIND = "base";

/**
 * « Ce qui figure sur le CV » : `availability` et `interests` n'ont pas de place sur un
 * CV — même le générateur ciblé ne les y met pas — donc elles ne sont jamais proposées ici,
 * contrairement à `RESUME_SECTIONS` dont le générateur ciblé se sert tel quel.
 */
const BASE_RESUME_SECTIONS = RESUME_SECTIONS.filter(
  ([section]) => section !== "availability" && section !== "interests",
);

function errorMessage(error: unknown): string {
  return error instanceof AppError ? error.message : "Une erreur inattendue s’est produite.";
}

function errorDetail(error: unknown): string | undefined {
  return error instanceof AppError ? error.message : undefined;
}

/**
 * Composition et enregistrement d'un CV de base : lu dans le seul profil, sans offre ni
 * IA. La recomposition est un changement de clé de requête, jamais un effet manuel ; la
 * feuille affichée reste une copie locale pour que les retouches survivent au rendu, et
 * n'est écrasée par une nouvelle composition qu'après confirmation (`toggle`/`confirmToggle`).
 */
export function useBaseResumeViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const [excluded, setExcluded] = useState<ProfileSection[]>([]);
  const [document, setDocumentState] = useState<ResumeDocument | null>(null);
  const [dirty, setDirty] = useState(false);
  const [pendingToggle, setPendingToggle] = useState<ProfileSection | null>(null);
  const [name, setName] = useState("CV de base");

  const profile = useQuery({ queryKey: PROFILE_KEY, queryFn: profileService.load });
  const composed = useQuery({
    queryKey: [...BASE_RESUME_KEY, excluded],
    queryFn: () => documentsService.composeBaseResume(excluded),
  });

  // Resynchronise la copie locale sur chaque nouvelle composition reçue, pendant le rendu
  // plutôt que dans un effet (https://react.dev/learn/you-might-not-need-an-effect) : une
  // retouche en cours n'est jamais écrasée par la même réponse déjà appliquée, comparée par
  // référence.
  const [syncedWith, setSyncedWith] = useState(composed.data);
  if (composed.data && composed.data !== syncedWith) {
    setSyncedWith(composed.data);
    setDocumentState(composed.data);
    setDirty(false);
  }

  /** Retouche locale de la feuille : la marque comme modifiée pour la prochaine bascule. */
  function setDocument(next: ResumeDocument): void {
    setDocumentState(next);
    setDirty(true);
  }

  /**
   * Bascule une section. Une feuille non retouchée recompose aussitôt (changement de clé) ;
   * une feuille retouchée attend `confirmToggle`, pour ne jamais écraser une modification en
   * silence.
   */
  function toggle(section: ProfileSection): void {
    if (!dirty) {
      setExcluded((current) => toggleSection(current, section));
      return;
    }
    setPendingToggle(section);
  }

  function confirmToggle(): void {
    if (pendingToggle === null) return;
    const section = pendingToggle;
    setPendingToggle(null);
    setDirty(false);
    setExcluded((current) => toggleSection(current, section));
  }

  function cancelToggle(): void {
    setPendingToggle(null);
  }

  const saveMutation = useMutation({
    mutationFn: (content: ResumeDocument) => {
      const base: BaseResume = {
        schema_version: BASE_RESUME_SCHEMA_VERSION,
        kind: BASE_RESUME_KIND,
        document: content,
      };
      return documentsService.saveResume({ name, content: base, version_note: "Composé depuis le profil" });
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: RESUME_KEY });
      notify({ tone: "success", title: "CV ajouté à la bibliothèque" });
    },
    onError: (caught: unknown) => {
      notify({ tone: "error", title: "Enregistrement impossible", detail: errorDetail(caught) });
    },
  });

  async function save(): Promise<void> {
    if (!document) return;
    await saveMutation.mutateAsync(document);
  }

  return {
    document,
    excluded,
    toggle,
    pendingToggle,
    confirmToggle,
    cancelToggle,
    dirty,
    setDocument,
    isComposing: composed.isFetching,
    error: composed.error ? errorMessage(composed.error) : null,
    /** « Ce qui figure sur le CV », compté sur le profil courant. */
    options: sectionOptions(profile.data?.profile ?? null, BASE_RESUME_SECTIONS),
    // Vide tant que le profil n'est pas chargé : un bandeau qui clignote au démarrage
    // avertirait de ce qu'on ignore encore.
    profileGaps: profile.data ? profileGaps(profile.data.profile) : [],
    name,
    setName,
    save,
    isSaving: saveMutation.isPending,
  };
}
