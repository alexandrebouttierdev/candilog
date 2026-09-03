import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { AnalyticsPage } from "../AnalyticsPage";
import { analyticsService } from "../../../services/analyticsService";
import { applicationService } from "@/features/applications/services/applicationService";
import { followUpService } from "@/features/followups/services/followUpService";
import type { Analytics } from "@/shared/types/generated/analytics";
import type { Application } from "@/shared/types/generated/applications";

const ANALYTICS: Analytics = {
  metrics: {
    applications: 12,
    interviews: 3,
    responses: 5,
    rejected: 2,
    pending: 7,
    followed_up: 1,
    response_rate: 42,
    interview_rate: 25,
  },
  performance: {
    average_response_days: 9,
    applications_per_week: 3,
    upcoming_interviews: 1,
    overdue_follow_ups: 2,
  },
  activity: [{ start: "2026-08-24", count: 4 }],
  funnel: [
    { label: "Envoyées", count: 12, percentage: 100 },
    { label: "Réponses", count: 5, percentage: 42 },
    { label: "Entretiens", count: 3, percentage: 25 },
    { label: "Refus", count: 2, percentage: 17 },
  ],
  to_follow_up: [
    {
      id: "candidature-42",
      job_title: "Développeur Rust",
      company_name: "Nova Digital",
      sent_date: "2026-08-10",
      days: 18,
    },
  ],
};

function candidature(): Application {
  return {
    id: "candidature-42",
    job_title: "Développeur Rust",
    company_id: "e1",
    company_name: "Nova Digital",
    company_size: "PME",
    contact_id: null,
    application_type: "OFFRE",
    contract_type_code: "CDI",
    contract_type_name: "CDI",
    weekly_work_schedule: "FULL_TIME",
    weekly_hours: 35,
    professional_domain_id: "M18",
    professional_domain_name: "Informatique / Télécommunication",
    city: null,
    address: null,
    company_type_id: null,
    effective_city: "Rennes",
    effective_address: null,
    effective_company_type_id: null,
    effective_company_type_name: null,
    status: "EN_ATTENTE",
    sent_date: "2026-08-10",
    job_url: null,
    notes: null,
    created_at: "2026-08-10T00:00:00Z",
    updated_at: "2026-08-10T00:00:00Z",
  };
}

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
  vi.spyOn(analyticsService, "load").mockResolvedValue(ANALYTICS);
  vi.spyOn(applicationService, "get").mockResolvedValue(candidature());
});

describe("écran Analyses — candidatures à relancer", () => {
  it("ouvre la relance avec la candidature déjà sélectionnée", async () => {
    const creer = vi.spyOn(followUpService, "create").mockResolvedValue({
      id: "relance-1",
      application_id: "candidature-42",
      application_job_title: "Développeur Rust",
      company_name: "Nova Digital",
      follow_up_date: "2026-09-03",
      type: "Email",
      notes: null,
      created_at: "2026-09-03T00:00:00Z",
    });

    render(<AnalyticsPage />, { wrapper });

    await waitFor(() => expect(screen.getByText("Développeur Rust")).toBeInTheDocument());
    await userEvent.click(screen.getByRole("button", { name: /Relancer/ }));

    const modale = await screen.findByRole("dialog", { name: "Nouvelle relance" });
    expect(modale).toBeInTheDocument();

    // La candidature est déjà choisie : l'utilisateur n'a pas à la rechercher une seconde fois.
    await waitFor(() =>
      expect(within(modale).getByLabelText(/^Candidature/)).toHaveValue(
        "Développeur Rust — Nova Digital",
      ),
    );

    await userEvent.click(within(modale).getByRole("button", { name: "Programmer" }));

    await waitFor(() =>
      expect(creer).toHaveBeenCalledWith(
        expect.objectContaining({ application_id: "candidature-42" }),
      ),
    );
  });
});
