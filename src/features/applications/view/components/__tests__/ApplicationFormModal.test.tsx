import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { ApplicationFormModal } from "../ApplicationFormModal";
import { companyService } from "@/features/companies/services/companyService";
import type { Company } from "@/features/companies/services/companyService";
import { referentialService } from "@/features/referentials/services/referentialService";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

function entreprise(name: string): Company {
  return {
    id: `id-${name}`,
    name,
    sector_id: null,
    sector_name: null,
    company_type_id: null,
    company_type_name: null,
    company_size: "UNKNOWN",
    website: null,
    city: null,
    address: null,
    notes: null,
    created_at: "2026-08-20T00:00:00Z",
    updated_at: "2026-08-20T00:00:00Z",
  };
}

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ toasts: [] });
  vi.spyOn(referentialService, "load").mockResolvedValue({
    sectors: [],
    professional_domains: [],
    company_types: [],
    contract_types: [],
  });
  vi.spyOn(companyService, "listPage").mockResolvedValue({
    items: [],
    total: 0,
    page: 1,
    page_size: 4,
    total_pages: 0,
  });
});

/** Ouvre la modale de création rapide depuis une recherche restée sans résultat. */
async function ouvrirCreationRapide(user: ReturnType<typeof userEvent.setup>) {
  const picker = screen.getByLabelText(/^Entreprise/);
  await user.click(picker);
  await user.type(picker, "Nova Digital");
  await user.click(await screen.findByRole("button", { name: "Créer « Nova Digital »" }));
  return screen.getByRole("dialog", { name: "Nouvelle entreprise" });
}

describe("Nouvelle candidature — création rapide d'entreprise", () => {
  it("crée l'entreprise sans quitter le formulaire et la sélectionne", async () => {
    const user = userEvent.setup();
    const creer = vi.spyOn(companyService, "create").mockResolvedValue(entreprise("Nova Digital"));
    vi.spyOn(companyService, "get").mockResolvedValue(entreprise("Nova Digital"));

    render(
      <ApplicationFormModal
        open
        application={null}
        busy={false}
        onClose={vi.fn()}
        onSubmit={vi.fn()}
      />,
      { wrapper },
    );

    await user.type(screen.getByLabelText(/^Poste/), "Développeur Frontend");

    const creation = await ouvrirCreationRapide(user);
    // Le nom recherché est déjà là : l'utilisateur n'a pas à le ressaisir.
    expect(within(creation).getByLabelText(/^Nom/)).toHaveValue("Nova Digital");

    await user.click(within(creation).getByRole("button", { name: "Enregistrer" }));

    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Nouvelle entreprise" })).not.toBeInTheDocument(),
    );
    expect(creer).toHaveBeenCalledWith(expect.objectContaining({ name: "Nova Digital" }));

    // Le formulaire de candidature n'a pas bougé et porte la nouvelle entreprise.
    expect(screen.getByLabelText(/^Poste/)).toHaveValue("Développeur Frontend");
    await waitFor(() =>
      expect(screen.getByLabelText(/^Entreprise/)).toHaveValue("Nova Digital"),
    );
  });

  it("conserve les deux formulaires quand le backend refuse la création", async () => {
    const user = userEvent.setup();
    vi.spyOn(companyService, "create").mockRejectedValue(
      new AppError({ code: "VALIDATION_ERROR", message: "Cette entreprise existe déjà" }),
    );

    render(
      <ApplicationFormModal
        open
        application={null}
        busy={false}
        onClose={vi.fn()}
        onSubmit={vi.fn()}
      />,
      { wrapper },
    );

    await user.type(screen.getByLabelText(/^Poste/), "Développeur Frontend");
    const creation = await ouvrirCreationRapide(user);
    await user.click(within(creation).getByRole("button", { name: "Enregistrer" }));

    await waitFor(() =>
      expect(useUiStore.getState().toasts.at(-1)).toMatchObject({
        tone: "error",
        title: "Enregistrement impossible",
        detail: "Cette entreprise existe déjà",
      }),
    );
    expect(screen.getByRole("dialog", { name: "Nouvelle entreprise" })).toBeInTheDocument();
    expect(screen.getByLabelText(/^Poste/)).toHaveValue("Développeur Frontend");
  });

  it("abandonne proprement la création annulée", async () => {
    const user = userEvent.setup();
    const creer = vi.spyOn(companyService, "create");

    render(
      <ApplicationFormModal
        open
        application={null}
        busy={false}
        onClose={vi.fn()}
        onSubmit={vi.fn()}
      />,
      { wrapper },
    );

    const creation = await ouvrirCreationRapide(user);
    await user.click(within(creation).getByRole("button", { name: "Annuler" }));

    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Nouvelle entreprise" })).not.toBeInTheDocument(),
    );
    expect(creer).not.toHaveBeenCalled();
    expect(screen.getByRole("dialog", { name: "Nouvelle candidature" })).toBeInTheDocument();
  });
});

describe("Nouvelle candidature — garde-fous de la création rapide", () => {
  it("refuse une entreprise sans nom sans appeler le backend", async () => {
    const user = userEvent.setup();
    const creer = vi.spyOn(companyService, "create");

    render(
      <ApplicationFormModal
        open
        application={null}
        busy={false}
        onClose={vi.fn()}
        onSubmit={vi.fn()}
      />,
      { wrapper },
    );

    const creation = await ouvrirCreationRapide(user);
    await user.clear(within(creation).getByLabelText(/^Nom/));
    await user.click(within(creation).getByRole("button", { name: "Enregistrer" }));

    expect(await within(creation).findByText("Le nom de l'entreprise est obligatoire")).toBeInTheDocument();
    expect(creer).not.toHaveBeenCalled();
    expect(screen.getByRole("dialog", { name: "Nouvelle entreprise" })).toBeInTheDocument();
  });

  it("ne crée pas deux fois l'entreprise sur un double clic", async () => {
    const user = userEvent.setup();
    let resoudre: ((company: Company) => void) | undefined;
    const creer = vi.spyOn(companyService, "create").mockReturnValue(
      new Promise<Company>((resolve) => {
        resoudre = resolve;
      }),
    );

    render(
      <ApplicationFormModal
        open
        application={null}
        busy={false}
        onClose={vi.fn()}
        onSubmit={vi.fn()}
      />,
      { wrapper },
    );

    const creation = await ouvrirCreationRapide(user);
    const enregistrer = within(creation).getByRole("button", { name: "Enregistrer" });
    await user.click(enregistrer);

    // Le bouton se désactive pendant la mutation : le second clic n'atteint rien.
    await waitFor(() => expect(enregistrer).toBeDisabled());
    await user.click(enregistrer);

    resoudre?.(entreprise("Nova Digital"));
    await waitFor(() => expect(creer).toHaveBeenCalledTimes(1));
  });

  it("ne referme que la modale du dessus quand on appuie sur Échap", async () => {
    const user = userEvent.setup();
    const fermer = vi.fn();

    render(
      <ApplicationFormModal
        open
        application={null}
        busy={false}
        onClose={fermer}
        onSubmit={vi.fn()}
      />,
      { wrapper },
    );

    await ouvrirCreationRapide(user);
    await user.keyboard("{Escape}");

    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Nouvelle entreprise" })).not.toBeInTheDocument(),
    );
    expect(fermer).not.toHaveBeenCalled();
    expect(screen.getByRole("dialog", { name: "Nouvelle candidature" })).toBeInTheDocument();
  });
});
