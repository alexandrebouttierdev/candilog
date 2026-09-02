import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ActiveFilterChip, FilterGroup, FilterMenu, FilterOption } from "../FilterBar";

describe("filtres de tableau", () => {
  it("élargit le menu selon son contenu sans dépasser la fenêtre", async () => {
    render(
      <FilterMenu count={0}>
        <FilterGroup label="Contrat">
          {Array.from({ length: 12 }, (_, index) => (
            <FilterOption
              key={index}
              label={`Type de contrat ${index + 1}`}
              selected={false}
              onSelect={() => {}}
            />
          ))}
        </FilterGroup>
      </FilterMenu>,
    );

    await userEvent.click(screen.getByRole("button", { name: "Filtres" }));

    expect(screen.getByRole("dialog", { name: "Filtres" })).toHaveClass(
      "w-max",
      "min-w-[230px]",
      "max-w-[calc(100vw-2rem)]",
      "sm:max-w-[640px]",
      "max-h-[min(70vh,640px)]",
      "overflow-y-auto",
    );
  });

  it("conserve les libellés des options sur une seule ligne", () => {
    render(
      <FilterOption
        label="Contrat d'engagement éducatif"
        selected={false}
        onSelect={() => {}}
      />,
    );

    expect(screen.getByRole("button", { name: "Contrat d'engagement éducatif" })).toHaveClass(
      "whitespace-nowrap",
    );
  });

  it("limite chaque catégorie à six options puis permet de tout afficher", async () => {
    render(
      <FilterGroup label="Contrat">
        {Array.from({ length: 8 }, (_, index) => (
          <FilterOption
            key={index}
            label={`Contrat ${index + 1}`}
            selected={false}
            onSelect={() => {}}
          />
        ))}
      </FilterGroup>,
    );

    expect(screen.getByRole("button", { name: "Contrat 6" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Contrat 7" })).not.toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Voir plus pour Contrat" }));

    expect(screen.getByRole("button", { name: "Contrat 8" })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Voir moins pour Contrat" }));
    expect(screen.queryByRole("button", { name: "Contrat 7" })).not.toBeInTheDocument();
  });

  it("bascule une option de popover", async () => {
    const onSelect = vi.fn();
    render(<FilterOption label="Entretien" selected={false} onSelect={onSelect} />);
    await userEvent.click(screen.getByRole("button", { name: "Entretien" }));
    expect(onSelect).toHaveBeenCalledOnce();
  });

  it("retire un chip actif", async () => {
    const onRemove = vi.fn();
    render(<ActiveFilterChip field="Statut" value="Entretien" onRemove={onRemove} />);
    expect(screen.getByText("Statut · Entretien")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Retirer le filtre Statut Entretien" }));
    expect(onRemove).toHaveBeenCalledOnce();
  });
});
