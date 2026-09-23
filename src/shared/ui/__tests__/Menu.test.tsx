import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Menu } from "../Menu";
import type { MenuEntry } from "../Menu";

function entrees(onOuvrir = vi.fn(), onSupprimer = vi.fn()): MenuEntry[] {
  return [
    { kind: "item", id: "ouvrir", label: "Ouvrir", shortcut: "enter", onSelect: onOuvrir },
    { kind: "separator", id: "s1" },
    { kind: "item", id: "indispo", label: "Llava 7B", disabled: true, reason: "modèle de vision", onSelect: vi.fn() },
    { kind: "item", id: "supprimer", label: "Supprimer…", tone: "danger", onSelect: onSupprimer },
  ];
}

describe("Menu", () => {
  it("active la première entrée disponible sur ⏎ et se ferme", () => {
    const onOuvrir = vi.fn();
    const onClose = vi.fn();
    render(<Menu open anchor={{ x: 10, y: 10 }} entries={entrees(onOuvrir)} onClose={onClose} label="Actions" />);
    fireEvent.keyDown(screen.getByRole("menu"), { key: "Enter" });
    expect(onOuvrir).toHaveBeenCalledOnce();
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("saute les entrées désactivées aux flèches, sans les masquer", () => {
    const onSupprimer = vi.fn();
    render(
      <Menu open anchor={{ x: 10, y: 10 }} entries={entrees(vi.fn(), onSupprimer)} onClose={vi.fn()} label="Actions" />,
    );
    const menu = screen.getByRole("menu");
    // L'entrée incompatible reste visible avec son motif (jamais masquée).
    expect(screen.getByText("modèle de vision")).toBeInTheDocument();
    fireEvent.keyDown(menu, { key: "ArrowDown" });
    fireEvent.keyDown(menu, { key: "Enter" });
    expect(onSupprimer).toHaveBeenCalledOnce();
  });

  it("se ferme sur Échap et sur un clic à l'extérieur", () => {
    const onClose = vi.fn();
    render(<Menu open anchor={{ x: 10, y: 10 }} entries={entrees()} onClose={onClose} label="Actions" />);
    fireEvent.keyDown(document, { key: "Escape" });
    fireEvent.pointerDown(document.body);
    expect(onClose).toHaveBeenCalledTimes(2);
  });
});
