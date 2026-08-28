import { fireEvent, render, screen } from "@testing-library/react";
import { ActivityChart, FollowUpList, FunnelChart } from "../AnalyticsUi";

describe("AnalyticsUi", () => {
  it("annonce chaque semaine du graphique aux lecteurs d'écran", () => {
    render(
      <ActivityChart
        activite={[
          { debut: "2026-08-17", nombre: 1 },
          { debut: "2026-08-24", nombre: 3 },
        ]}
      />,
    );

    expect(screen.getByRole("img", { name: "Candidatures envoyées par semaine" })).toBeInTheDocument();
    expect(screen.getByText(/Semaine du 17 août : 1 candidature/)).toBeInTheDocument();
    expect(screen.getByText(/Semaine du 24 août : 3 candidatures/)).toBeInTheDocument();
  });

  it("affiche le nombre et le pourcentage de chaque étape", () => {
    render(
      <FunnelChart
        etapes={[
          { label: "Envoyées", nombre: 10, pourcentage: 100 },
          { label: "Entretiens", nombre: 3, pourcentage: 30 },
        ]}
      />,
    );

    expect(screen.getByText("Envoyées")).toBeInTheDocument();
    expect(
      screen.getByText((_, element) => element?.textContent === "3 · 30 %"),
    ).toBeInTheDocument();
  });

  it("transmet la candidature choisie à l'action de relance", () => {
    const onRelancer = vi.fn();
    const item = {
      id: "candidate-1",
      poste: "Développeur Rust",
      entrepriseNom: "Nova Digital",
      dateEnvoi: "2026-08-10",
      jours: 18,
    };
    render(<FollowUpList items={[item]} onRelancer={onRelancer} />);

    fireEvent.click(screen.getByRole("button", { name: /Relancer/ }));

    expect(onRelancer).toHaveBeenCalledWith(item);
  });
});
