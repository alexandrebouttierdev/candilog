import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CompletionBar, ProfilePanel, ProfileTabs } from "../ProfileUi";

const counts = {
  identity: null,
  objective: null,
  online: null,
  experiences: 2,
  skills: 5,
  education: 1,
  projects: 4,
  certifications: 2,
  languages: 3,
  interests: 1,
};

describe("interface du profil", () => {
  it("expose la section active comme un véritable onglet", async () => {
    const onChange = vi.fn();
    render(<ProfileTabs active="experiences" counts={counts} onChange={onChange} />);

    expect(screen.getByRole("tab", { name: /Expériences/ })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("tab", { name: /Identité/ })).toHaveAttribute("aria-selected", "false");
    expect(screen.getByRole("tab", { name: /Objectif professionnel/ })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: /Présence en ligne/ })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: /Compétences/ })).toHaveAttribute("aria-selected", "false");
    await userEvent.click(screen.getByRole("tab", { name: /Compétences/ }));
    expect(onChange).toHaveBeenCalledWith("skills");

    screen.getByRole("tab", { name: /Expériences/ }).focus();
    await userEvent.keyboard("{ArrowRight}");
    expect(onChange).toHaveBeenLastCalledWith("skills");
    expect(screen.getByRole("tab", { name: /Compétences/ })).toHaveFocus();
  });

  it("n'affiche pas de compteur sur les onglets d'identité", () => {
    render(<ProfileTabs active="identity" counts={counts} onChange={vi.fn()} />);
    const identite = screen.getByRole("tab", { name: /Identité/ });
    expect(identite.querySelector(".rounded-tag")).toBeNull();
    expect(screen.getByRole("tab", { name: /Expériences/ }).querySelector(".rounded-tag")).toHaveTextContent("2");
  });

  it("ne rend que le panneau actif", () => {
    const { rerender } = render(<ProfilePanel tab="languages" active={false}>Français</ProfilePanel>);
    expect(screen.queryByText("Français")).not.toBeInTheDocument();
    rerender(<ProfilePanel tab="languages" active>Français</ProfilePanel>);
    expect(screen.getByRole("tabpanel")).toHaveTextContent("Français");
  });

  it("annonce la complétion aux technologies d'assistance", () => {
    render(<CompletionBar value={57} hint="Ajoutez vos formations." />);
    const barre = screen.getByRole("progressbar", { name: "Profil complété" });
    expect(barre).toHaveAttribute("aria-valuenow", "57");
  });
});
