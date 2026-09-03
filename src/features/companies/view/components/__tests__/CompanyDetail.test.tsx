import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { CompanyDetail } from "../CompanyDetail";
import type { Company } from "../../../services/companyService";
import { openExternal } from "@/shared/services/external-link";

vi.mock("@/shared/services/external-link", () => ({ openExternal: vi.fn() }));

function ent(overrides: Partial<Company> = {}): Company {
  return {
    id: "e1",
    name: "Nova Digital",
    sector_id: null,
    sector_name: null,
    company_type_id: "IT_SERVICES_COMPANY",
    company_type_name: "ESN / Société de services numériques",
    company_size: "PME",
    website: null,
    city: null,
    address: null,
    notes: null,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

const METRICS = { total: 0, interview: 0, pending: 0 };

describe("CompanyDetail — lien du site web", () => {
  it("ouvre le site via openExternal depuis l'action d'en-tête", () => {
    render(
      <CompanyDetail
        company={ent({ website: "https://nova-digital.test" })}
        applications={[]}
        metrics={METRICS}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        onOuvrirApplication={vi.fn()}
        onToutVoir={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Site web" }));

    expect(openExternal).toHaveBeenCalledWith("https://nova-digital.test");
  });

  it("ouvre le site via openExternal depuis la rangée d'informations", () => {
    render(
      <CompanyDetail
        company={ent({ website: "https://nova-digital.test" })}
        applications={[]}
        metrics={METRICS}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        onOuvrirApplication={vi.fn()}
        onToutVoir={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "https://nova-digital.test" }));

    expect(openExternal).toHaveBeenCalledWith("https://nova-digital.test");
  });
});
