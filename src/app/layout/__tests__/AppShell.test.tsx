import { describe, expect, it } from "vitest";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { fireEvent, render, screen } from "@testing-library/react";
import { AppShell } from "../AppShell";
import { NavRail } from "../NavRail";
import { TopBar } from "../TopBar";

describe("coque applicative", () => {
  it("offre un lien d'évitement vers le contenu et un seul main", () => {
    render(
      <MemoryRouter>
        <Routes>
          <Route element={<AppShell />}>
            <Route index element={<p>Accueil</p>} />
          </Route>
        </Routes>
      </MemoryRouter>,
    );
    const skip = screen.getByRole("link", { name: "Aller au contenu" });
    expect(skip).toHaveAttribute("href", "#contenu");
    expect(screen.getAllByRole("main")).toHaveLength(1);
    expect(screen.getByRole("main")).toHaveAttribute("id", "contenu");
  });

  it("ne navigue plus au clavier avec Ctrl/Cmd + chiffre", () => {
    render(
      <MemoryRouter initialEntries={["/"]}>
        <Routes>
          <Route element={<AppShell />}>
            <Route index element={<p>Accueil</p>} />
            <Route path="tracking/applications" element={<p>Candidatures</p>} />
          </Route>
        </Routes>
      </MemoryRouter>,
    );

    fireEvent.keyDown(document, { key: "2", ctrlKey: true });

    expect(screen.getByText("Accueil")).toBeInTheDocument();
    expect(screen.queryByText("Candidatures")).not.toBeInTheDocument();
  });

  it("n'ouvre plus de palette globale avec Ctrl/Cmd+K", () => {
    render(
      <MemoryRouter>
        <Routes>
          <Route element={<AppShell />}>
            <Route index element={<p>Accueil</p>} />
          </Route>
        </Routes>
      </MemoryRouter>,
    );

    fireEvent.keyDown(document, { key: "k", ctrlKey: true });

    expect(screen.queryByRole("dialog", { name: "Palette de commandes" })).not.toBeInTheDocument();
  });
});

describe("rail de navigation", () => {
  it("expose des entrées accessibles par icône avec libellé complet", () => {
    render(
      <MemoryRouter>
        <NavRail />
      </MemoryRouter>,
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
  it("affiche le titre de l'écran actif", () => {
    render(
      <MemoryRouter initialEntries={["/tracking/calendar"]}>
        <TopBar slotRef={() => {}} />
      </MemoryRouter>,
    );
    expect(screen.getByRole("heading", { name: "Calendrier" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Rechercher ou exécuter" })).not.toBeInTheDocument();
  });

  it("garde le titre au centre indépendamment du libellé", () => {
    render(
      <MemoryRouter initialEntries={["/settings/ai"]}>
        <TopBar slotRef={() => {}} />
      </MemoryRouter>,
    );
    const header = screen.getByRole("banner");
    const title = screen.getByRole("heading", { name: "Intelligence artificielle" });
    expect(header).toHaveClass("grid");
    expect(header.firstElementChild).toContainElement(title);
    expect(title.parentElement).toHaveClass("col-start-2");
  });
});
