import { render, screen } from "@testing-library/react";
import { A4Preview, AiProgress, ScoreBadge } from "../DocumentUi";

describe("DocumentUi", () => {
  it("annonce l'état vide de l'aperçu A4", () => {
    render(<A4Preview />);
    expect(screen.getByLabelText("Aperçu du document")).toHaveTextContent(
      "Le document apparaîtra ici après la génération.",
    );
  });

  it("rend le score ATS de façon textuelle, pas seulement par la couleur", () => {
    render(<ScoreBadge value={82} />);
    expect(screen.getByText("82")).toBeInTheDocument();
    expect(screen.getByText("Score ATS")).toBeInTheDocument();
    expect(screen.getByText("sur 100")).toBeInTheDocument();
  });

  it("expose l'étape et le temps écoulé, sans pourcentage inventé", () => {
    render(
      <AiProgress
        progress={{ generation_id: "op-1", step: "Analyse ATS", chunk: null }}
        elapsedMs={72_000}
      />,
    );
    expect(screen.getByRole("status")).toHaveTextContent("Analyse ATS");
    expect(screen.getByRole("status")).toHaveTextContent("01:12");
    expect(screen.getByRole("status")).not.toHaveTextContent("%");
  });
});
