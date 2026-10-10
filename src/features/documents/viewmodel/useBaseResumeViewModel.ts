import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { PROFILE_KEY, profileService } from "@/features/profile";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import type { ProfileSection } from "@/shared/types/generated/ai";
import type { BaseResume, ResumeDocument } from "@/shared/types/generated/documents";
import { profileGaps } from "../model/profileReadiness";
import { RESUME_BASE_KIND, RESUME_WORKSPACE_VERSION } from "../model/resumeWorkspace";
import { BASE_RESUME_SECTIONS, sectionOptions, toggleSection } from "../model/profileSections";
import { documentsService } from "../services/documentsService";
import { BASE_RESUME_KEY, RESUME_KEY } from "./documentKeys";

function errorMessage(error: unknown): string {
  return error instanceof AppError ? error.message : "Une erreur inattendue s’est produite.";
}

function errorDetail(error: unknown): string | undefined {
  return error instanceof AppError ? error.message : undefined;
}

export interface BaseResumeInitial {
  /** Document rouvert depuis la bibliothèque : affiché tel qu'enregistré, sans composer. */
  document?: ResumeDocument | null;
  name?: string | null;
  /** Document rouvert depuis la bibliothèque : l'enregistrer en ajoute une version. */
  documentId?: string | null;
  /**
   * Sections écartées à la composition du document rouvert : les interrupteurs repartent
   * de là, et la recomposition ne ressuscite pas une section que l'utilisateur avait retirée.
   */
  excludedSections?: readonly ProfileSection[] | null;
}

/**
 * Composition et enregistrement d'un CV de base : lu dans le seul profil, sans offre ni
 * IA. La recomposition est un changement de clé de requête, jamais un effet manuel ; la
 * feuille affichée reste une copie locale pour que les retouches survivent au rendu, et
 * n'est écrasée par une nouvelle composition qu'après confirmation (`toggle`/`confirmToggle`).
 *
 * Rouvrir un CV de base enregistré (`initial.document`) affiche ce document directement :
 * composer depuis le profil à l'ouverture écraserait silencieusement les retouches que
 * l'utilisateur avait enregistrées. La composition ne reprend qu'à la première bascule de
 * section, qui demande alors confirmation exactement comme une retouche de session — rouvrir
 * un document déjà retouché est précisément le cas où l'on écrase ces retouches en
 * connaissance de cause.
 */
export function useBaseResumeViewModel(initial: BaseResumeInitial = {}) {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const [excluded, setExcluded] = useState<ProfileSection[]>([...(initial.excludedSections ?? [])]);
  const [document, setDocumentState] = useState<ResumeDocument | null>(initial.document ?? null);
  const [dirty, setDirty] = useState(initial.document != null);
  const [pendingToggle, setPendingToggle] = useState<ProfileSection | null>(null);
  const [name, setName] = useState(initial.name ?? "CV de base");
  // Document que le prochain enregistrement révise : celui rouvert, puis celui enregistré
  // ici, pour qu'un second ⌘S ajoute une version plutôt qu'un second CV de base.
  const [revises, setRevises] = useState<string | null>(initial.documentId ?? null);
  // Tant qu'un document rouvert n'a pas subi sa première bascule confirmée, la composition
  // ne doit pas partir : sa réponse écraserait l'affichage du document enregistré dès
  // qu'elle arrive, avant toute confirmation.
  const [composeEnabled, setComposeEnabled] = useState(initial.document == null);

  const profile = useQuery({ queryKey: PROFILE_KEY, queryFn: profileService.load });
  const composed = useQuery({
    queryKey: [...BASE_RESUME_KEY, excluded],
    queryFn: () => documentsService.composeBaseResume(excluded),
    enabled: composeEnabled,
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
      setComposeEnabled(true);
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
    setComposeEnabled(true);
    setExcluded((current) => toggleSection(current, section));
  }

  function cancelToggle(): void {
    setPendingToggle(null);
  }

  const saveMutation = useMutation({
    mutationFn: (content: ResumeDocument) => {
      const base: BaseResume = {
        schema_version: RESUME_WORKSPACE_VERSION,
        kind: RESUME_BASE_KIND,
        document: content,
        excluded_sections: excluded,
      };
      return documentsService.saveResume({
        name,
        content: base,
        ...(revises ? { revises, version_note: "Modifiée depuis le CV de base" } : { version_note: "Composée depuis le profil" }),
      });
    },
    onSuccess: async (saved) => {
      await queryClient.invalidateQueries({ queryKey: RESUME_KEY });
      notify({ tone: "success", title: revises ? "Nouvelle version enregistrée" : "CV ajouté à la bibliothèque" });
      setRevises(saved.id);
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
