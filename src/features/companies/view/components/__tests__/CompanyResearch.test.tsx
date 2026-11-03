import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { aiService, useAiOperationStore, useAiRequiredStore } from "@/features/ai";
import { referentialService } from "@/features/referentials";
import { settingsService } from "@/features/settings";
import { REFERENTIELS_DE_TEST, createTestQueryClient } from "@/shared/lib/test-utils";
import { AppError } from "@/shared/types/app-error";
import type {
  AiExecution,
  CompanyCandidate,
  CompanyResearchResult,
  CompanySearchResult,
} from "@/shared/types/generated/ai";
import type { LlmForm, Settings } from "@/shared/types/generated/settings";
import { CompanyFormModal } from "../CompanyFormModal";

vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(() => Promise.resolve(() => undefined)) }));

const IA_LOCALE: LlmForm = {
  provider: "candilog_local",
  api_key_configured: false,
  endpoint: null,
  model: "",
  temperature: 0.7,
  mode: "auto",
};

const reglages: Settings = {
  llm: IA_LOCALE,
  llm_presets: {},
  ai_routes: {},
  remote_send_consents: [],
  theme: "system",
  language: "fr",
};

function etablissement(siret: string, city: string, head_office: boolean): CompanyCandidate {
  return {
    siret,
    siren: siret.slice(0, 9),
    name: "Orange",
    trade_name: null,
    city,
    postal_code: null,
    address: `1 Rue de la Gare ${city}`,
    head_office,
    activity_code: "61.10Z",
    activity_label: "Information et communication",
    legal_category: "5599",
    size_category: "GE",
    workforce_range: "53",
    open_establishments: 1200,
    source_url: `https://annuaire-entreprises.data.gouv.fr/etablissement/${siret}`,
  };
}

const execution = <T,>(output: T): AiExecution<T> => ({ output, elapsed_ms: 10, tokens_used: null });

function resultat(): CompanyResearchResult {
  const champ = (value: string | null) => ({
    value,
    origin: value === null ? null : ("registry" as const),
    source_url: null,
    note: null,
  });
  return {
    fields: {
      name: champ("Orange"),
      sector_id: { ...champ("5ec70000-0000-4000-8000-00000000000d"), origin: "ai" },
      company_type_id: champ(null),
      company_size: champ("LARGE"),
      website: { ...champ("https://www.orange.fr"), origin: "wikidata" },
      city: champ("Rennes"),
      address: champ("1 Rue de la Gare Rennes"),
    },
    sources: [{ label: "Annuaire des entreprises", url: "https://annuaire-entreprises.data.gouv.fr/etablissement/38012986612345" }],
    warnings: [],
    ai_used: true,
  };
}

function wrapper({ children }: { children: ReactNode }) {
  return (
    <QueryClientProvider client={createTestQueryClient()}>
      <MemoryRouter>{children}</MemoryRouter>
    </QueryClientProvider>
  );
}

function ouvrir(onSubmit = vi.fn().mockResolvedValue(undefined)) {
  render(<CompanyFormModal open company={null} busy={false} onClose={vi.fn()} onSubmit={onSubmit} />, { wrapper });
  return onSubmit;
}

beforeEach(() => {
  vi.restoreAllMocks();
  useAiOperationStore.setState({ active: null });
  useAiRequiredStore.setState({ llm: IA_LOCALE, open: false });
  vi.spyOn(referentialService, "load").mockResolvedValue(REFERENTIELS_DE_TEST);
  vi.spyOn(settingsService, "load").mockResolvedValue(reglages);
});

describe("bouton « Compléter avec l'IA »", () => {
  it("n'apparaît qu'une fois le nom saisi", async () => {
    ouvrir();
    expect(screen.queryByRole("button", { name: "Compléter avec l'IA" })).not.toBeInTheDocument();
    await userEvent.type(screen.getByPlaceholderText("Nom de l'entreprise"), "Orange");
    expect(screen.getByRole("button", { name: "Compléter avec l'IA" })).toBeInTheDocument();
  });

  it("reste caché tant qu'aucun fournisseur IA n'est prêt", async () => {
    useAiRequiredStore.setState({ llm: { ...IA_LOCALE, provider: "openai", model: "" } });
    ouvrir();
    await userEvent.type(screen.getByPlaceholderText("Nom de l'entreprise"), "Orange");
    expect(screen.queryByRole("button", { name: "Compléter avec l'IA" })).not.toBeInTheDocument();
  });

  it("propose de configurer l'IA si les réglages ne sont pas encore connus", async () => {
    useAiRequiredStore.setState({ llm: null });
    const recherche = vi.spyOn(aiService, "searchCompanies");
    ouvrir();
    await userEvent.type(screen.getByPlaceholderText("Nom de l'entreprise"), "Orange");
    await userEvent.click(screen.getByRole("button", { name: "Compléter avec l'IA" }));
    expect(useAiRequiredStore.getState().open).toBe(true);
    expect(recherche).not.toHaveBeenCalled();
  });
});

describe("recherche et vérification", () => {
  it("fait choisir l'établissement, puis ne remplit que les champs validés", async () => {
    const recherche = vi.spyOn(aiService, "searchCompanies").mockResolvedValue(
      execution<CompanySearchResult>({
        candidates: [
          etablissement("38012986600001", "Issy-les-Moulineaux", true),
          etablissement("38012986612345", "Rennes", false),
        ],
        confident: false,
        truncated: false,
      }),
    );
    const complement = vi.spyOn(aiService, "researchCompany").mockResolvedValue(execution(resultat()));
    const onSubmit = ouvrir();

    await userEvent.type(screen.getByPlaceholderText("Nom de l'entreprise"), "Orange");
    await userEvent.type(screen.getByPlaceholderText("Ville"), "Saint-Malo");
    await userEvent.click(screen.getByRole("button", { name: "Compléter avec l'IA" }));

    // La ville déjà saisie sert de précision de lieu.
    await waitFor(() => expect(recherche).toHaveBeenCalledOnce());
    expect(recherche.mock.calls[0]?.[0]).toMatchObject({ name: "Orange", location: "Saint-Malo" });

    const choix = await screen.findByRole("group", { name: "Établissements trouvés" });
    expect(within(choix).getByText("Siège social")).toBeInTheDocument();
    const continuer = screen.getByRole("button", { name: "Continuer avec cet établissement" });
    expect(continuer).toBeDisabled();
    await userEvent.click(within(choix).getByText("Rennes"));
    await userEvent.click(continuer);

    await waitFor(() => expect(complement).toHaveBeenCalledOnce());
    expect(complement.mock.calls[0]?.[0]).toMatchObject({ siret: "38012986612345", name: "Orange" });

    const verification = await screen.findByRole("dialog", { name: /Vérifier les informations trouvées/ });
    // Ville déjà saisie : proposée en remplacement, non cochée par défaut.
    const villes = within(verification).getAllByRole("checkbox", { name: "Remplacer" });
    expect(villes).toHaveLength(2); // nom et ville
    villes.forEach((caseACocher) => expect(caseACocher).not.toBeChecked());
    expect(within(verification).getByText(/Aucune information fiable trouvée — le champ reste tel quel/)).toBeInTheDocument();
    await userEvent.click(within(verification).getByRole("button", { name: "Remplir le formulaire" }));

    await waitFor(() => expect(screen.queryByRole("dialog", { name: /Vérifier/ })).not.toBeInTheDocument());
    expect(screen.getByPlaceholderText("Ville")).toHaveValue("Saint-Malo");
    expect(screen.getByPlaceholderText("Adresse")).toHaveValue("1 Rue de la Gare Rennes");
    expect(screen.getByPlaceholderText("https://…")).toHaveValue("https://www.orange.fr");

    await userEvent.click(screen.getByRole("button", { name: "Enregistrer" }));
    await waitFor(() => expect(onSubmit).toHaveBeenCalledOnce());
    expect(onSubmit.mock.calls[0]?.[0]).toMatchObject({
      name: "Orange",
      city: "Saint-Malo",
      company_size: "LARGE",
      sector_id: "5ec70000-0000-4000-8000-00000000000d",
    });
  });

  it("saute le choix quand un seul établissement porte exactement ce nom", async () => {
    vi.spyOn(aiService, "searchCompanies").mockResolvedValue(
      execution<CompanySearchResult>({
        candidates: [etablissement("38012986612345", "Rennes", false)],
        confident: true,
        truncated: false,
      }),
    );
    const complement = vi.spyOn(aiService, "researchCompany").mockResolvedValue(execution(resultat()));
    ouvrir();
    await userEvent.type(screen.getByPlaceholderText("Nom de l'entreprise"), "Orange");
    await userEvent.click(screen.getByRole("button", { name: "Compléter avec l'IA" }));

    await screen.findByRole("dialog", { name: /Vérifier les informations trouvées/ });
    expect(complement).toHaveBeenCalledOnce();
    expect(screen.queryByRole("group", { name: "Établissements trouvés" })).not.toBeInTheDocument();
  });

  it("propose de préciser ou de chercher sur le Web quand l'annuaire ne trouve rien", async () => {
    const recherche = vi
      .spyOn(aiService, "searchCompanies")
      .mockResolvedValue(execution<CompanySearchResult>({ candidates: [], confident: false, truncated: false }));
    const complement = vi.spyOn(aiService, "researchCompany").mockResolvedValue(
      execution({ ...resultat(), warnings: ["La recherche Web (SearXNG) est injoignable : il n'a pas été consulté."] }),
    );
    ouvrir();
    await userEvent.type(screen.getByPlaceholderText("Nom de l'entreprise"), "Inconnue");
    await userEvent.click(screen.getByRole("button", { name: "Compléter avec l'IA" }));

    expect(await screen.findByText("Aucun établissement trouvé")).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText("Ville ou code postal"), "Rennes");
    await userEvent.click(screen.getByRole("button", { name: "Relancer" }));
    await waitFor(() => expect(recherche).toHaveBeenCalledTimes(2));
    expect(recherche.mock.calls[1]?.[0]).toMatchObject({ location: "Rennes" });

    await userEvent.click(await screen.findByRole("button", { name: "Rechercher sur le Web seulement" }));
    await waitFor(() => expect(complement).toHaveBeenCalledOnce());
    expect(complement.mock.calls[0]?.[0]).toMatchObject({ siret: null });
    expect(await screen.findByText("Recherche partiellement réussie")).toBeInTheDocument();
  });

  it("explique une erreur réseau et permet de réessayer", async () => {
    vi.spyOn(aiService, "searchCompanies")
      .mockRejectedValueOnce(new AppError({ code: "HTTP_ERROR", message: "La connexion au service distant a échoué." }))
      .mockResolvedValueOnce(execution<CompanySearchResult>({ candidates: [], confident: false, truncated: false }));
    ouvrir();
    await userEvent.type(screen.getByPlaceholderText("Nom de l'entreprise"), "Orange");
    await userEvent.click(screen.getByRole("button", { name: "Compléter avec l'IA" }));

    expect(await screen.findByText("Connexion impossible")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Réessayer" }));
    expect(await screen.findByText("Aucun établissement trouvé")).toBeInTheDocument();
  });

  it("n'affiche pas une réponse IPC hors contrat", async () => {
    vi.spyOn(aiService, "searchCompanies").mockResolvedValue(
      execution<CompanySearchResult>({
        candidates: [{ ...etablissement("38012986612345", "Rennes", false), source_url: "javascript:alert(1)" }],
        confident: false,
        truncated: false,
      }),
    );
    ouvrir();
    await userEvent.type(screen.getByPlaceholderText("Nom de l'entreprise"), "Orange");
    await userEvent.click(screen.getByRole("button", { name: "Compléter avec l'IA" }));
    expect(await screen.findByText("Réponse inattendue")).toBeInTheDocument();
  });
});
