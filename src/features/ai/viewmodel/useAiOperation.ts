import { useCallback, useEffect, useRef } from "react";
import { aiService, generation_id } from "../services/aiService";
import {
  useAiOperationStore,
  type AiOperationKind,
} from "./ai-operation-store";

export function useAiOperation() {
  const operation = useAiOperationStore((state) => state.active);
  const ownedIdRef = useRef<string | null>(null);
  const mountedRef = useRef(true);
  const cancellationsRef = useRef(new Map<string, Promise<void>>());
  const completedDuringCancellationRef = useRef(new Set<string>());

  const cancel = useCallback((id: string): Promise<void> => {
    const pending = cancellationsRef.current.get(id);
    if (pending) return pending;

    const state = useAiOperationStore.getState();
    if (state.active?.id !== id) return Promise.resolve();

    // Le bouton et la garde voient l'état occupé avant que l'IPC puisse se résoudre.
    state.markStopping(id, true);
    const cancellation = aiService.cancel(id)
      .then(() => {
        completedDuringCancellationRef.current.delete(id);
        if (!mountedRef.current) return;
        useAiOperationStore.getState().finish(id);
        if (ownedIdRef.current === id) ownedIdRef.current = null;
      })
      .catch((error: unknown) => {
        if (mountedRef.current) {
          if (completedDuringCancellationRef.current.delete(id)) {
            useAiOperationStore.getState().finish(id);
            if (ownedIdRef.current === id) ownedIdRef.current = null;
          } else {
            useAiOperationStore.getState().markStopping(id, false);
          }
        }
        throw error;
      })
      .finally(() => {
        cancellationsRef.current.delete(id);
      });
    cancellationsRef.current.set(id, cancellation);
    return cancellation;
  }, []);

  const start = useCallback((kind: AiOperationKind): string => {
    if (useAiOperationStore.getState().active) {
      throw new Error("Impossible de démarrer une opération IA : une opération est déjà active.");
    }
    const id = generation_id();
    ownedIdRef.current = id;
    useAiOperationStore.getState().begin({
      id,
      kind,
      stop: () => cancel(id),
    });
    return id;
  }, [cancel]);

  const stop = useCallback((): Promise<void> => {
    const id = ownedIdRef.current;
    return id ? cancel(id) : Promise.resolve();
  }, [cancel]);

  const finish = useCallback((id: string): void => {
    const active = useAiOperationStore.getState().active;
    if (active?.id === id && active.stopping) {
      // L'appel métier peut se terminer avant l'IPC d'annulation. On garde alors la
      // navigation bloquée et le bouton en état d'attente jusqu'au verdict de l'arrêt.
      completedDuringCancellationRef.current.add(id);
      return;
    }
    useAiOperationStore.getState().finish(id);
    if (ownedIdRef.current === id) ownedIdRef.current = null;
  }, []);

  const isCurrent = useCallback((id: string): boolean => {
    const active = useAiOperationStore.getState().active;
    return ownedIdRef.current === id && active?.id === id && !active.stopping;
  }, []);

  useEffect(() => {
    mountedRef.current = true;
    const cancellations = cancellationsRef.current;
    return () => {
      mountedRef.current = false;
      const id = ownedIdRef.current;
      if (!id) return;

      useAiOperationStore.getState().finish(id);
      if (!cancellations.has(id)) {
        // Après démontage, l'échec ne peut plus être présenté par cet écran.
        void aiService.cancel(id).catch(() => undefined);
      }
    };
  }, []);

  return {
    operation,
    stopping: operation?.stopping ?? false,
    start,
    stop,
    finish,
    isCurrent,
  };
}
