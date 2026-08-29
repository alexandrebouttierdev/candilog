import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ApplicationFilters } from "../ApplicationFilters";
import type { ApplicationFilterValues } from "../../../model/schemas/application-filter.schema";

const FILTERS: ApplicationFilterValues = {
  status: [],
  contract: [],
  company_id: null,
  city: "",
  job_title: "",
  start_date: null,
  end_date: null,
};

async function openFilters() {
  await userEvent.click(screen.getByRole("button", { name: "Filtres" }));
}

describe("filtres de période", () => {
  it("signale une date inexistante au blur, sans l'appliquer", async () => {
    const onApply = vi.fn();
    render(
      <ApplicationFilters
        search=""
        onSearch={() => {}}
        filters={FILTERS}
        count={0}
        total={0}
        onApply={onApply}
        onReset={() => {}}
      />,
    );

    await openFilters();
    const debut = screen.getByLabelText("Début de période");
    await userEvent.type(debut, "31-02-2026");
    await userEvent.tab();

    expect(screen.getByText(/Date invalide/)).toBeInTheDocument();
    expect(onApply).not.toHaveBeenCalled();
  });

  it("applique une date valide saisie au clavier", async () => {
    const onApply = vi.fn();
    render(
      <ApplicationFilters
        search=""
        onSearch={() => {}}
        filters={FILTERS}
        count={0}
        total={0}
        onApply={onApply}
        onReset={() => {}}
      />,
    );

    await openFilters();
    await userEvent.type(screen.getByLabelText("Début de période"), "01-08-2026");

    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({ start_date: "2026-08-01" }),
    );
  });
});
