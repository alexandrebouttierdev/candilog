import { useState } from "react";
import type { GeneratorStep } from "@/shared/ui";

interface Entry {
  readonly label: string;
  /** Temps écoulé de l'opération au début de l'étape. */
  readonly startMs: number;
  readonly endMs: number | null;
}

/**
 * Déroulé d'une génération pour la colonne « Étapes » : les étapes prévues, dans leur
 * ordre, avec la durée mesurée de celles réellement traversées. La durée vient du chronomètre de
 * l'opération (`elapsedMs`) relevé à chaque changement d'étape, jamais d'une estimation.
 *
 * Ajusté pendant le rendu, sans effet : l'état ne fait que suivre les props reçues.
 */
export function useStepLog(
  planned: readonly string[],
  step: string | null,
  running: boolean,
  elapsedMs: number,
): GeneratorStep[] {
  const [entries, setEntries] = useState<readonly Entry[]>([]);
  const [lastStep, setLastStep] = useState<string | null>(null);
  const [wasRunning, setWasRunning] = useState(running);

  if (running && !wasRunning) {
    // Nouvelle génération : le déroulé repart de zéro.
    setWasRunning(true);
    setLastStep(null);
    setEntries([]);
  } else if (!running && wasRunning) {
    setWasRunning(false);
    setEntries((current) => current.map((entry) => (entry.endMs === null ? { ...entry, endMs: elapsedMs } : entry)));
  } else if (running && step && step !== lastStep && step !== "Terminé") {
    setLastStep(step);
    setEntries((current) => [
      ...current.map((entry) => (entry.endMs === null ? { ...entry, endMs: elapsedMs } : entry)),
      { label: step, startMs: elapsedMs, endMs: null },
    ]);
  }

  const toStep = (entry: Entry): GeneratorStep => ({
    label: entry.label,
    state: entry.endMs === null ? "running" : "done",
    ms: entry.endMs === null ? null : Math.max(0, entry.endMs - entry.startMs),
  });
  const byLabel = new Map(entries.map((entry) => [entry.label, entry]));
  // Une étape prévue dont l'événement a été manqué (émis avant l'écoute) est forcément
  // passée si une étape suivante a été reçue : elle est faite, sans durée inventée.
  const reached = Math.max(-1, ...planned.map((label, index) => (byLabel.has(label) ? index : -1)));
  return [
    ...planned.map((label, index): GeneratorStep => {
      const entry = byLabel.get(label);
      if (entry) return toStep(entry);
      return { label, state: index < reached ? "done" : "pending", ms: null };
    }),
    // Étapes annoncées par le backend hors du plan : à la suite, dans l'ordre reçu.
    ...entries.filter((entry) => !planned.includes(entry.label)).map(toStep),
  ];
}
