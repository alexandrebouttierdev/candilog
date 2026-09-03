import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { FunnelChart } from "../FunnelChart";

const ETAPES = [
  { label: "Envoyées", count: 10, percentage: 100 },
  { label: "Réponses", count: 6, percentage: 60 },
  { label: "Entretiens", count: 3, percentage: 30 },
  { label: "Refus", count: 2, percentage: 20 },
];

describe("FunnelChart", () => {
  it("nomme et chiffre chaque étape sans dépendre du survol", () => {
    render(<FunnelChart steps={ETAPES} />);

    expect(
      screen.getByRole("img", { name: "Conversion des candidatures, étape par étape" }),
    ).toBeInTheDocument();
    for (const etape of ETAPES) {
      expect(screen.getByText(etape.label)).toBeInTheDocument();
      expect(
        screen.getByText(
          new RegExp(`${etape.label} : ${etape.count} candidature.*${etape.percentage} %`),
        ),
      ).toBeInTheDocument();
    }
  });

  it("affiche un état vide quand aucune étape n'est atteinte", () => {
    render(
      <FunnelChart
        steps={ETAPES.map((etape) => ({ ...etape, count: 0, percentage: 0 }))}
      />,
    );

    expect(screen.getByText("Entonnoir vide")).toBeInTheDocument();
    expect(screen.queryByRole("img")).not.toBeInTheDocument();
  });

  it("se démonte et se remonte sans erreur", () => {
    const { unmount } = render(<FunnelChart steps={ETAPES} />);
    unmount();

    render(<FunnelChart steps={ETAPES} />);
    expect(screen.getByText("Envoyées")).toBeInTheDocument();
  });
});
