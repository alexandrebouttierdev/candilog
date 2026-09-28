import { useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { create } from "zustand";
import type { AiTask, Settings } from "@/shared/types/generated/settings";
import { useUiStore } from "@/shared/lib/ui-store";
import { AI_TASKS, remoteSendToConfirm, tasksSentTo } from "../model/taskRouting";
import type { TaskDestination } from "../model/taskRouting";
import { settingsService } from "../services/settingsService";
import { SETTINGS_KEY } from "./useSettingsViewModel";

/** Réponse au dialogue de premier envoi distant. */
export interface RemoteSendAnswer {
  readonly send: boolean;
  /** « Ne plus demander pour … » : mémorisé dans les réglages. */
  readonly remember: boolean;
}

/** Envoi en attente de confirmation, tel que le dialogue le présente. */
export interface RemoteSendRequest {
  readonly destination: TaskDestination;
  readonly taskLabel: string;
  /** Ce qui quitte l'ordinateur, en clair : « Votre profil et le texte de l'offre ». */
  readonly content: string;
  /** Tâches routées vers ce fournisseur, sur les cinq. */
  readonly tasks: number;
}

interface RemoteSendState {
  pending: (RemoteSendRequest & { resolve: (answer: RemoteSendAnswer) => void }) | null;
  ask: (request: RemoteSendRequest) => Promise<RemoteSendAnswer>;
  answer: (answer: RemoteSendAnswer) => void;
}

/** État d'interface du dialogue : une seule demande à la fois. */
export const useRemoteSendStore = create<RemoteSendState>((set, get) => ({
  pending: null,
  ask: (request) =>
    new Promise<RemoteSendAnswer>((resolve) => {
      // Une seconde demande pendant la première ne s'empile pas : elle n'envoie rien.
      if (get().pending) {
        resolve({ send: false, remember: false });
        return;
      }
      set({ pending: { ...request, resolve } });
    }),
  answer: (answer) => {
    get().pending?.resolve(answer);
    set({ pending: null });
  },
}));

/**
 * Garde du premier envoi à un service distant (`reference_design/DECISIONS.md` D4) : à
 * appeler juste avant d'envoyer une tâche. Elle laisse passer une tâche locale ou un
 * fournisseur déjà accepté ; sinon elle ouvre le dialogue et résout `false` si
 * l'utilisateur annule — rien n'est alors envoyé.
 */
export function useRemoteSendGuard() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  return useCallback(
    async (task: AiTask, content: string): Promise<boolean> => {
      let settings: Settings;
      try {
        settings = await queryClient.fetchQuery({ queryKey: SETTINGS_KEY, queryFn: settingsService.load });
      } catch {
        // Réglages illisibles : le backend choisit son fournisseur dans ces mêmes réglages
        // (`load_task_config`) et échouera sans rien envoyer. C'est lui qui dira pourquoi.
        return true;
      }
      const destination = remoteSendToConfirm(task, settings);
      if (!destination) return true;
      const answer = await useRemoteSendStore.getState().ask({
        destination,
        taskLabel: AI_TASKS.find((entry) => entry.value === task)?.label ?? task,
        content,
        tasks: tasksSentTo(destination.providerId, settings),
      });
      if (!answer.send) return false;
      if (answer.remember) {
        try {
          await settingsService.save({
            ...settings,
            remote_send_consents: [...settings.remote_send_consents, destination.providerId],
          });
          await queryClient.invalidateQueries({ queryKey: SETTINGS_KEY });
        } catch {
          // L'envoi reste accepté pour cette fois ; seule la mémorisation a échoué.
          notify({ tone: "error", title: "Préférence non enregistrée", detail: "La confirmation sera redemandée." });
        }
      }
      return true;
    },
    [queryClient, notify],
  );
}
