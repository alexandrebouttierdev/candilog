import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Identity } from "@/shared/types/generated/profile";
import { LetterPaper } from "../LetterPaper";

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

function identity(): Identity {
  return {
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
  };
}

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("feuille de la lettre", () => {
  it("compose le template : identité, destinataire, intitulé et pièce jointe", () => {
    render(
      <LetterPaper
        identity={identity()}
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

    expect(screen.getByRole("heading", { name: /Alex/ })).toBeInTheDocument();
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

  it("omet les blocs facultatifs vides en lecture", () => {
    render(
      <LetterPaper
        identity={identity()}
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

    expect(screen.getByText("Astek")).toBeInTheDocument();
    expect(screen.queryByText("Interlocuteur")).not.toBeInTheDocument();
    expect(screen.queryByText(/Référence de l'offre/)).not.toBeInTheDocument();
  });

  it("reste imprimable sans profil renseigné", () => {
    render(
      <LetterPaper
        identity={null}
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

    expect(screen.getAllByText("Candilog")).toHaveLength(2);
    expect(screen.getByText(/curriculum vitæ/)).toBeInTheDocument();
  });

  it("délègue l'enregistrement de l'identité au parent via onSaveIdentity", async () => {
    const onSaveIdentity = vi.fn().mockResolvedValue(undefined);

    render(
      <LetterPaper
        editable
        identity={identity()}
        onSaveIdentity={onSaveIdentity}
        fields={{
          company: null,
          job_title: null,
          recipient: null,
          recipient_address: null,
          job_reference: null,
        }}
        onChange={() => {}}
      >
        <div>Corps</div>
      </LetterPaper>,
      { wrapper },
    );

    const champ = screen.getByRole("textbox", { name: "Téléphone" });
    champ.textContent = "01 02 03 04 05";
    fireEvent.input(champ);
    fireEvent.blur(champ);

    await waitFor(() => expect(onSaveIdentity).toHaveBeenCalledTimes(1));
    const saved = onSaveIdentity.mock.calls[0]?.[0] as Identity;
    expect(saved.phone).toBe("01 02 03 04 05");
  });
});
