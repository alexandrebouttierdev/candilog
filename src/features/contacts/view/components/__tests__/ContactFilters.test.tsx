import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ContactFilters } from "../ContactFilters";

async function openFilters() {
  await userEvent.click(screen.getByRole("button", { name: /Filtres/ }));
}

describe("barre de filtres du réseau", () => {
  it("affiche la recherche, le total et l'action principale", () => {
    render(
      <ContactFilters
        search=""
        onSearch={() => {}}
        tracking_role={null}
        count={0}
        total={3}
        onSelectRole={() => {}}
        onReset={() => {}}
        actions={<button type="button">Nouveau contact</button>}
      />,
    );

    expect(screen.getByRole("searchbox", { name: "Rechercher…" })).toBeInTheDocument();
    expect(screen.getByText("3 contacts")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Nouveau contact" })).toBeInTheDocument();
  });

  it("sélectionne un seul rôle à la fois", async () => {
    const onSelectRole = vi.fn();
    render(
      <ContactFilters
        search=""
        onSearch={() => {}}
        tracking_role={null}
        count={0}
        total={3}
        onSelectRole={onSelectRole}
        onReset={() => {}}
      />,
    );

    await openFilters();
    await userEvent.click(screen.getByRole("button", { name: "Recruteur" }));

    expect(onSelectRole).toHaveBeenCalledWith("Recruteur");
  });

  it("ôte le rôle actif au second clic et affiche le chip", async () => {
    const onSelectRole = vi.fn();
    render(
      <ContactFilters
        search=""
        onSearch={() => {}}
        tracking_role="Manager"
        count={1}
        total={1}
        onSelectRole={onSelectRole}
        onReset={() => {}}
      />,
    );

    expect(screen.getByText("Rôle · Manager")).toBeInTheDocument();
    await openFilters();
    await userEvent.click(screen.getByRole("button", { name: "Manager" }));

    expect(onSelectRole).toHaveBeenCalledWith(null);
  });
});
