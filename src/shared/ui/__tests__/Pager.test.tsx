import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Pager, page_bounds } from "../Pager";

describe("bornes de page", () => {
  it("décrit la tranche affichée", () => {
    expect(page_bounds(2, 8, 15)).toMatchObject({ from: 9, to: 15, page_count: 2 });
  });

  it("affiche 0–0 sur une collection vide plutôt que 1–0", () => {
    expect(page_bounds(1, 8, 0)).toMatchObject({ from: 0, to: 0 });
  });

  it("garde une page même sans élément, pour ne pas afficher « page 1 sur 0 »", () => {
    expect(page_bounds(1, 8, 0).page_count).toBe(1);
  });

  it("borne la fin de page au total réel", () => {
    expect(page_bounds(2, 8, 12).to).toBe(12);
  });

  it("désactive les flèches aux extrémités", () => {
    expect(page_bounds(1, 8, 15)).toMatchObject({ hasPrev: false, hasNext: true });
    expect(page_bounds(2, 8, 15)).toMatchObject({ hasPrev: true, hasNext: false });
  });
});

describe("Pager", () => {
  it("annonce la tranche et le total", () => {
    render(<Pager page={1} page_size={8} total={15} label="candidatures" onPageChange={vi.fn()} />);
    expect(screen.getByText("1–8 sur 15 candidatures")).toBeInTheDocument();
  });

  it("demande la page suivante sans jamais recevoir la collection", async () => {
    // Le composant ne connaît que `page`, `page_size` et `total` : il ne peut donc pas
    // paginer côté client, ce qui est précisément la garantie recherchée.
    const onPageChange = vi.fn();
    render(<Pager page={1} page_size={8} total={15} label="candidatures" onPageChange={onPageChange} />);

    await userEvent.click(screen.getByRole("button", { name: "Page suivante" }));

    expect(onPageChange).toHaveBeenCalledWith(2);
  });

  it("empêche de reculer avant la première page", async () => {
    const onPageChange = vi.fn();
    render(<Pager page={1} page_size={8} total={15} label="candidatures" onPageChange={onPageChange} />);

    await userEvent.click(screen.getByRole("button", { name: "Page précédente" }));

    expect(onPageChange).not.toHaveBeenCalled();
  });

  it("marque la page courante pour les technologies d'assistance", () => {
    render(<Pager page={2} page_size={4} total={15} label="candidatures" onPageChange={vi.fn()} />);
    expect(screen.getByRole("button", { current: "page" })).toHaveTextContent("2");
  });
});
