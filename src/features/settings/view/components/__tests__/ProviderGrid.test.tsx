import { describe, expect, it, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ProviderGrid } from "../ProviderGrid";
import { PROVIDERS, OTHER_PROVIDERS } from "../../../model/providers";

describe("grille des fournisseurs", () => {
  it("propose tous les fournisseurs comme un groupe de boutons radio", () => {
    render(<ProviderGrid value="openai" onChange={() => undefined} />);
    expect(screen.getByRole("radiogroup", { name: "Fournisseur IA" })).toBeInTheDocument();
    expect(screen.getAllByRole("radio")).toHaveLength(PROVIDERS.length);
    expect(screen.getByRole("radio", { name: /OpenAI/ })).toHaveAttribute("aria-checked", "true");
    expect(screen.getByRole("radio", { name: "Mistral" })).toHaveAttribute("aria-checked", "false");
    for (const fournisseur of PROVIDERS) {
      const radio = screen.getByRole("radio", { name: fournisseur.label });
      expect(within(radio).getByText(fournisseur.label)).toBeInTheDocument();
    }
  });

  it("permet de restreindre la liste aux autres fournisseurs", () => {
    render(
      <ProviderGrid value="openai" onChange={() => undefined} items={OTHER_PROVIDERS} />,
    );
    expect(screen.getAllByRole("radio")).toHaveLength(OTHER_PROVIDERS.length);
    expect(screen.queryByRole("radio", { name: "IA locale Candilog" })).not.toBeInTheDocument();
    expect(screen.queryByRole("radio", { name: "Ollama" })).not.toBeInTheDocument();
  });

  it("signale le fournisseur choisi et notifie le changement", async () => {
    const onChange = vi.fn();
    render(<ProviderGrid value="openai" onChange={onChange} />);
    await userEvent.click(screen.getByRole("radio", { name: /Claude/ }));
    expect(onChange).toHaveBeenCalledWith("claude");
  });

  it("marque la tuile choisie autrement que par la seule couleur", () => {
    render(<ProviderGrid value="openai" onChange={() => undefined} />);

    expect(
      within(screen.getByRole("radio", { name: "OpenAI" })).getByText("check_circle"),
    ).toBeInTheDocument();
    expect(
      within(screen.getByRole("radio", { name: "Mistral" })).queryByText("check_circle"),
    ).not.toBeInTheDocument();
  });

  it("nomme le fournisseur Candilog « IA locale Candilog »", () => {
    render(<ProviderGrid value="candilog_local" onChange={() => undefined} />);

    expect(screen.getByRole("radio", { name: "IA locale Candilog" })).toBeInTheDocument();
  });

  it("présente l'IA locale Candilog comme le choix recommandé", () => {
    render(<ProviderGrid value="openai" onChange={() => undefined} />);

    expect(
      within(screen.getByRole("radio", { name: "IA locale Candilog" })).getByText("Recommandé"),
    ).toBeInTheDocument();
    expect(
      within(screen.getByRole("radio", { name: "Personnalisé" })).getByText(
        "Compatible OpenAI, Ollama local, LM Studio, etc.",
      ),
    ).toBeInTheDocument();
  });

  it("affiche le logo Candilog sur la tuile IA locale", () => {
    render(<ProviderGrid value="openai" onChange={() => undefined} />);
    const tuile = screen.getByRole("radio", { name: "IA locale Candilog" });

    expect(tuile.querySelector("img")).not.toBeNull();
  });
});
