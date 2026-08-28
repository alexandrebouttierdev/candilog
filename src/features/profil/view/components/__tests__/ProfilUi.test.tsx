import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CompletionRing, ProfilPanel, ProfilTabs } from "../ProfilUi";

const counts = { experiences: 2, competences: 5, formations: 1, langues: 3 };

describe("interface du profil", () => {
  it("expose la section active comme un véritable onglet", async () => {
    const onChange = vi.fn();
    render(<ProfilTabs active="experiences" counts={counts} onChange={onChange} />);

    expect(screen.getByRole("tab", { name: /Expériences/ })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("tab", { name: /Compétences/ })).toHaveAttribute("aria-selected", "false");
    await userEvent.click(screen.getByRole("tab", { name: /Compétences/ }));
    expect(onChange).toHaveBeenCalledWith("competences");

    screen.getByRole("tab", { name: /Expériences/ }).focus();
    await userEvent.keyboard("{ArrowRight}");
    expect(onChange).toHaveBeenLastCalledWith("competences");
    expect(screen.getByRole("tab", { name: /Compétences/ })).toHaveFocus();
  });

  it("ne rend que le panneau actif", () => {
    const { rerender } = render(<ProfilPanel tab="langues" active={false}>Français</ProfilPanel>);
    expect(screen.queryByText("Français")).not.toBeInTheDocument();
    rerender(<ProfilPanel tab="langues" active>Français</ProfilPanel>);
    expect(screen.getByRole("tabpanel")).toHaveTextContent("Français");
  });

  it("annonce la complétion aux technologies d'assistance", () => {
    render(<CompletionRing value={57} />);
    expect(screen.getByRole("img", { name: "Profil complété à 57 %" })).toBeInTheDocument();
  });
});
