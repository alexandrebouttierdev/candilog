import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ActiveFilterChip, FilterOption } from "../FilterBar";

describe("filtres de tableau", () => {
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
    await userEvent.click(screen.getByRole("button", { name: "Retirer le filtre Statut" }));
    expect(onRemove).toHaveBeenCalledOnce();
  });
});
