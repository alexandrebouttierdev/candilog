import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  createMemoryRouter,
  MemoryRouter,
  RouterProvider,
  type RouteObject,
} from "react-router-dom";
import { fireEvent, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { QueryWrapper } from "@/shared/lib/test-utils";
import { managedOllamaService, settingsService } from "@/features/settings";
import type { Settings } from "@/shared/types/generated/settings";
import { AppShell } from "../AppShell";
import { NavRail } from "../NavRail";
import { TopBar } from "../TopBar";

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
  theme: "system",
  language: "fr",
};

function renderShell(children: RouteObject[], initialEntries = ["/"]) {
  const router = createMemoryRouter(
    [{ path: "/", element: <AppShell />, children }],
    { initialEntries },
  );
  return render(
    <QueryWrapper>
      <RouterProvider router={router} />
    </QueryWrapper>,
  );
}

beforeEach(() => {
  vi.restoreAllMocks();
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
    renderShell([{ index: true, element: <p>Accueil</p> }]);
    const skip = screen.getByRole("link", { name: "Aller au contenu" });
    expect(skip).toHaveAttribute("href", "#contenu");
    expect(screen.getAllByRole("main")).toHaveLength(1);
    expect(screen.getByRole("main")).toHaveAttribute("id", "contenu");
  });

  it("ne navigue plus au clavier avec Ctrl/Cmd + chiffre", () => {
    renderShell([
      { index: true, element: <p>Accueil</p> },
      { path: "tracking/applications", element: <p>Candidatures</p> },
    ]);

    fireEvent.keyDown(document, { key: "2", ctrlKey: true });

    expect(screen.getByText("Accueil")).toBeInTheDocument();
    expect(screen.queryByText("Candidatures")).not.toBeInTheDocument();
  });

  it("n'ouvre plus de palette globale avec Ctrl/Cmd+K", () => {
    renderShell([{ index: true, element: <p>Accueil</p> }]);

    fireEvent.keyDown(document, { key: "k", ctrlKey: true });

    expect(screen.queryByRole("dialog", { name: "Palette de commandes" })).not.toBeInTheDocument();
  });
});

describe("rail de navigation", () => {
  it("expose des entrées accessibles par icône avec libellé complet", () => {
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    render(
      <QueryClientProvider client={client}>
        <MemoryRouter>
          <NavRail />
        </MemoryRouter>
      </QueryClientProvider>,
    );
    expect(screen.getByRole("navigation", { name: "Navigation principale" })).toHaveClass("z-20");
    expect(screen.getByRole("link", { name: "Aujourd'hui" })).toBeInTheDocument();
    // Plus aucune pastille de raccourci : la navigation se fait à la souris ou au clavier
    // par tabulation, pas par une combinaison à mémoriser.
    expect(screen.queryByText("⌘1")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Passer en thème sombre" })).toBeInTheDocument();
    const logo = screen.getByRole("img", { name: "Candilog" });
    expect(logo).toHaveAttribute("width", "36");
    expect(logo).toHaveAttribute("height", "36");
  });
});

describe("barre supérieure", () => {
  it("n'affiche pas de titre d'écran et expose les contrôles IA", () => {
    render(
      <QueryWrapper>
        <MemoryRouter initialEntries={["/tracking/calendar"]}>
          <TopBar slotRef={() => {}} />
        </MemoryRouter>
      </QueryWrapper>,
    );
    expect(screen.queryByRole("heading", { name: "Calendrier" })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Fournisseur IA/ })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Tester" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Réglages Intelligence artificielle" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Rechercher ou exécuter" })).not.toBeInTheDocument();
  });
});
