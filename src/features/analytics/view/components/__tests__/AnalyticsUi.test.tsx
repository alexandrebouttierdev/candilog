import { fireEvent, render, screen } from "@testing-library/react";
import { ActivityChart, FollowUpList, FunnelChart } from "../AnalyticsUi";

describe("AnalyticsUi", () => {
  it("annonce chaque semaine du graphique aux lecteurs d'écran", () => {
    render(
      <ActivityChart
        activity={[
          { start: "2026-08-17", count: 1 },
          { start: "2026-08-24", count: 3 },
        ]}
      />,
    );

    expect(screen.getByRole("img", { name: "Candidatures envoyées par semaine" })).toBeInTheDocument();
    expect(screen.getByText("1")).toBeInTheDocument();
    expect(screen.getByText("3")).toBeInTheDocument();
    expect(screen.getByText(/Semaine du 17 août : 1 candidature/)).toBeInTheDocument();
    expect(screen.getByText(/Semaine du 24 août : 3 candidatures/)).toBeInTheDocument();
  });

  it("affiche le nombre et le pourcentage de chaque étape", () => {
    render(
      <FunnelChart
        steps={[
          { label: "Envoyées", count: 10, percentage: 100 },
          { label: "Entretiens", count: 3, percentage: 30 },
        ]}
      />,
    );

    expect(screen.getByText("Envoyées")).toBeInTheDocument();
    expect(
      screen.getByText((_, element) => element?.textContent === "3 · 30 %"),
    ).toBeInTheDocument();
  });

  it("transmet la candidature choisie à l'action de relance", () => {
    const onFollowUp = vi.fn();
    const item = {
      id: "candidate-1",
      job_title: "Développeur Rust",
      company_name: "Nova Digital",
      sent_date: "2026-08-10",
      days: 18,
    };
    render(<FollowUpList items={[item]} onFollowUp={onFollowUp} />);

    fireEvent.click(screen.getByRole("button", { name: /Relancer/ }));

    expect(onFollowUp).toHaveBeenCalledWith(item);
  });
});
