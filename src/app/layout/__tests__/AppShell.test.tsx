import { beforeEach, describe, expect, it, vi } from "vitest";
import { createMemoryRouter, RouterProvider, type RouteObject } from "react-router-dom";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryWrapper } from "@/shared/lib/test-utils";
import { useUiStore } from "@/shared/lib/ui-store";
import { managedOllamaService, settingsService } from "@/features/settings";
import { syncService } from "@/features/sync/services/syncService";
import { applicationService } from "@/features/applications";
import { profileService } from "@/features/profile";
import type { Settings } from "@/shared/types/generated/settings";
import { Toaster } from "@/shared/ui";
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
  remote_send_consents: [],
  theme: "system",
  language: "fr",
};

const ECRANS: RouteObject[] = [
  { index: true, element: <p>Écran Aujourd'hui</p> },
  { path: "applications", element: <><p>Écran Candidatures</p><input aria-label="Recherche" /></> },
  { path: "profile", element: <p>Écran Profil</p> },
  { path: "ai", element: <p>Écran Intelligence artificielle</p> },
];

function renderShell(initialEntries = ["/"]) {
  const router = createMemoryRouter([{ path: "/", element: <AppShell />, children: ECRANS }], {
    initialEntries,
  });
  render(
    <QueryWrapper>
      <RouterProvider router={router} />
      {/* `App.tsx` monte le Toaster à côté du routeur : sans lui, rien de ce que la coque
          notifie ne serait observable ici. */}
      <Toaster />
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
  // Par défaut, rien à annoncer : les autres tests n'ont pas à composer avec le dialogue.
  vi.spyOn(settingsService, "checkUpdateOnStartup").mockResolvedValue(null);
  vi.spyOn(syncService, "status").mockResolvedValue({
    provider: "local",
    phase: "not_configured",
    auto_sync: false,
    location: null,
    pending_changes: 0,
    conflicts: 0,
    waiting_changes: 0,
    missing_files: 0,
    computers: 0,
    last_success_at: null,
    last_attempt_at: null,
    next_attempt_at: null,
    message: null,
  });
  window.localStorage.setItem(
    "candilog.startup-setup",
    JSON.stringify({ storage: "local", aiPromptDismissed: true }),
  );
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

  it("marque l'IA d'un badge bêta dans la navigation et dans le fil d'Ariane", () => {
    renderShell(["/ai"]);
    const nav = screen.getByRole("navigation", { name: "Navigation principale" });
    // Le badge occupe la place du décompte : l'IA n'a rien à dénombrer.
    expect(within(nav).getByRole("link", { name: /Intelligence artificielle/ })).toHaveTextContent("bêta");
    // Et il redit l'état pour l'écran entier, l'IA n'ayant pas de surcouche à soi.
    const fil = screen.getByRole("navigation", { name: "Fil d'Ariane" });
    expect(fil).toHaveTextContent("bêta");
  });

  it("efface le badge bêta avec les libellés quand le rail tombe à 52 px", () => {
    renderShell(["/ai"]);
    const nav = screen.getByRole("navigation", { name: "Navigation principale" });
    const lien = within(nav).getByRole("link", { name: /Intelligence artificielle/ });

    // Sous le palier de 1060 px le rail n'affiche que les icônes, et « bêta » débordait des
    // 52 px. `hidden` et l'`inline-flex` du badge sont deux utilitaires `display` : lequel
    // gagne ne dépend que de l'ordre de la feuille, que jsdom ne connaît pas. La règle
    // vérifiable ici est donc structurelle — ce qui masque ne déclare pas son propre
    // `display`, sinon il entre en concurrence avec lui-même.
    const masque = within(lien).getByText("bêta").closest(".hidden");
    expect(masque).not.toBeNull();
    expect(masque?.className).not.toMatch(/\b(inline-)?flex\b/);
    expect(masque?.className).toContain("wide:block");
    // Le libellé suit la même règle, et c'est lui qui donne la mesure.
    expect(within(lien).getByText("Intelligence artificielle").className).toContain("hidden");
  });

  it("ouvre un dialogue au démarrage et dirige vers les mises à jour", async () => {
    const verifie = vi.spyOn(settingsService, "checkUpdateOnStartup").mockResolvedValue({
      version: "1.4.0",
      notes: "",
      page_url: "https://github.com/alexandrebouttierdev/candilog/releases/latest",
      asset: null,
    });

    renderShell();

    const dialog = await screen.findByRole("dialog", {
      name: "Candilog 1.4.0 est disponible",
    });
    expect(within(dialog).getByRole("button", { name: "Voir les mises à jour" })).toBeInTheDocument();
    expect(verifie).toHaveBeenCalledTimes(1);

    await userEvent.click(within(dialog).getByRole("button", { name: "Voir les mises à jour" }));

    expect(await screen.findByRole("dialog", { name: "Réglages" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Mises à jour" })).toHaveAttribute("aria-current", "page");
  });

  it("ne dit rien au démarrage quand aucune version n'est annoncée", async () => {
    renderShell();
    await waitFor(() => expect(settingsService.checkUpdateOnStartup).toHaveBeenCalled());
    expect(screen.queryByText(/est disponible/)).not.toBeInTheDocument();
  });

  it("ne met de badge bêta que sur l'IA", () => {
    renderShell(["/applications"]);
    const fil = screen.getByRole("navigation", { name: "Fil d'Ariane" });
    expect(fil).not.toHaveTextContent("bêta");
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
