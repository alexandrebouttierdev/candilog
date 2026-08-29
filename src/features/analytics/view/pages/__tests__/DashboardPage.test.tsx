import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { DashboardPage } from "../DashboardPage";
import { analyticsService } from "../../../services/analyticsService";
import type { Dashboard } from "@/shared/types/generated/analytics";

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
    expect(screen.getByText("30 derniers jours")).toBeInTheDocument();
    expect(screen.getByText("À relancer")).toBeInTheDocument();
    expect(screen.getAllByText("Nova Digital").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Atlas Studio").length).toBeGreaterThan(0);
    expect(screen.queryByText("Rien de prévu")).not.toBeInTheDocument();
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
      expect(screen.getByText("Rien de prévu")).toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: "Nouvelle candidature" })).toBeInTheDocument();
    expect(screen.queryByText("Prochainement")).not.toBeInTheDocument();
  });
});
