import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { DashboardPage } from "../DashboardPage";
import { analyticsService } from "../../../services/analyticsService";
import type { Dashboard } from "@/shared/types/generated/analytics";

/**
 * Payload tel que serde le sérialise (`rename_all = "snake_case"`), pas tel que le
 * type généré l'a parfois laissé en camelCase. Le tableau de bord doit lire ces clés.
 */
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
  upcoming_items: [],
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

describe("tableau de bord", () => {
  it("affiche le compteur d'entretiens à venir renvoyé en snake_case par l'IPC", async () => {
    vi.spyOn(analyticsService, "dashboard").mockResolvedValue(DASHBOARD_IPC);

    render(<DashboardPage />, { wrapper });

    await waitFor(() => {
      expect(screen.getByText("Entretiens à venir")).toBeInTheDocument();
    });
    expect(screen.getByText("Entretiens à venir").closest("div")?.parentElement).toHaveTextContent(
      "3",
    );
  });
});
