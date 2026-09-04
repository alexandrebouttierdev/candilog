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
      />,
    );

    expect(screen.getByText("Temps écoulé : 00:12 · 1 024 tokens")).toBeInTheDocument();
  });

  it("affiche un zéro communiqué par le fournisseur", () => {
    render(
      <ImportAnalysisPanel
        step="Analyse du CV…"
        elapsedMs={12_000}
        entries={[]}
        tokens_used={0}
      />,
    );

    expect(screen.getByText("Temps écoulé : 00:12 · 0 tokens")).toBeInTheDocument();
  });
});
