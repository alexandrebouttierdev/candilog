import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { DataTable } from "../DataTable";
import type { Column } from "../DataTable";
import { EmptyState } from "../EmptyState";

interface Ligne {
  id: string;
  poste: string;
  entreprise: string;
}

const LIGNES: Ligne[] = [
  { id: "1", poste: "Développeur Frontend", entreprise: "Nova Digital" },
  { id: "2", poste: "Product Designer", entreprise: "Atlas Studio" },
];

const COLONNES: Column<Ligne, "poste" | "entreprise">[] = [
  { key: "poste", header: "Poste", sortKey: "poste", render: (row) => row.poste },
  { key: "entreprise", header: "Entreprise", render: (row) => row.entreprise },
];

describe("DataTable", () => {
  it("rend une ligne par élément de la page", () => {
    render(<DataTable columns={COLONNES} rows={LIGNES} rowKey={(row) => row.id} />);
    expect(screen.getAllByRole("row")).toHaveLength(LIGNES.length + 1);
  });

  it("n'applique pas le tri lui-même", () => {
    // Trier ici ne trierait que les éléments chargés : l'ordre serait faux dès la seconde
    // page. Malgré un tri descendant déclaré, les lignes restent dans l'ordre reçu.
    render(
      <DataTable
        columns={COLONNES}
        rows={LIGNES}
        rowKey={(row) => row.id}
        sort={{ key: "poste", direction: "desc" }}
        onSortChange={vi.fn()}
      />,
    );

    expect(screen.getAllByRole("cell")[0]).toHaveTextContent("Développeur Frontend");
  });

  it("demande le tri de la colonne cliquée", async () => {
    const onSortChange = vi.fn();
    render(
      <DataTable columns={COLONNES} rows={LIGNES} rowKey={(row) => row.id} onSortChange={onSortChange} />,
    );

    await userEvent.click(screen.getByRole("button", { name: /Poste/ }));

    expect(onSortChange).toHaveBeenCalledWith("poste");
  });

  it("annonce la direction du tri de la colonne active", () => {
    render(
      <DataTable
        columns={COLONNES}
        rows={LIGNES}
        rowKey={(row) => row.id}
        sort={{ key: "poste", direction: "asc" }}
        onSortChange={vi.fn()}
      />,
    );

    expect(screen.getByRole("columnheader", { name: /Poste/ })).toHaveAttribute(
      "aria-sort",
      "ascending",
    );
    expect(screen.getByRole("columnheader", { name: "Entreprise" })).not.toHaveAttribute(
      "aria-sort",
    );
  });

  it("ne rend pas triable une colonne sans clé de tri", () => {
    render(
      <DataTable columns={COLONNES} rows={LIGNES} rowKey={(row) => row.id} onSortChange={vi.fn()} />,
    );
    expect(screen.queryByRole("button", { name: "Entreprise" })).not.toBeInTheDocument();
  });

  it("substitue l'état vide au corps, en conservant les en-têtes", () => {
    // Garder les en-têtes indique de quoi la liste serait faite : un cadre entièrement vide
    // est indiscernable d'un écran en panne.
    render(
      <DataTable
        columns={COLONNES}
        rows={[]}
        rowKey={(row) => row.id}
        emptyState={<EmptyState title="Aucune candidature" />}
      />,
    );

    expect(screen.getByText("Aucune candidature")).toBeInTheDocument();
    expect(screen.getByText("Poste")).toBeInTheDocument();
  });

  it("ouvre la ligne au clavier autant qu'à la souris", async () => {
    const onRowClick = vi.fn();
    render(
      <DataTable columns={COLONNES} rows={LIGNES} rowKey={(row) => row.id} onRowClick={onRowClick} />,
    );

    const premiere = screen.getAllByRole("row")[1]!;
    premiere.focus();
    await userEvent.keyboard("{Enter}");

    expect(onRowClick).toHaveBeenCalledWith(LIGNES[0]);
  });
});
