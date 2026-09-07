import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ImportAnalysisPanel } from "../ImportAnalysisPanel";

describe("ImportAnalysisPanel", () => {
  it("affiche une analyse sans pourcentage", () => {
    render(
      <ImportAnalysisPanel
        step="Analyse du CV…"
        elapsedMs={12_000}
        entries={[{ at: "2026-08-29T14:32:01.000Z", message: "Analyse démarrée" }]}
        stopping={false}
        onStop={() => undefined}
      />,
    );

    expect(screen.getByText("Analyse du CV en cours…")).toBeInTheDocument();
    expect(screen.getByText("Temps écoulé : 00:12")).toBeInTheDocument();
    expect(screen.getByText("Analyse du CV…")).toBeInTheDocument();
    expect(screen.queryByText(/%/)).not.toBeInTheDocument();
    expect(screen.queryByText("42 %")).not.toBeInTheDocument();
  });

  it("ajoute le total de tokens au temps écoulé quand il est connu", () => {
    render(
      <ImportAnalysisPanel
        step="Analyse du CV…"
        elapsedMs={12_000}
        entries={[]}
        tokens_used={1_024}
        stopping={false}
        onStop={() => undefined}
      />,
    );

    expect(screen.getByText("Temps écoulé : 00:12 · 1 024 tokens · 85,3 tokens/s")).toBeInTheDocument();
  });

  it("affiche un zéro communiqué par le fournisseur", () => {
    render(
      <ImportAnalysisPanel
        step="Analyse du CV…"
        elapsedMs={12_000}
        entries={[]}
        tokens_used={0}
        stopping={false}
        onStop={() => undefined}
      />,
    );

    expect(screen.getByText("Temps écoulé : 00:12 · 0 tokens")).toBeInTheDocument();
  });

  it("permet d'arrêter l'analyse", () => {
    render(
      <ImportAnalysisPanel
        step={null}
        elapsedMs={0}
        entries={[]}
        stopping
        onStop={() => undefined}
      />,
    );

    expect(screen.getByRole("button", { name: "Arrêt…" })).toBeDisabled();
  });
});

  it("affiche le débit natif à côté des tokens", () => {
    render(
      <ImportAnalysisPanel
        step="Analyse du CV… 23 tokens · 0.6 tokens/s · 39 s"
        elapsedMs={39_000}
        entries={[]}
        tokens_used={23}
        tokens_per_second={0.6}
        stopping={false}
        onStop={() => undefined}
      />,
    );

    expect(
      screen.getByText("Temps écoulé : 00:39 · 23 tokens · 0,6 tokens/s"),
    ).toBeInTheDocument();
  });

  it("estime le débit quand seul le total de tokens est connu", () => {
    render(
      <ImportAnalysisPanel
        step="Analyse du CV…"
        elapsedMs={10_000}
        entries={[]}
        tokens_used={20}
        stopping={false}
        onStop={() => undefined}
      />,
    );

    expect(
      screen.getByText("Temps écoulé : 00:10 · 20 tokens · 2,0 tokens/s"),
    ).toBeInTheDocument();
  });

