import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { PipelineChart } from "../PipelineChart";

const STATUTS = [
  { label: "En attente", count: 7, percentage: 47 },
  { label: "Relancées", count: 4, percentage: 27 },
  { label: "Entretien", count: 3, percentage: 20 },
  { label: "Refusées", count: 1, percentage: 6 },
];

describe("PipelineChart", () => {
  it("légende chaque segment par son libellé et son compte", () => {
    render(<PipelineChart steps={STATUTS} />);

    expect(
      screen.getByRole("img", { name: "Répartition des candidatures par statut" }),
    ).toBeInTheDocument();
    for (const statut of STATUTS) {
      expect(screen.getByText(statut.label)).toBeInTheDocument();
      expect(screen.getByText(String(statut.count))).toBeInTheDocument();
    }
  });

  it("affiche un état vide quand le pipeline ne contient rien", () => {
    render(<PipelineChart steps={STATUTS.map((statut) => ({ ...statut, count: 0 }))} />);

    expect(screen.getByText("Pipeline vide")).toBeInTheDocument();
    expect(screen.queryByRole("img")).not.toBeInTheDocument();
  });

  it("se démonte et se remonte sans erreur", () => {
    const { unmount } = render(<PipelineChart steps={STATUTS} />);
    unmount();

    render(<PipelineChart steps={STATUTS} />);
    expect(screen.getByText("En attente")).toBeInTheDocument();
  });
});
