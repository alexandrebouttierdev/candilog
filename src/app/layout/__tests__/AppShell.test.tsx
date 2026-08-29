import { describe, expect, it } from "vitest";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { render, screen } from "@testing-library/react";
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
        <TopBar slotRef={() => {}} onOpenPalette={() => {}} />
      </MemoryRouter>,
    );
    expect(screen.getByRole("heading", { name: "Calendrier" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Rechercher ou exécuter" })).toBeInTheDocument();
  });

  it("fixe la recherche à gauche et le titre au centre, indépendamment du libellé", () => {
    render(
      <MemoryRouter initialEntries={["/settings/ai"]}>
        <TopBar slotRef={() => {}} onOpenPalette={() => {}} />
      </MemoryRouter>,
    );
    const header = screen.getByRole("banner");
    const title = screen.getByRole("heading", { name: "Intelligence artificielle" });
    const search = screen.getByRole("button", { name: "Rechercher ou exécuter" });
    expect(header).toHaveClass("grid");
    expect(header.firstElementChild).toContainElement(title);
    expect(search.parentElement).toHaveClass("col-start-1");
  });
});
