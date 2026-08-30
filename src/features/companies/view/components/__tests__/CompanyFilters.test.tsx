import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CompanyFilters } from "../CompanyFilters";
import type { CompanyCriteria } from "../../../viewmodel/useCompaniesViewModel";
import { CRITERES_VIDES } from "../../../viewmodel/useCompaniesViewModel";
import { referentialService } from "@/features/referentials/services/referentialService";
import { QueryWrapper, REFERENTIELS_DE_TEST } from "@/shared/lib/test-utils";

async function openFilters() {
  await userEvent.click(screen.getByRole("button", { name: /Filtres/ }));
}

/** Rend la barre avec ses valeurs par défaut, surchargées au cas par cas. */
function renderFilters(props: {
  criteres?: CompanyCriteria;
  count?: number;
  total?: number;
  onApply?: (values: CompanyCriteria) => void;
  onReset?: () => void;
  actions?: React.ReactNode;
}) {
  return render(
    <QueryWrapper>
      <CompanyFilters
        search=""
        onSearch={() => {}}
        criteres={props.criteres ?? CRITERES_VIDES}
        count={props.count ?? 0}
        total={props.total ?? 0}
        onApply={props.onApply ?? (() => {})}
        onReset={props.onReset ?? (() => {})}
        actions={props.actions}
      />
    </QueryWrapper>,
  );
}

beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(referentialService, "load").mockResolvedValue(REFERENTIELS_DE_TEST);
});

describe("barre de filtres des entreprises", () => {
  it("affiche la recherche, le total et l'action principale", () => {
    renderFilters({ total: 4, actions: <button type="button">Nouvelle entreprise</button> });

    expect(screen.getByRole("searchbox", { name: "Rechercher…" })).toBeInTheDocument();
    expect(screen.getByText("4 entreprises")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Nouvelle entreprise" })).toBeInTheDocument();
  });

  it("propose les types du référentiel et applique celui qui est choisi", async () => {
    // Les options viennent de la base : une liste écrite dans le composant divergerait du
    // référentiel au premier ajout.
    const onApply = vi.fn();
    renderFilters({ onApply });

    await openFilters();
    await userEvent.click(await screen.findByRole("button", { name: "Client final" }));

    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({ company_type_id: "FINAL_CLIENT" }),
    );
  });

  it("ôte le type actif au second clic et affiche son libellé en chip", async () => {
    const onApply = vi.fn();
    renderFilters({
      criteres: { ...CRITERES_VIDES, company_type_id: "IT_SERVICES_COMPANY" },
      count: 1,
      onApply,
    });

    // Le chip affiche le libellé français, jamais le code persisté.
    expect(
      await screen.findByText("Type · ESN / Société de services numériques"),
    ).toBeInTheDocument();
    await openFilters();
    await userEvent.click(
      screen.getByRole("button", { name: "ESN / Société de services numériques" }),
    );

    expect(onApply).toHaveBeenCalledWith(expect.objectContaining({ company_type_id: null }));
  });

  it("filtre par taille indépendamment du type", async () => {
    const onApply = vi.fn();
    renderFilters({
      criteres: { ...CRITERES_VIDES, company_type_id: "IT_SERVICES_COMPANY" },
      count: 1,
      onApply,
    });

    await openFilters();
    await userEvent.click(screen.getByRole("button", { name: "PME" }));

    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        company_type_id: "IT_SERVICES_COMPANY",
        company_size: "PME",
      }),
    );
  });

  it("efface les critères via Tout effacer", async () => {
    const onReset = vi.fn();
    renderFilters({
      criteres: { ...CRITERES_VIDES, company_size: "PME" },
      count: 1,
      onReset,
    });

    await userEvent.click(screen.getByRole("button", { name: "Tout effacer" }));
    expect(onReset).toHaveBeenCalled();
  });
});
