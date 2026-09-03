import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { DashboardPage } from "../DashboardPage";
import { analyticsService } from "../../../services/analyticsService";
import type { Dashboard } from "@/shared/types/generated/analytics";
import { AppError } from "@/shared/types/app-error";

const DASHBOARD_IPC = {
  metrics: {
    applications: 10,
    interviews: 2,
    responses: 4,
    rejected: 1,
    pending: 5,
    followed_up: 2,
    response_rate: 40,
    interview_rate: 20,
  },
  performance: {
    average_response_days: 7,
    applications_per_week: 2.5,
    upcoming_interviews: 3,
    overdue_follow_ups: 1,
  },
  upcoming_items: [
    {
      id: "relance-1",
      kind: "relance",
      date: "2026-08-29",
      job_title: "Designer produit",
      company_name: "Atlas Studio",
      detail: "LinkedIn",
    },
    {
      id: "entretien-1",
      kind: "entretien",
      date: "2026-08-29T14:30:00+02:00",
      job_title: "Développeur Frontend",
      company_name: "Nova Digital",
      detail: "Visio",
    },
  ],
  pipeline: [],
  activity: [],
  recent: [],
} satisfies Dashboard;

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return (
    <QueryClientProvider client={client}>
      <MemoryRouter>{children}</MemoryRouter>
    </QueryClientProvider>
  );
}

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("écran Aujourd'hui", () => {
  it("affiche les statistiques discrètes et le compteur d'entretiens", async () => {
    vi.spyOn(analyticsService, "dashboard").mockResolvedValue(DASHBOARD_IPC);

    render(<DashboardPage />, { wrapper });

    await waitFor(() => {
      expect(screen.getByText("Candidatures")).toBeInTheDocument();
    });
    expect(screen.getByText("10")).toBeInTheDocument();
    expect(screen.getByText("Prochainement")).toBeInTheDocument();
    // Chaque bloc de l'écran est un panneau nommé posé sur la surface, pas une bande à filet.
    const stats = screen.getByRole("region", { name: "30 derniers jours" });
    expect(stats).toHaveTextContent("Entretiens2");
    expect(stats).not.toHaveTextContent("À relancer");
    expect(screen.getByRole("region", { name: "Prochainement" })).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Candidatures récentes" })).toBeInTheDocument();
    expect(screen.getByText("Relancer les candidatures en retard")).toBeInTheDocument();
    expect(screen.getAllByText("Nova Digital").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Atlas Studio").length).toBeGreaterThan(0);
    expect(screen.queryByText("Rien de prévu")).not.toBeInTheDocument();
  });

  it("affiche un squelette pendant le chargement", () => {
    vi.spyOn(analyticsService, "dashboard").mockReturnValue(new Promise(() => undefined));

    render(<DashboardPage />, { wrapper });

    expect(screen.getByRole("status", { name: "Chargement de l'écran" })).toBeInTheDocument();
  });

  it("propose de réessayer quand le chargement échoue", async () => {
    const charger = vi
      .spyOn(analyticsService, "dashboard")
      .mockRejectedValue(new AppError({ code: "DATABASE_ERROR", message: "Base illisible." }));

    render(<DashboardPage />, { wrapper });

    expect(await screen.findByText("Base illisible.")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Réessayer" }));

    await waitFor(() => expect(charger).toHaveBeenCalledTimes(2));
  });

  it("montre la répartition du pipeline dès qu'il contient une candidature", async () => {
    vi.spyOn(analyticsService, "dashboard").mockResolvedValue({
      ...DASHBOARD_IPC,
      pipeline: [
        { label: "En attente", count: 5, percentage: 50 },
        { label: "Relancées", count: 2, percentage: 20 },
        { label: "Entretien", count: 2, percentage: 20 },
        { label: "Refusées", count: 1, percentage: 10 },
      ],
    });

    render(<DashboardPage />, { wrapper });

    await waitFor(() => expect(screen.getByText("Pipeline")).toBeInTheDocument());
    expect(
      screen.getByRole("img", { name: "Répartition des candidatures par statut" }),
    ).toBeInTheDocument();
    expect(screen.getByText("En attente")).toBeInTheDocument();
  });

  it("reste équilibré quand rien n'est prévu mais que des candidatures existent", async () => {
    vi.spyOn(analyticsService, "dashboard").mockResolvedValue({
      ...DASHBOARD_IPC,
      performance: {
        ...DASHBOARD_IPC.performance,
        upcoming_interviews: 0,
        overdue_follow_ups: 0,
      },
      upcoming_items: [],
      pipeline: [{ label: "En attente", count: 3, percentage: 100 }],
    });

    render(<DashboardPage />, { wrapper });

    // Ni écran vide global, ni simple phrase grise : un vrai état vide de section.
    await waitFor(() => expect(screen.getByText("Rien de prévu aujourd’hui")).toBeInTheDocument());
    expect(screen.getByText("Vous êtes à jour.")).toBeInTheDocument();
    expect(screen.getByText("Prochainement")).toBeInTheDocument();
    expect(screen.getByText("Pipeline")).toBeInTheDocument();
  });

  it("propose un état vide quand le bureau n'a aucune donnée", async () => {
    vi.spyOn(analyticsService, "dashboard").mockResolvedValue({
      ...DASHBOARD_IPC,
      metrics: { ...DASHBOARD_IPC.metrics, applications: 0, responses: 0, interviews: 0 },
      performance: {
        ...DASHBOARD_IPC.performance,
        upcoming_interviews: 0,
        overdue_follow_ups: 0,
      },
      upcoming_items: [],
      activity: [{ start: "2026-08-24", count: 0 }],
      recent: [],
    });

    render(<DashboardPage />, { wrapper });

    await waitFor(() => {
      expect(screen.getByText("Rien de prévu pour aujourd’hui")).toBeInTheDocument();
    });
    expect(
      screen.getByText(
        "Vous êtes à jour. Ajoutez une candidature pour lancer le suivi, ou revenez sur celles déjà envoyées.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Ajouter une candidature" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Voir les candidatures" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Voir les prochains entretiens" })).toBeInTheDocument();
    expect(screen.queryByText("Prochainement")).not.toBeInTheDocument();
  });
});
