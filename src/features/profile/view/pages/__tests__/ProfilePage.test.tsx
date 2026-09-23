import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { ProfilePage } from "../ProfilePage";
import { profileService } from "../../../services/profileService";
import type { ProfilePayload } from "@/shared/types/generated/profile";
import { useUiStore } from "@/shared/lib/ui-store";

/** PNG minimal encodé, suffisant pour un `src` d'image en test. */
const PHOTO =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

function payload(photo: string | null = null): ProfilePayload {
  return {
    profile: {
      photo,
      identity: {
        first_name: "Camille",
        name: "Rivet",
        email: "camille@example.fr",
        phone: null,
        address: null,
        city: null,
        title: null,
        resume: null,
        birth_date: null,
        age: null,
        availability: null,
        desired_contracts: null,
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
    interests: [],
    },
    completion: 14,
    incomplete_sections: ["une expérience"],
    updated_at: "2026-09-01T10:00:00Z",
  };
}

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return (
    <QueryClientProvider client={client}>
      <MemoryRouter>{children}</MemoryRouter>
    </QueryClientProvider>
  );
}

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ toasts: [] });
  vi.spyOn(profileService, "load").mockResolvedValue(payload());
  vi.spyOn(profileService, "photo").mockResolvedValue(null);
});

describe("écran Profil — photo", () => {
  it("propose d'ajouter une photo quand le profil n'en a pas", async () => {
    render(<ProfilePage />, { wrapper });

    expect(await screen.findByRole("button", { name: "Ajouter une photo" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Supprimer la photo" })).not.toBeInTheDocument();
    // Le profil reste parfaitement utilisable sans photo.
    expect(screen.getByText("Camille Rivet")).toBeInTheDocument();
  });

  it("place les actions de photo dans la section Identité", async () => {
    // La photo se change là où on la voit : à côté du nom, dans la section Identité.
    render(<ProfilePage />, { wrapper });

    const panneau = await screen.findByRole("tabpanel");
    expect(within(panneau).getByRole("button", { name: "Ajouter une photo" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: /Identité/ })).toHaveAttribute("aria-selected", "true");
  });

  it("affiche la photo enregistrée et permet de la remplacer", async () => {
    vi.spyOn(profileService, "load").mockResolvedValue(payload("photo-1.png"));
    vi.spyOn(profileService, "photo").mockResolvedValue(PHOTO);
    const choisir = vi.spyOn(profileService, "setPhoto").mockResolvedValue(payload("photo-2.png"));

    render(<ProfilePage />, { wrapper });

    const preview = await screen.findByRole("img", { name: "Photo de profil" });
    expect(preview).toHaveAttribute("src", PHOTO);
    expect(preview.className).toContain("rounded-field");
    expect(preview.className).not.toContain("rounded-full");

    await userEvent.click(screen.getByRole("button", { name: "Remplacer la photo" }));

    await waitFor(() => expect(choisir).toHaveBeenCalled());
  });

  it("supprime la photo à la demande", async () => {
    vi.spyOn(profileService, "load").mockResolvedValue(payload("photo-1.png"));
    vi.spyOn(profileService, "photo").mockResolvedValue(PHOTO);
    const supprimer = vi.spyOn(profileService, "removePhoto").mockResolvedValue(payload());

    render(<ProfilePage />, { wrapper });

    await userEvent.click(await screen.findByRole("button", { name: "Supprimer la photo" }));

    await waitFor(() => expect(supprimer).toHaveBeenCalled());
  });

  it("n'appelle rien quand le sélecteur natif est annulé", async () => {
    const choisir = vi.spyOn(profileService, "setPhoto").mockResolvedValue(null);

    render(<ProfilePage />, { wrapper });
    await userEvent.click(await screen.findByRole("button", { name: "Ajouter une photo" }));

    await waitFor(() => expect(choisir).toHaveBeenCalled());
    expect(useUiStore.getState().toasts).toHaveLength(0);
  });
});

describe("écran Profil — sections", () => {
  it("n'expose plus le bouton Modifier le profil", async () => {
    render(<ProfilePage />, { wrapper });
    await screen.findByRole("tablist", { name: "Sections du profil" });
    expect(screen.queryByRole("button", { name: "Modifier le profil" })).not.toBeInTheDocument();
  });

  it("expose chaque section comme un véritable onglet, un seul panneau à la fois", async () => {
    render(<ProfilePage />, { wrapper });

    const contact = await screen.findByRole("tab", { name: /Contact/ });
    expect(contact).toHaveAttribute("aria-selected", "false");
    await userEvent.click(contact);

    expect(contact).toHaveAttribute("aria-selected", "true");
    expect(screen.getAllByRole("tabpanel")).toHaveLength(1);
    expect(screen.getByRole("heading", { name: "Contact" })).toBeInTheDocument();
    expect(screen.getByText("camille@example.fr")).toBeInTheDocument();
  });

  it("indique l'état et le décompte de chaque section", async () => {
    render(<ProfilePage />, { wrapper });

    // Contact : seul le courriel est renseigné sur quatre champs.
    expect(await screen.findByRole("tab", { name: /Contact.*1\/4/ })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: /Expériences.*0/ })).toBeInTheDocument();
  });

  it("ouvre la modale Identité depuis la section dédiée", async () => {
    render(<ProfilePage />, { wrapper });
    await userEvent.click(await screen.findByRole("tab", { name: /Identité/ }));
    await userEvent.click(screen.getByRole("button", { name: "Modifier" }));
    expect(await screen.findByRole("dialog", { name: "Identité" })).toBeInTheDocument();
  });

  it("retire une entrée après confirmation", async () => {
    const avecExperience = payload();
    avecExperience.profile.experiences = [
      { title: "Technicien support", company: "Vallis", location: null, start_date: "2018-01", end_date: "2021-01", current: false, description: null },
    ];
    vi.spyOn(profileService, "load").mockResolvedValue(avecExperience);
    const enregistrer = vi.spyOn(profileService, "save").mockResolvedValue(payload());

    render(<ProfilePage />, { wrapper });
    await userEvent.click(await screen.findByRole("tab", { name: /Expériences/ }));
    await userEvent.click(screen.getByRole("button", { name: "Retirer Technicien support" }));
    const dialogue = screen.getByRole("alertdialog", { name: "Retirer cette entrée ?" });
    await userEvent.click(within(dialogue).getByRole("button", { name: "Retirer" }));

    await waitFor(() =>
      expect(enregistrer).toHaveBeenCalledWith(expect.objectContaining({ experiences: [] })),
    );
  });
});

describe("écran Profil — import de CV", () => {
  it("propose l'import sous les sections et ouvre la revue", async () => {
    render(<ProfilePage />, { wrapper });

    await screen.findByRole("tablist", { name: "Sections du profil" });
    const bouton = within(screen.getByRole("navigation", { name: "Profil" })).getByRole("button", {
      name: "Importer un CV",
    });

    await userEvent.click(bouton);

    expect(await screen.findByRole("dialog", { name: "Importer depuis un CV" })).toBeInTheDocument();
  });
});

describe("écran Profil — réinitialisation", () => {
  it("place la réinitialisation sous les sections, après l'import", async () => {
    render(<ProfilePage />, { wrapper });

    const reset = await screen.findByRole("button", { name: "Réinitialiser mon profil" });
    const importer = screen.getByRole("button", { name: "Importer un CV" });

    expect(importer.compareDocumentPosition(reset) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  it("ne réinitialise rien tant que la confirmation n'est pas donnée", async () => {
    const reset = vi.spyOn(profileService, "reset");

    render(<ProfilePage />, { wrapper });
    await userEvent.click(await screen.findByRole("button", { name: "Réinitialiser mon profil" }));

    const dialog = screen.getByRole("alertdialog", { name: "Réinitialiser le profil ?" });
    expect(
      within(dialog).getByText(
        "Toutes les informations de votre profil seront supprimées, photo comprise.",
      ),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByText(
        "Vos candidatures, entreprises, contacts, entretiens et autres données ne sont pas modifiés.",
      ),
    ).toBeInTheDocument();

    await userEvent.click(within(dialog).getByRole("button", { name: "Annuler" }));

    await waitFor(() =>
      expect(
        screen.queryByRole("alertdialog", { name: "Réinitialiser le profil ?" }),
      ).not.toBeInTheDocument(),
    );
    expect(reset).not.toHaveBeenCalled();
  });

  it("réinitialise le profil une fois confirmé", async () => {
    const reset = vi.spyOn(profileService, "reset").mockResolvedValue(payload());

    render(<ProfilePage />, { wrapper });
    await userEvent.click(await screen.findByRole("button", { name: "Réinitialiser mon profil" }));

    const dialog = screen.getByRole("alertdialog", { name: "Réinitialiser le profil ?" });
    await userEvent.click(within(dialog).getByRole("button", { name: "Réinitialiser" }));

    await waitFor(() => expect(reset).toHaveBeenCalledTimes(1));
    expect(useUiStore.getState().toasts.at(-1)).toMatchObject({
      tone: "success",
      title: "Profil réinitialisé",
    });
  });
});
