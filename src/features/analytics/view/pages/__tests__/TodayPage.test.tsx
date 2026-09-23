import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { TodayPage } from "../TodayPage";
import { analyticsService } from "../../../services/analyticsService";
import { localIso } from "../../../model/agenda";
import { applicationService } from "@/features/applications";
import { followUpService } from "@/features/followups";
import type { FollowUp } from "@/features/followups";
import type { AgendaItem, Analytics, Dashboard } from "@/shared/types/generated/analytics";
import { ChromeProvider } from "@/shared/lib/chrome";

const TODAY = localIso();

function shift(days: number): string {
  const date = new Date(`${TODAY}T00:00:00`);
  date.setDate(date.getDate() + days);
  return localIso(date);
}

const LATE: AgendaItem = {
  kind: "follow_up",
  id: "relance-1",
  application_id: "candidature-97",
  reference_number: 97,
  job_title: "Ingénieur systèmes & réseaux",
  company_name: "Groupe Astréa",
  status: "RELANCEE",
  date: shift(-5),
  detail: "Email",
  location: null,
};

const INTERVIEW: AgendaItem = {
  kind: "interview",
  id: "entretien-1",
  application_id: "candidature-133",
  reference_number: 133,
  job_title: "Technicien support N2",
  company_name: "Vallis Conseil",
  status: "ENTRETIEN",
  date: `${TODAY}T14:30:00`,
  detail: "Visio",
  location: null,
};

const WEEK: AgendaItem = {
  ...LATE,
  id: "relance-2",
  application_id: "candidature-142",
  reference_number: 142,
  company_name: "Novéa Services",
  job_title: "Chargé d'exploitation",
  date: shift(2),
};

const METRICS = {
  applications: 6,
  interviews: 1,
  responses: 2,
  rejected: 1,
  pending: 3,
  followed_up: 1,
  response_rate: 33,
  interview_rate: 17,
};

const DASHBOARD: Dashboard = {
  metrics: METRICS,
  performance: { average_response_days: 9, applications_per_week: 2, upcoming_interviews: 1, overdue_follow_ups: 1 },
  upcoming_items: [],
  pipeline: [],
  activity: [],
  recent: [],
};

const ANALYTICS: Analytics = {
  metrics: METRICS,
  performance: DASHBOARD.performance,
  activity: [],
  funnel: [],
  to_follow_up: [
    {
      id: "candidature-118",
      job_title: "Développeur Fullstack",
      company_name: "Linaïa",
      sent_date: shift(-41),
      days: 41,
      reference_number: 118,
    },
  ],
};

function followUp(item: AgendaItem): FollowUp {
  return {
    id: item.id,
    application_id: item.application_id,
    application_job_title: item.job_title,
    company_name: item.company_name,
    follow_up_date: item.date,
    type: "Email",
    notes: "Rappeler la référence de l'offre",
    done_at: null,
    created_at: `${item.date}T00:00:00Z`,
  };
}

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } });
  return (
    <QueryClientProvider client={client}>
      <MemoryRouter>
        <ChromeProvider>{children}</ChromeProvider>
      </MemoryRouter>
    </QueryClientProvider>
  );
}

beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(analyticsService, "agenda").mockResolvedValue([LATE, INTERVIEW, WEEK]);
  vi.spyOn(analyticsService, "dashboard").mockResolvedValue(DASHBOARD);
  vi.spyOn(analyticsService, "load").mockResolvedValue(ANALYTICS);
  vi.spyOn(applicationService, "breakdown").mockResolvedValue({ pending: 3, followed_up: 1, interview: 1, rejected: 1 });
});

describe("écran Aujourd'hui", () => {
  it("répartit les échéances en retard, aujourd'hui et cette semaine", async () => {
    render(<TodayPage />, { wrapper });

    const late = await screen.findByRole("region", { name: "En retard, 1" });
    expect(within(late).getByText("CAN-097")).toBeInTheDocument();
    expect(within(late).getByText("5 j de retard")).toBeInTheDocument();

    const today = screen.getByRole("region", { name: "Aujourd'hui, 1" });
    expect(within(today).getByText("Entretien — Technicien support N2")).toBeInTheDocument();
    expect(within(today).getByText("14:30")).toBeInTheDocument();

    const week = screen.getByRole("region", { name: "Cette semaine, 1" });
    expect(within(week).getByText("Relance — Novéa Services")).toBeInTheDocument();

    expect(screen.getByText("1 en retard · 1 aujourd'hui · 1 cette semaine")).toBeInTheDocument();
  });

  it("marque une relance faite", async () => {
    const setDone = vi.spyOn(followUpService, "setDone").mockResolvedValue({ ...followUp(LATE), done_at: TODAY });
    render(<TodayPage />, { wrapper });

    const late = await screen.findByRole("region", { name: "En retard, 1" });
    await userEvent.click(within(late).getByRole("button", { name: "Faire" }));

    await waitFor(() => expect(setDone).toHaveBeenCalledWith("relance-1", true));
  });

  it("reporte une relance en conservant ses notes", async () => {
    vi.spyOn(followUpService, "listBetween").mockResolvedValue([followUp(LATE)]);
    const update = vi.spyOn(followUpService, "update").mockResolvedValue({ ...followUp(LATE), follow_up_date: shift(3) });
    render(<TodayPage />, { wrapper });

    await screen.findByRole("region", { name: "En retard, 1" });
    await userEvent.keyboard("r");

    const dialog = await screen.findByRole("dialog");
    await userEvent.click(within(dialog).getByRole("button", { name: /Enregistrer|Programmer/ }));

    await waitFor(() =>
      expect(update).toHaveBeenCalledWith(
        "relance-1",
        expect.objectContaining({ notes: "Rappeler la référence de l'offre" }),
      ),
    );
  });

  it("montre la colonne Situation avec les candidatures sans réponse", async () => {
    render(<TodayPage />, { wrapper });

    const aside = await screen.findByRole("complementary", { name: "Situation" });
    await waitFor(() => expect(within(aside).getByText("Linaïa")).toBeInTheDocument());
    expect(within(aside).getByText("41 j")).toBeInTheDocument();
    expect(within(aside).getByText("envoyées")).toBeInTheDocument();
  });

  it("invite à commencer quand la base est vide", async () => {
    vi.spyOn(analyticsService, "agenda").mockResolvedValue([]);
    vi.spyOn(applicationService, "breakdown").mockResolvedValue({ pending: 0, followed_up: 0, interview: 0, rejected: 0 });
    render(<TodayPage />, { wrapper });

    expect(await screen.findByText("Votre suivi commence ici")).toBeInTheDocument();
  });

  it("annonce une bonne nouvelle quand rien n'est dû", async () => {
    vi.spyOn(analyticsService, "agenda").mockResolvedValue([]);
    render(<TodayPage />, { wrapper });

    expect(await screen.findByText("Rien à faire aujourd'hui")).toBeInTheDocument();
  });
});
