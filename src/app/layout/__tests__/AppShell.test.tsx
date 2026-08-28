import { describe, expect, it } from "vitest";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { render, screen } from "@testing-library/react";
import { AppShell } from "../AppShell";
import { NavRail } from "../NavRail";
import { ContextTabs } from "../ContextTabs";

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
  it("conserve les libellés et un nom accessible, comme les maquettes", () => {
    render(
      <MemoryRouter>
        <NavRail />
      </MemoryRouter>,
    );
    expect(screen.getByRole("link", { name: "Tableau de bord" })).toBeInTheDocument();
    expect(screen.getByText("Accueil")).toHaveClass("text-micro", "font-mid");
    expect(screen.getByRole("button", { name: "Passer en thème sombre" })).toHaveTextContent("Clair");
  });
});

describe("onglets contextuels", () => {
  it("expose l'onglet actif aux technologies d'assistance", () => {
    render(
      <MemoryRouter initialEntries={["/suivi/calendrier"]}>
        <ContextTabs slotRef={() => {}} />
      </MemoryRouter>,
    );
    expect(screen.getByRole("tab", { name: /Calendrier/ })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("tab", { name: /Candidatures/ })).toHaveAttribute("aria-selected", "false");
    expect(screen.getByRole("tab", { name: /Calendrier/ })).toHaveClass("bg-accent-tint");
  });
});
