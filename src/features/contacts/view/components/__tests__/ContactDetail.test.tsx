import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { ContactDetail } from "../ContactDetail";
import type { Contact } from "../../../services/contactService";
import { openExternal } from "@/shared/services/external-link";

vi.mock("@/shared/services/external-link", () => ({ openExternal: vi.fn() }));

function ct(overrides: Partial<Contact> = {}): Contact {
  return {
    id: "c1",
    company_id: null,
    company_name: null,
    first_name: "Camille",
    name: "Rivet",
    job_title: null,
    tracking_role: "Recruteur",
    email: null,
    phone: null,
    linkedin: null,
    notes: null,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

describe("ContactDetail — liens externes", () => {
  it("ouvre LinkedIn via openExternal depuis l'action d'en-tête", () => {
    render(
      <ContactDetail
        contact={ct({ linkedin: "https://www.linkedin.com/in/camille-rivet" })}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "LinkedIn" }));

    expect(openExternal).toHaveBeenCalledWith("https://www.linkedin.com/in/camille-rivet");
  });

  it("ouvre LinkedIn via openExternal depuis la rangée d'informations", () => {
    render(
      <ContactDetail
        contact={ct({ linkedin: "https://www.linkedin.com/in/camille-rivet" })}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "https://www.linkedin.com/in/camille-rivet" }));

    expect(openExternal).toHaveBeenCalledWith("https://www.linkedin.com/in/camille-rivet");
  });

  it("garde un lien natif pour l'e-mail (mailto n'a pas besoin d'openExternal)", () => {
    render(<ContactDetail contact={ct({ email: "camille@exemple.test" })} onEdit={vi.fn()} onDelete={vi.fn()} />);

    const lien = screen.getByRole("link", { name: "camille@exemple.test" });
    expect(lien).toHaveAttribute("href", "mailto:camille@exemple.test");
  });
});
