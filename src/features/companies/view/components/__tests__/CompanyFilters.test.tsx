import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CompanyFilters } from "../CompanyFilters";

async function openFilters() {
  await userEvent.click(screen.getByRole("button", { name: /Filtres/ }));
}

describe("barre de filtres des entreprises", () => {
  it("affiche la recherche, le total et l'action principale", () => {
    render(
      <CompanyFilters
        search=""
        onSearch={() => {}}
        company_type={null}
        types={["ESN"]}
        count={0}
        total={4}
        onSelectType={() => {}}
        onReset={() => {}}
        actions={<button type="button">Nouvelle entreprise</button>}
      />,
    );

    expect(screen.getByRole("searchbox", { name: "Rechercher…" })).toBeInTheDocument();
    expect(screen.getByText("4 entreprises")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Nouvelle entreprise" })).toBeInTheDocument();
  });

  it("sélectionne un seul type à la fois et affiche le chip", async () => {
    const onSelectType = vi.fn();
    render(
      <CompanyFilters
        search=""
        onSearch={() => {}}
        company_type={null}
        types={["ESN", "Cabinet"]}
        count={0}
        total={2}
        onSelectType={onSelectType}
        onReset={() => {}}
      />,
    );

    await openFilters();
    await userEvent.click(screen.getByRole("button", { name: "Cabinet" }));

    expect(onSelectType).toHaveBeenCalledWith("Cabinet");
  });

  it("ôte le type actif au second clic", async () => {
    const onSelectType = vi.fn();
    render(
      <CompanyFilters
        search=""
        onSearch={() => {}}
        company_type="ESN"
        types={["ESN", "Cabinet"]}
        count={1}
        total={1}
        onSelectType={onSelectType}
        onReset={() => {}}
      />,
    );

    expect(screen.getByText("Type · ESN")).toBeInTheDocument();
    await openFilters();
    await userEvent.click(screen.getByRole("button", { name: "ESN" }));

    expect(onSelectType).toHaveBeenCalledWith(null);
  });

  it("efface le type via Tout effacer", async () => {
    const onReset = vi.fn();
    render(
      <CompanyFilters
        search=""
        onSearch={() => {}}
        company_type="ESN"
        types={["ESN"]}
        count={1}
        total={1}
        onSelectType={() => {}}
        onReset={onReset}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: "Tout effacer" }));
    expect(onReset).toHaveBeenCalled();
  });
});
