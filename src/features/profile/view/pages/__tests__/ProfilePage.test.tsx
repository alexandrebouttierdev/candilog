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

  it("place les actions de photo dans l'en-tête, avant les onglets", async () => {
    // La photo se change là où on la voit : sur la pastille d'identité, et non dans une
    // carte perdue en bas de la colonne de droite.
    render(<ProfilePage />, { wrapper });

    const ajouter = await screen.findByRole("button", { name: "Ajouter une photo" });
    const onglets = screen.getByRole("tablist", { name: "Sections du profil" });

    expect(ajouter.compareDocumentPosition(onglets) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(screen.queryByText("Photo", { exact: true })).not.toBeInTheDocument();
  });

  it("affiche la photo enregistrée et permet de la remplacer", async () => {
    vi.spyOn(profileService, "load").mockResolvedValue(payload("photo-1.png"));
    vi.spyOn(profileService, "photo").mockResolvedValue(PHOTO);
    const choisir = vi.spyOn(profileService, "setPhoto").mockResolvedValue(payload("photo-2.png"));

    render(<ProfilePage />, { wrapper });

    const apercu = await screen.findByRole("img", { name: "Photo de profil" });
    expect(apercu).toHaveAttribute("src", PHOTO);

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

describe("écran Profil — import de CV", () => {
  it("propose l'import dans le bandeau, après la progression du profil", async () => {
    render(<ProfilePage />, { wrapper });

    const onglets = await screen.findByRole("tablist", { name: "Sections du profil" });
    const progression = screen.getByRole("progressbar", { name: "Profil complété" });
    const bouton = screen.getByRole("button", { name: "Analyser un CV" });

    // Le bloc suit la barre de progression et précède les onglets : il est donc en bout de
    // bandeau, et non plus en bas de la colonne de droite.
    expect(progression.compareDocumentPosition(bouton) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(bouton.compareDocumentPosition(onglets) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();

    await userEvent.click(bouton);

    expect(await screen.findByRole("dialog", { name: "Importer depuis un CV" })).toBeInTheDocument();
  });
});

describe("écran Profil — réinitialisation", () => {
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
