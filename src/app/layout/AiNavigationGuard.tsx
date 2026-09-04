import { useCallback, useEffect, useState } from "react";
import { useBlocker, type BlockerFunction } from "react-router-dom";
import { useAiOperationStore, type AiOperationKind } from "@/features/ai/viewmodel/ai-operation-store";
import { useUiStore } from "@/shared/lib/ui-store";
import { ConfirmDialog } from "@/shared/ui";

const DESCRIPTIONS: Record<AiOperationKind, string> = {
  analyse: "L’analyse en cours sera arrêtée avant de quitter cet écran.",
  generation: "La génération en cours sera arrêtée avant de quitter cet écran.",
  import: "L’import en cours sera arrêté avant de quitter cet écran.",
};

/** Bloque les changements d'écran tant qu'une opération IA doit être arrêtée. */
export function AiNavigationGuard() {
  const active = useAiOperationStore((state) => state.active);
  const notify = useUiStore((state) => state.notify);
  const [waiting, setWaiting] = useState(false);
  const blocker = useBlocker(
    useCallback<BlockerFunction>(
      ({ currentLocation, nextLocation }) =>
        active !== null &&
        (currentLocation.pathname !== nextLocation.pathname ||
          currentLocation.search !== nextLocation.search ||
          currentLocation.hash !== nextLocation.hash),
      [active],
    ),
  );

  useEffect(() => {
    if (blocker.state === "blocked" && active === null && !waiting) {
      blocker.proceed();
    }
  }, [active, blocker, waiting]);

  const cancelNavigation = () => {
    if (blocker.state === "blocked") blocker.reset();
  };

  const stopAndLeave = async () => {
    if (blocker.state !== "blocked" || !active || waiting) return;
    setWaiting(true);
    try {
      await active.stop();
      setWaiting(false);
      blocker.proceed();
    } catch {
      setWaiting(false);
      notify({ tone: "error", title: "Arrêt impossible" });
    }
  };

  return (
    <ConfirmDialog
      open={blocker.state === "blocked"}
      title="Quitter cet écran ?"
      description={active ? DESCRIPTIONS[active.kind] : "Le traitement en cours sera arrêté."}
      confirmLabel="Quitter et arrêter"
      confirmIcon="stop"
      busy={waiting || active?.stopping === true}
      cancelDisabled={waiting || active?.stopping === true}
      dismissDisabled={waiting || active?.stopping === true}
      onCancel={cancelNavigation}
      onConfirm={() => void stopAndLeave()}
    />
  );
}
