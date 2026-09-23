import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { applicationService } from "../../../services/applicationService";
import { ApplicationInspector } from "../ApplicationInspector";
import type { Application } from "@/shared/types/generated/applications";
import { openExternal } from "@/shared/services/external-link";

vi.mock("@/shared/services/external-link", () => ({ openExternal: vi.fn() }));

function cand(job_url: string | null): Application {
  return {
    id: "a1",
    job_title: "Développeur",
    company_id: "e1",
    company_name: "Nova Digital",
    company_size: "PME",
    contact_id: null,
    application_type: "OFFRE",
    channel: "OFFER",
    reference_number: 142,
    next_follow_up_date: null,
    next_interview_at: null,
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
    effective_company_type_id: "IT_SERVICES_COMPANY",
    effective_company_type_name: "ESN / Société de services numériques",
    status: "EN_ATTENTE",
    sent_date: "2026-08-20",
    job_url,
    notes: null,
    created_at: "2026-08-20T00:00:00Z",
    updated_at: "2026-08-20T00:00:00Z",
  };
}

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

function afficher(application: Application) {
  return render(
    <ApplicationInspector
      application={application}
      onClose={vi.fn()}
      onEdit={vi.fn()}
      onMenu={vi.fn()}
      onStatusMenu={vi.fn()}
      onScheduleFollowUp={vi.fn()}
    />,
    { wrapper },
  );
}

describe("inspecteur d'une candidature — lien de l'offre", () => {
  it("ouvre l'offre via openExternal plutôt qu'un <a target=\"_blank\">", () => {
    vi.spyOn(applicationService, "statusHistory").mockResolvedValue([]);
    afficher(cand("https://exemple.test/offre/42"));

    const bouton = screen.getByRole("button", { name: "Ouvrir l'offre" });
    expect(bouton.tagName).toBe("BUTTON");
    fireEvent.click(bouton);

    expect(openExternal).toHaveBeenCalledWith("https://exemple.test/offre/42");
  });

  it("annonce l'absence de lien sans bouton", () => {
    vi.spyOn(applicationService, "statusHistory").mockResolvedValue([]);
    afficher(cand(null));

    expect(screen.getByText("Aucun lien")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Ouvrir l'offre" })).not.toBeInTheDocument();
  });
});

describe("inspecteur d'une candidature — contenu", () => {
  it("dit qu'une valeur vient de l'entreprise", () => {
    // Une ville affichée sans mention laisserait croire qu'elle a été saisie pour cette
    // candidature, alors qu'elle suivra l'entreprise si celle-ci change.
    vi.spyOn(applicationService, "statusHistory").mockResolvedValue([]);
    afficher(cand(null));

    expect(screen.getByRole("complementary", { name: "Fiche CAN-142" })).toBeInTheDocument();
    // Ville et type viennent tous deux de l'entreprise dans ce jeu de données.
    expect(screen.getAllByText("— héritée", { exact: false })).toHaveLength(2);
  });

  it("affiche l'historique des statuts, le plus récent d'abord", async () => {
    vi.spyOn(applicationService, "statusHistory").mockResolvedValue([
      { status: "RELANCEE", changed_at: "2026-09-10T10:00:00Z" },
      { status: "EN_ATTENTE", changed_at: "2026-09-02T10:00:00Z" },
    ]);
    afficher(cand(null));

    expect(await screen.findByText("Relancée")).toBeInTheDocument();
    expect(screen.getByText("10-09")).toBeInTheDocument();
  });
});
