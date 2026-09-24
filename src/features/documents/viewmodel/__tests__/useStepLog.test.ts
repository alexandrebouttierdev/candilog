import { describe, expect, it } from "vitest";
import { renderHook } from "@testing-library/react";
import { useStepLog } from "../useStepLog";

const PLANNED = ["Rédaction", "Relecture du français"] as const;

describe("déroulé d'une génération", () => {
  it("mesure la durée de chaque étape traversée", () => {
    const { result, rerender } = renderHook(({ step, running, elapsed }) => useStepLog(PLANNED, step, running, elapsed), {
      initialProps: { step: null as string | null, running: true, elapsed: 0 },
    });
    rerender({ step: "Rédaction", running: true, elapsed: 0 });
    rerender({ step: "Relecture du français", running: true, elapsed: 3000 });

    expect(result.current).toEqual([
      { label: "Rédaction", state: "done", ms: 3000 },
      { label: "Relecture du français", state: "running", ms: null },
    ]);
  });

  it("garde l'ordre prévu quand la première étape n'a pas été reçue", () => {
    const { result, rerender } = renderHook(({ step, running, elapsed }) => useStepLog(PLANNED, step, running, elapsed), {
      initialProps: { step: null as string | null, running: true, elapsed: 0 },
    });
    rerender({ step: "Relecture du français", running: true, elapsed: 2000 });

    // La rédaction est forcément passée, mais sa durée n'a pas été mesurée : aucune n'est inventée.
    expect(result.current).toEqual([
      { label: "Rédaction", state: "done", ms: null },
      { label: "Relecture du français", state: "running", ms: null },
    ]);

    rerender({ step: "Relecture du français", running: false, elapsed: 4000 });
    expect(result.current.map((step) => step.state)).toEqual(["done", "done"]);
  });
});
