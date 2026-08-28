import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { DataTable } from "../DataTable";
import type { Column } from "../DataTable";
import { EmptyState } from "../EmptyState";

interface Row {
  id: string;
  job_title: string;
  company: string;
}

const Rows: Row[] = [
  { id: "1", job_title: "Développeur Frontend", company: "Nova Digital" },
  { id: "2", job_title: "Product Designer", company: "Atlas Studio" },
];

const COLUMNS: Column<Row, "job_title" | "company">[] = [
  { key: "job_title", header: "Poste", sort_key: "job_title", render: (row) => row.job_title },
  { key: "company", header: "Entreprise", render: (row) => row.company },
];

describe("DataTable", () => {
  it("rend une ligne par élément de la page", () => {
    render(<DataTable columns={COLUMNS} rows={Rows} row_key={(row) => row.id} />);
    expect(screen.getAllByRole("row")).toHaveLength(Rows.length + 1);
  });

  it("n'applique pas le tri lui-même", () => {
    // Trier ici ne trierait que les éléments chargés : l'ordre serait faux dès la seconde
    // page. Malgré un tri descendant déclaré, les lignes restent dans l'ordre reçu.
    render(
      <DataTable
        columns={COLUMNS}
        rows={Rows}
        row_key={(row) => row.id}
        sort={{ key: "job_title", direction: "desc" }}
        onSortChange={vi.fn()}
      />,
    );

    expect(screen.getAllByRole("cell")[0]).toHaveTextContent("Développeur Frontend");
  });

  it("demande le tri de la colonne cliquée", async () => {
    const onSortChange = vi.fn();
    render(
      <DataTable columns={COLUMNS} rows={Rows} row_key={(row) => row.id} onSortChange={onSortChange} />,
    );

    await userEvent.click(screen.getByRole("button", { name: /Poste/ }));

    expect(onSortChange).toHaveBeenCalledWith("job_title");
  });

  it("annonce la direction du tri de la colonne active", () => {
    render(
      <DataTable
        columns={COLUMNS}
        rows={Rows}
        row_key={(row) => row.id}
        sort={{ key: "job_title", direction: "asc" }}
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
      <DataTable columns={COLUMNS} rows={Rows} row_key={(row) => row.id} onSortChange={vi.fn()} />,
    );
    expect(screen.queryByRole("button", { name: "Entreprise" })).not.toBeInTheDocument();
  });

  it("substitue l'état vide au corps, en conservant les en-têtes", () => {
    // Garder les en-têtes indique de quoi la liste serait faite : un cadre entièrement vide
    // est indiscernable d'un écran en panne.
    render(
      <DataTable
        columns={COLUMNS}
        rows={[]}
        row_key={(row) => row.id}
        empty_state={<EmptyState title="Aucune candidature" />}
      />,
    );

    expect(screen.getByText("Aucune candidature")).toBeInTheDocument();
    expect(screen.getByText("Poste")).toBeInTheDocument();
  });

  it("ouvre la ligne au clavier autant qu'à la souris", async () => {
    const onRowClick = vi.fn();
    render(
      <DataTable columns={COLUMNS} rows={Rows} row_key={(row) => row.id} onRowClick={onRowClick} />,
    );

    const premiere = screen.getAllByRole("row")[1]!;
    premiere.focus();
    await userEvent.keyboard("{Enter}");

    expect(onRowClick).toHaveBeenCalledWith(Rows[0]);
  });
});
