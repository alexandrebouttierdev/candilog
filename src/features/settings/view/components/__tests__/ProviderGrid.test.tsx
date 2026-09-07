import { describe, expect, it, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ProviderGrid } from "../ProviderGrid";
import { FOURNISSEURS } from "../../../model/providers";

describe("grille des fournisseurs", () => {
  it("propose tous les fournisseurs comme un groupe de boutons radio", () => {
    render(<ProviderGrid value="ollama" onChange={() => undefined} />);
    expect(screen.getByRole("radiogroup", { name: "Fournisseur IA" })).toBeInTheDocument();
    expect(screen.getAllByRole("radio")).toHaveLength(FOURNISSEURS.length);
    expect(screen.getByRole("radio", { name: /Ollama/ })).toHaveAttribute("aria-checked", "true");
    expect(screen.getByRole("radio", { name: "OpenAI" })).toHaveAttribute("aria-checked", "false");
    // Le logo est décoratif : la tuile porte déjà le nom en clair et en `aria-label`, un
    // `alt` le ferait annoncer une troisième fois.
    for (const fournisseur of FOURNISSEURS) {
      const radio = screen.getByRole("radio", { name: fournisseur.label });
      expect(within(radio).getByText(fournisseur.label)).toBeInTheDocument();
      expect(within(radio).queryByRole("img")).not.toBeInTheDocument();
    }
  });

  it("signale le fournisseur choisi et notifie le changement", async () => {
    const onChange = vi.fn();
    render(<ProviderGrid value="ollama" onChange={onChange} />);
    await userEvent.click(screen.getByRole("radio", { name: /Claude/ }));
    expect(onChange).toHaveBeenCalledWith("claude");
  });

  // La grille borde et remplit déjà les huit tuiles : la sélection ne se distinguait que
  // par un fond teinté à 10 % et une bordure à 22 % d'opacité en thème sombre. Un repère
  // non chromatique reste lisible dans les deux thèmes, et sans distinguer les couleurs.
  it("marque la tuile choisie autrement que par la seule couleur", () => {
    render(<ProviderGrid value="ollama" onChange={() => undefined} />);

    expect(
      within(screen.getByRole("radio", { name: "Ollama" })).getByText("check_circle"),
    ).toBeInTheDocument();
    expect(
      within(screen.getByRole("radio", { name: "OpenAI" })).queryByText("check_circle"),
    ).not.toBeInTheDocument();
  });

  // Le fournisseur local sélectionne l'artefact adapté à la machine : le nommer d'après une
  // seule famille de modèles devient faux dès qu'une autre peut être retenue.
  it("nomme le fournisseur local « IA locale », jamais d'après un modèle", () => {
    render(<ProviderGrid value="mistral_local" onChange={() => undefined} />);

    expect(screen.getByRole("radio", { name: "IA locale" })).toBeInTheDocument();
    expect(screen.queryByText("Mistral Local")).not.toBeInTheDocument();
  });

  it("présente l'IA locale comme le choix recommandé sans confondre Ollama", () => {
    render(<ProviderGrid value="ollama" onChange={() => undefined} />);

    expect(within(screen.getByRole("radio", { name: "IA locale" })).getByText("Recommandé")).toBeInTheDocument();
    expect(within(screen.getByRole("radio", { name: "Ollama" })).getByText("Votre installation ou Ollama Cloud")).toBeInTheDocument();
  });
});
