import { beforeEach, describe, expect, it, vi } from "vitest";
import { createMemoryRouter, RouterProvider, type RouteObject } from "react-router-dom";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryWrapper } from "@/shared/lib/test-utils";
import { useUiStore } from "@/shared/lib/ui-store";
import { managedOllamaService, settingsService } from "@/features/settings";
import { applicationService } from "@/features/applications";
import { profileService } from "@/features/profile";
import type { Settings } from "@/shared/types/generated/settings";
import { AppShell } from "../AppShell";

const REGLAGES: Settings = {
  llm: {
    provider: "openai",
    api_key_configured: true,
    endpoint: "https://api.openai.com",
    model: "gpt-4o",
    temperature: 0.7,
    mode: "auto",
  },
  llm_presets: {},
  ai_routes: {},
  theme: "system",
  language: "fr",
};

const ECRANS: RouteObject[] = [
  { index: true, element: <p>Écran Aujourd'hui</p> },
  { path: "applications", element: <><p>Écran Candidatures</p><input aria-label="Recherche" /></> },
  { path: "profile", element: <p>Écran Profil</p> },
];

function renderShell(initialEntries = ["/"]) {
  const router = createMemoryRouter([{ path: "/", element: <AppShell />, children: ECRANS }], {
    initialEntries,
  });
  render(
    <QueryWrapper>
      <RouterProvider router={router} />
    </QueryWrapper>,
  );
  return router;
}

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ settings: null, palette: false, toasts: [] });
  vi.spyOn(settingsService, "load").mockResolvedValue(REGLAGES);
  vi.spyOn(managedOllamaService, "status").mockResolvedValue({
    runtime_state: "not_installed",
    runtime_version: null,
    port: null,
    models_disk_bytes: 0,
    active_model: null,
    models: [],
    last_error: null,
  });
  vi.spyOn(managedOllamaService, "onProgress").mockResolvedValue(() => undefined);
});

describe("coque applicative", () => {
  it("offre un lien d'évitement vers le contenu et un seul main", () => {
    renderShell();
    expect(screen.getByRole("link", { name: "Aller au contenu" })).toHaveAttribute("href", "#contenu");
    expect(screen.getAllByRole("main")).toHaveLength(1);
    expect(screen.getByRole("main")).toHaveAttribute("id", "contenu");
  });

  it("expose les six destinations et marque la destination active", () => {
    renderShell(["/applications"]);
    const nav = screen.getByRole("navigation", { name: "Navigation principale" });
    for (const label of ["Aujourd'hui", "Candidatures", "Relations", "Documents", "Intelligence artificielle", "Profil"]) {
      expect(within(nav).getByRole("link", { name: new RegExp(label) })).toBeInTheDocument();
    }
    expect(within(nav).getByRole("link", { name: /Candidatures/ })).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("tab", { name: "Kanban" })).toBeInTheDocument();
  });

  it("affiche les décomptes connus : total des candidatures et complétude du profil", async () => {
    vi.spyOn(applicationService, "breakdown").mockResolvedValue({
      pending: 12,
      followed_up: 4,
      interview: 3,
      rejected: 5,
    });
    vi.spyOn(profileService, "load").mockResolvedValue({
      completion: 92,
    } as Awaited<ReturnType<typeof profileService.load>>);
    renderShell();
    const nav = screen.getByRole("navigation", { name: "Navigation principale" });
    expect(await within(nav).findByText("24")).toBeInTheDocument();
    // Profil incomplet : ambre, jamais rouge (`DECISIONS.md` B11).
    expect(await within(nav).findByText("92 %")).toHaveClass("text-st-a");
  });

  it("ouvre la palette avec Ctrl/Cmd+K et exécute une commande de navigation", async () => {
    const router = renderShell();
    fireEvent.keyDown(document, { key: "k", ctrlKey: true });
    const palette = await screen.findByRole("dialog", { name: "Palette de commandes" });
    await userEvent.type(within(palette).getByRole("combobox"), "profil");
    await userEvent.keyboard("{Enter}");
    expect(router.state.location.pathname).toBe("/profile");
    expect(screen.queryByRole("dialog", { name: "Palette de commandes" })).not.toBeInTheDocument();
  });

  it("ouvre les Réglages en surcouche et rend l'écran intact à la fermeture", async () => {
    const router = renderShell(["/applications"]);
    fireEvent.keyDown(document, { key: ",", metaKey: true });
    expect(await screen.findByRole("dialog", { name: "Réglages" })).toBeInTheDocument();
    expect(router.state.location.pathname).toBe("/applications");
    fireEvent.keyDown(document, { key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("dialog", { name: "Réglages" })).not.toBeInTheDocument());
    expect(screen.getByText("Écran Candidatures")).toBeInTheDocument();
  });

  it("change de destination avec G puis une lettre", async () => {
    const router = renderShell();
    act(() => {
      fireEvent.keyDown(document.body, { key: "g" });
      fireEvent.keyDown(document.body, { key: "c" });
    });
    await waitFor(() => expect(router.state.location.pathname).toBe("/applications"));
  });

  it("laisse la saisie intacte : G tapé dans un champ ne navigue pas", async () => {
    const router = renderShell(["/applications"]);
    const champ = screen.getByRole("textbox", { name: "Recherche" });
    await userEvent.type(champ, "gp");
    expect(champ).toHaveValue("gp");
    expect(router.state.location.pathname).toBe("/applications");
  });
});
