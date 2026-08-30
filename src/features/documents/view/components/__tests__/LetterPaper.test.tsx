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
        phone: null,
        city: "Rennes",
        title: null,
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
  it("compose l'en-tête du PDF : identité, lieu et date, objet, signature", async () => {
    vi.spyOn(profileService, "load").mockResolvedValue(payload());

    render(
      <LetterPaper jobTitle="Développeur" company="Astek">
        <div>Corps</div>
      </LetterPaper>,
      { wrapper },
    );

    expect(await screen.findAllByText("Alex Exemple")).toHaveLength(2);
    expect(screen.getByText("Rennes")).toBeInTheDocument();
    expect(screen.getByText("alex@exemple.fr")).toBeInTheDocument();
    expect(screen.getByText(/^Rennes, le \d{1,2} \S+ \d{4}$/)).toBeInTheDocument();
    expect(
      screen.getByText("Objet : candidature au poste de Développeur — Astek"),
    ).toBeInTheDocument();
  });

  it("reste imprimable sans profil renseigné", async () => {
    vi.spyOn(profileService, "load").mockRejectedValue(new Error("profil indisponible"));

    render(
      <LetterPaper jobTitle={null} company={null}>
        <div>Corps</div>
      </LetterPaper>,
      { wrapper },
    );

    expect(await screen.findAllByText("Candilog")).toHaveLength(2);
    expect(screen.getByText("Objet : candidature")).toBeInTheDocument();
  });
});
