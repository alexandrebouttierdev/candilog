import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { ApplicationDetail } from "../ApplicationDetail";
import type { Application } from "../../../services/applicationService";
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

describe("ApplicationDetail — lien de l'offre", () => {
  it("ouvre l'offre via openExternal plutôt qu'un <a target=\"_blank\">", () => {
    render(
      <ApplicationDetail
        application={cand("https://exemple.test/offre/42")}
        onClose={vi.fn()}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        onStatusChange={vi.fn()}
      />,
    );

    const bouton = screen.getByRole("button", { name: "Ouvrir l'offre" });
    expect(bouton.tagName).toBe("BUTTON");
    fireEvent.click(bouton);

    expect(openExternal).toHaveBeenCalledWith("https://exemple.test/offre/42");
  });

  it("annonce l'absence de lien sans bouton", () => {
    render(
      <ApplicationDetail
        application={cand(null)}
        onClose={vi.fn()}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        onStatusChange={vi.fn()}
      />,
    );

    expect(screen.getByText("Aucun lien")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Ouvrir l'offre" })).not.toBeInTheDocument();
  });
});
