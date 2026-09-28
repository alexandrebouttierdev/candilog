import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { userWithoutDelay } from "@/shared/lib/test-user";
import { ApplicationToolbar } from "../ApplicationToolbar";
import { EMPTY_FILTER } from "../../../model/schemas/application-filter.schema";
import type { ApplicationFilterValues } from "../../../model/schemas/application-filter.schema";
import { referentialService } from "@/features/referentials";
import { QueryWrapper, REFERENTIELS_DE_TEST } from "@/shared/lib/test-utils";

function renderToolbar(onApply: (values: ApplicationFilterValues) => void, filters = EMPTY_FILTER) {
  return render(
    <QueryWrapper>
      <ApplicationToolbar
        filters={filters}
        onApply={onApply}
        onReset={() => {}}
        search=""
        onSearch={() => {}}
        count={null}
        actions={null}
      />
    </QueryWrapper>,
  );
}

/** Ouvre « + Filtre » puis le champ demandé (premier niveau du menu). */
async function openField(label: string) {
  await userEvent.click(screen.getByRole("button", { name: /Filtre/ }));
  await userEvent.click(await screen.findByRole("menuitem", { name: new RegExp(`^${label}`) }));
}

beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(referentialService, "load").mockResolvedValue(REFERENTIELS_DE_TEST);
});

describe("menu de filtre — période d'envoi", () => {
  it("signale une date inexistante sans l'appliquer", async () => {
    const onApply = vi.fn();
    renderToolbar(onApply);
    const user = userWithoutDelay();

    await openField("Envoyée");
    await user.click(screen.getByRole("menuitem", { name: "Entre deux dates…" }));
    await user.type(screen.getByLabelText("Du"), "31-02-2026");
    await user.click(screen.getByRole("menuitem", { name: "Appliquer la période" }));

    expect(screen.getByRole("alert")).toHaveTextContent(/Date invalide/);
    expect(onApply).not.toHaveBeenCalled();
  });

  it("applique une date valide saisie au clavier, validée par Entrée", async () => {
    const onApply = vi.fn();
    renderToolbar(onApply);
    const user = userWithoutDelay();

    await openField("Envoyée");
    await user.click(screen.getByRole("menuitem", { name: "Entre deux dates…" }));
    await user.type(screen.getByLabelText("Du"), "01-08-2026{Enter}");

    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({ start_date: "2026-08-01", end_date: null }),
    );
  });

  it("refuse une période inversée", async () => {
    const onApply = vi.fn();
    renderToolbar(onApply);
    const user = userWithoutDelay();

    await openField("Envoyée");
    await user.click(screen.getByRole("menuitem", { name: "Entre deux dates…" }));
    await user.type(screen.getByLabelText("Du"), "10-08-2026");
    await user.type(screen.getByLabelText("Au"), "01-08-2026{Enter}");

    expect(screen.getByRole("alert")).toHaveTextContent("La fin de période précède son début.");
    expect(onApply).not.toHaveBeenCalled();
  });
});

describe("menu de filtre — référentiels", () => {
  it("propose les contrats de la base et applique le code, pas le libellé", async () => {
    // L'utilisateur choisit « Intérim », la base reçoit « MIS » : afficher le code serait
    // illisible, l'enregistrer en clair briserait la clé étrangère.
    const onApply = vi.fn();
    renderToolbar(onApply);

    await openField("Contrat");
    await userEvent.click(await screen.findByRole("menuitemradio", { name: "Intérim" }));

    expect(onApply).toHaveBeenCalledWith(expect.objectContaining({ contract_type_code: ["MIS"] }));
  });

  it("cumule plusieurs domaines professionnels", async () => {
    const onApply = vi.fn();
    renderToolbar(onApply, { ...EMPTY_FILTER, professional_domain_id: ["M18"] });

    await openField("Domaine");
    await userEvent.click(await screen.findByRole("menuitemradio", { name: "Banque / Assurance" }));

    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({ professional_domain_id: ["M18", "C"] }),
    );
  });

  it("distingue le secteur de l'entreprise du domaine du poste", async () => {
    // « Banque / Assurance » existe dans les deux référentiels : chaque champ du menu
    // n'ouvre que le sien.
    const onApply = vi.fn();
    renderToolbar(onApply);

    await openField("Secteur");
    await userEvent.click(await screen.findByRole("menuitemradio", { name: "Banque / Assurance" }));

    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        sector_id: ["5ec70000-0000-4000-8000-000000000003"],
        professional_domain_id: [],
      }),
    );
  });
});

describe("menu de filtre — amplitude horaire", () => {
  it("transmet les bornes saisies", async () => {
    const onApply = vi.fn();
    renderToolbar(onApply);
    const user = userWithoutDelay();

    await openField("Heures");
    await user.type(screen.getByLabelText("Minimum d'heures"), "24{Enter}");

    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({ min_weekly_hours: 24, max_weekly_hours: null }),
    );
  });

  it("refuse un minimum supérieur au maximum", async () => {
    const onApply = vi.fn();
    renderToolbar(onApply);
    const user = userWithoutDelay();

    await openField("Heures");
    await user.type(screen.getByLabelText("Minimum d'heures"), "35");
    await user.type(screen.getByLabelText("Maximum d'heures"), "20{Enter}");

    expect(screen.getByRole("alert")).toHaveTextContent("Le maximum d'heures est inférieur au minimum.");
    expect(onApply).not.toHaveBeenCalled();
  });
});

describe("puces des critères actifs", () => {
  it("affiche la condition et l'inverse d'un clic", async () => {
    const onApply = vi.fn();
    renderToolbar(onApply, { ...EMPTY_FILTER, status: ["REFUS"] });

    const puce = screen.getByRole("button", { name: /Statut est Refusée/ });
    await userEvent.click(puce);

    expect(onApply).toHaveBeenCalledWith(expect.objectContaining({ status: ["REFUS"], excluded: ["status"] }));
  });

  it("retire le critère avec sa croix, inversion comprise", async () => {
    const onApply = vi.fn();
    renderToolbar(onApply, { ...EMPTY_FILTER, city: "Rennes", excluded: ["city"] });

    expect(screen.getByRole("button", { name: /Ville n'est pas Rennes/ })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Retirer le filtre Ville" }));

    expect(onApply).toHaveBeenCalledWith(expect.objectContaining({ city: "", excluded: [] }));
  });

  it("ouvre le menu avec F", async () => {
    renderToolbar(vi.fn());

    await userEvent.keyboard("f");

    expect(await screen.findByRole("menu", { name: "Ajouter un filtre" })).toBeInTheDocument();
  });
});
