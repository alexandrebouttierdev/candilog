import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ApplicationFilters } from "../ApplicationFilters";
import { FILTER_VIDE } from "../../../model/schemas/application-filter.schema";
import { referentialService } from "@/features/referentials/services/referentialService";
import { QueryWrapper, REFERENTIELS_DE_TEST } from "@/shared/lib/test-utils";

/** Conteneur des options d'un groupe de filtres, désigné par son intitulé. */
async function groupe(label: string): Promise<HTMLElement> {
  const titre = await screen.findByText(label);
  return titre.parentElement as HTMLElement;
}

async function openFilters() {
  // Le déclencheur porte le nombre de critères actifs dès qu'il y en a : la recherche
  // exacte échouerait sur une barre déjà filtrée.
  await userEvent.click(screen.getByRole("button", { name: /Filtres/ }));
}

/** Rend la barre de filtres vide, seul `onApply` variant selon les tests. */
function renderFilters(onApply: (values: typeof FILTER_VIDE) => void) {
  return render(
    <QueryWrapper>
      <ApplicationFilters
        search=""
        onSearch={() => {}}
        filters={FILTER_VIDE}
        count={0}
        total={0}
        onApply={onApply}
        onReset={() => {}}
      />
    </QueryWrapper>,
  );
}

beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(referentialService, "load").mockResolvedValue(REFERENTIELS_DE_TEST);
});

describe("filtres de période", () => {
  it("signale une date inexistante au blur, sans l'appliquer", async () => {
    const onApply = vi.fn();
    renderFilters(onApply);

    await openFilters();
    await userEvent.type(screen.getByLabelText("Début de période"), "31-02-2026");
    await userEvent.tab();

    expect(screen.getByText(/Date invalide/)).toBeInTheDocument();
    expect(onApply).not.toHaveBeenCalled();
  });

  it("applique une date valide saisie au clavier", async () => {
    const onApply = vi.fn();
    renderFilters(onApply);

    await openFilters();
    await userEvent.type(screen.getByLabelText("Début de période"), "01-08-2026");

    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({ start_date: "2026-08-01" }),
    );
  });
});

describe("filtres alimentés par les référentiels", () => {
  it("propose les contrats de la base et applique le code, pas le libellé", async () => {
    // L'utilisateur choisit « Intérim », la base reçoit « MIS » : afficher le code serait
    // illisible, l'enregistrer en clair briserait la clé étrangère.
    const onApply = vi.fn();
    renderFilters(onApply);

    await openFilters();
    await userEvent.click(await screen.findByRole("button", { name: "Intérim" }));

    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({ contract_type_code: ["MIS"] }),
    );
  });

  it("cumule plusieurs domaines professionnels", async () => {
    const onApply = vi.fn();
    render(
      <QueryWrapper>
        <ApplicationFilters
          search=""
          onSearch={() => {}}
          filters={{ ...FILTER_VIDE, professional_domain_id: ["M18"] }}
          count={1}
          total={0}
          onApply={onApply}
          onReset={() => {}}
        />
      </QueryWrapper>,
    );

    await openFilters();
    // « Banque / Assurance » figure aussi parmi les secteurs : la recherche est portée sur
    // le groupe, faute de quoi le test cliquerait sur l'autre référentiel — exactement la
    // confusion que la séparation des concepts doit rendre impossible.
    await userEvent.click(
      within(await groupe("Domaine professionnel")).getByRole("button", {
        name: "Banque / Assurance",
      }),
    );

    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({ professional_domain_id: ["M18", "C"] }),
    );
  });

  it("distingue le secteur de l'entreprise du domaine du poste", async () => {
    const onApply = vi.fn();
    renderFilters(onApply);

    await openFilters();
    await userEvent.click(
      within(await groupe("Secteur d'activité")).getByRole("button", {
        name: "Banque / Assurance",
      }),
    );

    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        sector_id: ["5ec70000-0000-4000-8000-000000000003"],
        professional_domain_id: [],
      }),
    );
  });
});

describe("filtre par amplitude horaire", () => {
  it("transmet les bornes saisies", async () => {
    const onApply = vi.fn();
    renderFilters(onApply);

    await openFilters();
    // `fireEvent` et non une frappe caractère par caractère : le champ est contrôlé par la
    // valeur du filtre, qui ne bouge pas ici, et chaque frappe repartirait donc de zéro.
    fireEvent.change(screen.getByPlaceholderText("min"), { target: { value: "24" } });

    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({ min_weekly_hours: 24 }),
    );
  });
});
