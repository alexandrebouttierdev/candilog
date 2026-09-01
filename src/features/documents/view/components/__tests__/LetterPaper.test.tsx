import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { profileService } from "@/features/profile/services/profileService";
import type { ProfilePayload } from "@/shared/types/generated/profile";
import { LetterPaper } from "../LetterPaper";

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

function payload(): ProfilePayload {
  return {
    profile: {
      identity: {
        first_name: "Alex",
        name: "Exemple",
        email: "alex@exemple.fr",
        phone: "06 12 34 56 78",
        address: "14 rue Saint-Melaine",
        city: "Rennes",
        title: "Développeur Rust",
        resume: null,
        linkedin: null,
        github: null,
        website: null,
      },
      experiences: [],
      skills: [],
      education: [],
      languages: [],
      projects: [],
      certifications: [],
    },
    completion: 100,
    incomplete_sections: [],
    updated_at: "2026-08-31T00:00:00Z",
  };
}

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("feuille de la lettre", () => {
  it("compose le template : identité, destinataire, intitulé et pièce jointe", async () => {
    vi.spyOn(profileService, "load").mockResolvedValue(payload());

    render(
      <LetterPaper
        fields={{
          company: "Astek",
          job_title: "Développeur",
          recipient: "Service recrutement",
          recipient_address: "12 rue de la Monnaie, 35000 Rennes",
          job_reference: "FS-2026-114",
        }}
      >
        <div>Corps</div>
      </LetterPaper>,
      { wrapper },
    );

    expect(await screen.findByRole("heading", { name: /Alex/ })).toBeInTheDocument();
    expect(screen.getByText("Développeur Rust")).toBeInTheDocument();
    expect(screen.getByText("14 rue Saint-Melaine")).toBeInTheDocument();
    expect(screen.getByText("06 12 34 56 78")).toBeInTheDocument();
    expect(screen.getByText("Astek")).toBeInTheDocument();
    expect(screen.getByText("Service recrutement")).toBeInTheDocument();
    expect(screen.getByText("Candidature au poste de Développeur")).toBeInTheDocument();
    expect(screen.getByText("Référence de l'offre : FS-2026-114")).toBeInTheDocument();
    expect(screen.getByText(/curriculum vitæ/)).toBeInTheDocument();
    expect(screen.queryByText(/Objet :/)).not.toBeInTheDocument();
  });

  it("omet les blocs facultatifs vides en lecture", async () => {
    vi.spyOn(profileService, "load").mockResolvedValue(payload());

    render(
      <LetterPaper
        fields={{
          company: "Astek",
          job_title: "Développeur",
          recipient: null,
          recipient_address: null,
          job_reference: null,
        }}
      >
        <div>Corps</div>
      </LetterPaper>,
      { wrapper },
    );

    expect(await screen.findByText("Astek")).toBeInTheDocument();
    expect(screen.queryByText("Interlocuteur")).not.toBeInTheDocument();
    expect(screen.queryByText(/Référence de l'offre/)).not.toBeInTheDocument();
  });

  it("reste imprimable sans profil renseigné", async () => {
    vi.spyOn(profileService, "load").mockRejectedValue(new Error("profil indisponible"));

    render(
      <LetterPaper
        fields={{
          company: null,
          job_title: null,
          recipient: null,
          recipient_address: null,
          job_reference: null,
        }}
      >
        <div>Corps</div>
      </LetterPaper>,
      { wrapper },
    );

    expect(await screen.findAllByText("Candilog")).toHaveLength(2);
    expect(screen.getByText(/curriculum vitæ/)).toBeInTheDocument();
  });
});
