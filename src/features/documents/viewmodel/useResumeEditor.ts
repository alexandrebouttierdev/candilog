import { useEffect, useRef, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import * as model from "../model/resumeWorkspace";
import type { ResumeField, ResumeSectionKind } from "../model/resumeWorkspace";
import { documentsService } from "../services/documentsService";
import type { ResumeWorkspace } from "@/shared/types/generated/documents";
import { profileService } from "@/features/profile/services/profileService";
import { PROFILE_KEY } from "@/features/profile/viewmodel/useProfileViewModel";
import { AppError } from "@/shared/types/app-error";
import { runResumeRecalculation } from "./resumeRecalculation";

/** Pile d'annulation/rétablissement bornée : au-delà, les plus anciens états sont perdus. */
const HISTORY_LIMIT = 50;

/** Délai après la dernière frappe manuelle avant le recalcul IPC du score et des propositions. */
const RECALC_DEBOUNCE_MS = 300;

export interface PendingProfileSkill {
  proposal_id: string;
  skill: string;
  label: string;
}

function errorMessage(error: unknown): string {
  return error instanceof AppError ? error.message : "Une erreur inattendue s’est produite.";
}

/**
 * Orchestration de l'éditeur de CV : édition locale immédiate, recalcul IPC différé,
 * décisions ATS et confirmation d'ajout au profil.
 *
 * Le workspace vit en état React local, jamais dans Zustand ni dans le cache TanStack Query
 * — c'est un brouillon actif d'une seule session d'édition, pas une donnée serveur partagée.
 * Un compteur de révision écarte toute réponse de recalcul plus ancienne que la dernière
 * modification locale, sans jamais désactiver ESLint pour faire taire une dépendance.
 */
export function useResumeEditor(initial: ResumeWorkspace) {
  const queryClient = useQueryClient();
  const [workspace, setWorkspace] = useState(initial);
  const [undoStack, setUndoStack] = useState<ResumeWorkspace[]>([]);
  const [redoStack, setRedoStack] = useState<ResumeWorkspace[]>([]);
  const [isRecalculating, setIsRecalculating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pendingProfileSkill, setPendingProfileSkill] = useState<PendingProfileSkill | null>(null);

  const recalcTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const mounted = useRef(true);
  // Incrémenté à chaque changement local (édition, annulation, décision ATS) : une réponse
  // de recalcul dont la révision ne correspond plus à la dernière n'est plus d'actualité.
  const revision = useRef(0);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      revision.current += 1;
      if (recalcTimer.current) clearTimeout(recalcTimer.current);
    };
  }, []);

  function invalidatePendingRecalculation(): void {
    if (recalcTimer.current) clearTimeout(recalcTimer.current);
    revision.current += 1;
  }

  function scheduleRecalculation(base: ResumeWorkspace): void {
    if (recalcTimer.current) clearTimeout(recalcTimer.current);
    revision.current += 1;
    const requested = revision.current;
    recalcTimer.current = setTimeout(() => {
      setIsRecalculating(true);
      void runResumeRecalculation({
        workspace: base,
        recalculate: documentsService.recalculateResume,
        isCurrent: () => mounted.current && revision.current === requested,
        onSuccess: (updated) => {
          setWorkspace(updated);
          setError(null);
        },
        onError: (caught) => setError(errorMessage(caught)),
        onSettled: () => setIsRecalculating(false),
      });
    }, RECALC_DEBOUNCE_MS);
  }

  /** Édition locale immédiate : le document change avant tout aller-retour IPC. */
  function applyLocalChange(next: ResumeWorkspace): void {
    if (next === workspace) return;
    setUndoStack((stack) => [...stack, workspace].slice(-HISTORY_LIMIT));
    setRedoStack([]);
    setWorkspace(next);
    scheduleRecalculation(next);
  }

  function updateField(field: ResumeField, value: string): void {
    applyLocalChange(model.updateResumeField(workspace, field, value));
  }
  function addExperienceBullet(index: number): void {
    applyLocalChange(model.addExperienceBullet(workspace, index));
  }
  function removeExperienceBullet(index: number, item: number): void {
    applyLocalChange(model.removeExperienceBullet(workspace, index, item));
  }
  function addProjectBullet(index: number): void {
    applyLocalChange(model.addProjectBullet(workspace, index));
  }
  function removeProjectBullet(index: number, item: number): void {
    applyLocalChange(model.removeProjectBullet(workspace, index, item));
  }
  function addSkill(group: number): void {
    applyLocalChange(model.addSkill(workspace, group));
  }
  function removeSkill(group: number, item: number): void {
    applyLocalChange(model.removeSkill(workspace, group, item));
  }
  function addSection(section: ResumeSectionKind): void {
    applyLocalChange(model.addSection(workspace, section));
  }
  function removeSection(section: ResumeSectionKind, index: number): void {
    applyLocalChange(model.removeSection(workspace, section, index));
  }

  /** Restaure le document précédent sans appel IPC : la pile porte déjà un état cohérent. */
  function undo(): void {
    const previous = undoStack[undoStack.length - 1];
    if (previous === undefined) return;
    invalidatePendingRecalculation();
    setRedoStack((stack) => [...stack, workspace].slice(-HISTORY_LIMIT));
    setUndoStack((stack) => stack.slice(0, -1));
    setWorkspace(previous);
  }

  function redo(): void {
    const next = redoStack[redoStack.length - 1];
    if (next === undefined) return;
    invalidatePendingRecalculation();
    setUndoStack((stack) => [...stack, workspace].slice(-HISTORY_LIMIT));
    setRedoStack((stack) => stack.slice(0, -1));
    setWorkspace(next);
  }

  /**
   * Accepte une proposition ATS via l'IPC. Une compétence manquante acceptée ouvre la
   * demande séparée d'ajout au profil ; une suggestion textuelle ne l'ouvre jamais.
   */
  async function applyProposal(proposal_id: string): Promise<void> {
    invalidatePendingRecalculation();
    const before = workspace;
    setIsRecalculating(true);
    try {
      const updated = await documentsService.applyResumeProposal(before, proposal_id);
      if (!mounted.current) return;
      setUndoStack((stack) => [...stack, before].slice(-HISTORY_LIMIT));
      setRedoStack([]);
      setWorkspace(updated);
      setError(null);
      const accepted = updated.proposals.find((proposal) => proposal.id === proposal_id);
      if (accepted && accepted.kind === "missing_skill") {
        setPendingProfileSkill({
          proposal_id: accepted.id,
          skill: accepted.proposed_text,
          label: accepted.label,
        });
      }
    } catch (caught) {
      if (mounted.current) setError(errorMessage(caught));
    } finally {
      if (mounted.current) setIsRecalculating(false);
    }
  }

  async function rejectProposal(proposal_id: string): Promise<void> {
    invalidatePendingRecalculation();
    const before = workspace;
    setIsRecalculating(true);
    try {
      const updated = await documentsService.rejectResumeProposal(before, proposal_id);
      if (!mounted.current) return;
      setUndoStack((stack) => [...stack, before].slice(-HISTORY_LIMIT));
      setRedoStack([]);
      setWorkspace(updated);
      setError(null);
    } catch (caught) {
      if (mounted.current) setError(errorMessage(caught));
    } finally {
      if (mounted.current) setIsRecalculating(false);
    }
  }

  /**
   * Annule la décision d'*une* proposition (« Annuler {label} » d'une carte ATS), jamais la
   * dernière modification de la session : contrairement à `undo`, qui dépile la pile
   * partagée par toutes les éditions et décisions, cette fonction cible un `proposal_id`
   * précis et laisse les autres décisions déjà prises intactes, quel que soit ce qui s'est
   * passé depuis.
   */
  async function undoProposal(proposal_id: string): Promise<void> {
    const before = workspace;
    const reverted = model.revertProposalDecision(before, proposal_id);
    if (reverted === before) return;
    invalidatePendingRecalculation();
    setIsRecalculating(true);
    try {
      const updated = await documentsService.recalculateResume(reverted);
      if (!mounted.current) return;
      setUndoStack((stack) => [...stack, before].slice(-HISTORY_LIMIT));
      setRedoStack([]);
      setWorkspace(updated);
      setError(null);
    } catch (caught) {
      if (mounted.current) setError(errorMessage(caught));
    } finally {
      if (mounted.current) setIsRecalculating(false);
    }
  }

  /** « CV uniquement » : referme la demande sans toucher au profil général. */
  function keepSkillInResumeOnly(): void {
    setPendingProfileSkill(null);
  }

  /** « Ajouter au profil » : un échec laisse la compétence acceptée au CV et la demande ouverte. */
  async function addPendingSkillToProfile(): Promise<void> {
    if (!pendingProfileSkill) return;
    try {
      const payload = await profileService.addSkill(pendingProfileSkill.skill);
      queryClient.setQueryData(PROFILE_KEY, payload);
      if (!mounted.current) return;
      setPendingProfileSkill(null);
      setError(null);
    } catch (caught) {
      if (mounted.current) setError(errorMessage(caught));
    }
  }

  return {
    workspace,
    updateField,
    addExperienceBullet,
    removeExperienceBullet,
    addProjectBullet,
    removeProjectBullet,
    addSkill,
    removeSkill,
    addSection,
    removeSection,
    applyProposal,
    rejectProposal,
    undoProposal,
    undo,
    redo,
    canUndo: undoStack.length > 0,
    canRedo: redoStack.length > 0,
    pendingProfileSkill,
    keepSkillInResumeOnly,
    addPendingSkillToProfile,
    isRecalculating,
    error,
  };
}
