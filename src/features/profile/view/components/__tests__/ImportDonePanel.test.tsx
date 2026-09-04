import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ImportDonePanel } from "../ImportDonePanel";

describe("ImportDonePanel", () => {
  it("présente le bilan en cartes plutôt qu'en liste", () => {
    render(
      <ImportDonePanel
        result={{ added: 51, replaced: 0, skipped: 0 }}
        totalMs={70_000}
        aiMetrics={{ elapsed_ms: 18_400, tokens_used: 1_024 }}
      />,
    );

    expect(screen.getByText("Profil importé")).toBeInTheDocument();
    expect(screen.getByText("51 éléments ont été ajoutés à votre profil.")).toBeInTheDocument();
    expect(screen.getByText("Ajoutés")).toBeInTheDocument();
    expect(screen.getByText("51")).toBeInTheDocument();
    expect(screen.getByText("Terminé en 1 min 10 s")).toBeInTheDocument();
    expect(screen.getByText("Analysé en 18,4 s · 1 024 tokens")).toBeInTheDocument();
    expect(screen.queryByText(/éléments ajoutés/)).not.toBeInTheDocument();
  });
});
