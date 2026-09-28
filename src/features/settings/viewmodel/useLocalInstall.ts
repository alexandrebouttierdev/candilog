import { useEffect, useRef, useState } from "react";
import type { LocalModelProbe, ManagedModelDefinition } from "@/shared/types/generated/ai";
import { AppError } from "@/shared/types/app-error";
import { useUiStore } from "@/shared/lib/ui-store";
import { useAiTimer } from "@/features/ai";
import { currentStep, laterStep } from "../model/localInstall";
import type { InstallPhase, InstallStepKey } from "../model/localInstall";
import { managedOllamaService } from "../services/managedOllamaService";
import type { ManagedOllamaViewModel } from "./useManagedOllamaViewModel";

/** Issue d'une installation interrompue, montrée en place sous le choix du modèle. */
export type InstallOutcome = { kind: "cancelled" } | { kind: "error"; message: string } | null;

interface StepMark {
  /** Temps écoulé de l'installation au début de l'étape. */
  readonly start: number;
  readonly end: number | null;
}

function messageOf(error: unknown, fallback: string): string {
  return error instanceof AppError ? error.message : fallback;
}

/**
 * Installation guidée de l'IA locale (`INTERACTIONS.md` §4.3) : choix du modèle, puis
 * moteur, modèle et phrase de test, puis fin. Le téléchargement lui-même reste celui du
 * ViewModel Ollama géré ; ce hook n'en suit que le déroulé.
 *
 * La fin s'annonce en place. Si la surcouche a été fermée entre-temps, un toast prend le
 * relais : l'installation continue sans elle.
 */
export function useLocalInstall(vm: ManagedOllamaViewModel) {
  const notify = useUiStore((state) => state.notify);
  const [phase, setPhase] = useState<InstallPhase>("pick");
  const [outcome, setOutcome] = useState<InstallOutcome>(null);
  const [installed, setInstalled] = useState<ManagedModelDefinition | null>(null);
  const [verifying, setVerifying] = useState(false);
  const [probe, setProbe] = useState<LocalModelProbe | null>(null);
  const [probeError, setProbeError] = useState<string | null>(null);
  const timer = useAiTimer(phase === "run");

  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  // Durées des étapes : relevées sur le chronomètre à chaque changement d'étape, pendant le
  // rendu — l'état ne fait que suivre l'étape affichée.
  const [marks, setMarks] = useState<Partial<Record<InstallStepKey, StepMark>>>({});
  const [lastStep, setLastStep] = useState<InstallStepKey | null>(null);
  // La progression retombe à vide à la fin du téléchargement : l'étape n'avance que.
  const current: InstallStepKey | null =
    phase !== "run" ? null : laterStep(lastStep, verifying ? "verify" : currentStep(vm.progress));
  if (current !== lastStep) {
    setLastStep(current);
    setMarks((previous) => {
      const next = { ...previous };
      const closing = lastStep ? next[lastStep] : undefined;
      if (lastStep && closing && closing.end === null) next[lastStep] = { ...closing, end: timer.elapsedMs };
      if (current && !next[current]) next[current] = { start: timer.elapsedMs, end: null };
      return next;
    });
  }

  const verify = async () => {
    setVerifying(true);
    setProbeError(null);
    try {
      setProbe(await managedOllamaService.probe());
    } catch (error) {
      setProbeError(messageOf(error, "Le modèle local n'a pas répondu à la phrase de test."));
    } finally {
      setVerifying(false);
    }
  };

  const start = async (definition: ManagedModelDefinition) => {
    setOutcome(null);
    setProbe(null);
    setProbeError(null);
    setMarks({});
    setLastStep(null);
    setPhase("run");
    timer.start();
    try {
      await vm.installAsync(definition.id);
    } catch (error) {
      timer.stop();
      setPhase("pick");
      setOutcome(
        error instanceof AppError && error.isCancelled
          ? { kind: "cancelled" }
          : { kind: "error", message: messageOf(error, "L'installation a échoué.") },
      );
      return;
    }
    setInstalled(definition);
    await verify();
    timer.stop();
    setPhase("done");
    if (!mounted.current) notify({ tone: "success", title: `${definition.display_name} est prêt` });
  };

  /** Durée d'une étape terminée, en millisecondes ; `null` tant qu'elle n'est pas finie. */
  const durationOf = (step: InstallStepKey): number | null => {
    const mark = marks[step];
    return mark && mark.end !== null ? mark.end - mark.start : null;
  };

  /** Temps passé dans l'étape en cours, pour le débit du téléchargement. */
  const stepElapsedMs = current && marks[current] ? timer.elapsedMs - (marks[current]?.start ?? 0) : 0;

  return {
    phase,
    outcome,
    installed,
    current,
    verifying,
    probe,
    probeError,
    stepElapsedMs,
    durationOf,
    start: (definition: ManagedModelDefinition) => void start(definition),
    verify: () => void verify(),
    cancel: vm.cancel,
  };
}
