import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { ActivityChart } from "../ActivityChart";

describe("ActivityChart", () => {
  it("trace une barre par semaine et annonce chaque valeur aux lecteurs d'écran", () => {
    const { container } = render(
      <ActivityChart
        activity={[
          { start: "2026-08-17", count: 1 },
          { start: "2026-08-24", count: 3 },
        ]}
      />,
    );

    expect(
      screen.getByRole("img", { name: "Candidatures envoyées par semaine" }),
    ).toBeInTheDocument();
    expect(container.querySelectorAll(".recharts-bar-rectangle")).toHaveLength(2);
    expect(screen.getByText(/Semaine du 17 août : 1 candidature/)).toBeInTheDocument();
    expect(screen.getByText(/Semaine du 24 août : 3 candidatures/)).toBeInTheDocument();
  });

  it("remplace le graphique par un état vide quand aucune semaine n'a d'activité", () => {
    render(
      <ActivityChart
        activity={[
          { start: "2026-08-17", count: 0 },
          { start: "2026-08-24", count: 0 },
        ]}
      />,
    );

    expect(screen.getByText("Pas encore d’activité")).toBeInTheDocument();
    expect(screen.queryByRole("img")).not.toBeInTheDocument();
  });

  it("supporte une série longue et des valeurs extrêmes", () => {
    const activity = Array.from({ length: 52 }, (_, index) => ({
      start: `2026-01-${String((index % 28) + 1).padStart(2, "0")}`,
      count: index === 0 ? 0 : index * 137,
    }));

    const { container } = render(<ActivityChart activity={activity} shortLabels />);

    expect(container.querySelectorAll(".recharts-bar-rectangle")).toHaveLength(52);
  });

  it("suit un changement de série sans remontage", () => {
    const { container, rerender } = render(
      <ActivityChart activity={[{ start: "2026-08-17", count: 1 }]} />,
    );
    expect(container.querySelectorAll(".recharts-bar-rectangle")).toHaveLength(1);

    rerender(
      <ActivityChart
        activity={[
          { start: "2026-08-17", count: 1 },
          { start: "2026-08-24", count: 5 },
          { start: "2026-08-31", count: 2 },
        ]}
      />,
    );

    expect(container.querySelectorAll(".recharts-bar-rectangle")).toHaveLength(3);
    expect(screen.getByText(/Semaine du 31 août : 2 candidatures/)).toBeInTheDocument();
  });

  it("se démonte et se remonte sans erreur", () => {
    const activity = [{ start: "2026-08-17", count: 2 }];
    const { unmount } = render(<ActivityChart activity={activity} />);
    unmount();

    const { container } = render(<ActivityChart activity={activity} />);
    expect(container.querySelectorAll(".recharts-bar-rectangle")).toHaveLength(1);
  });
});
