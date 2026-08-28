import { describe, expect, it, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ProviderGrid } from "../ProviderGrid";
import { FOURNISSEURS } from "../../../model/providers";

describe("grille des fournisseurs", () => {
  it("propose les sept fournisseurs comme un groupe de boutons radio", () => {
    render(<ProviderGrid value="ollama" onChange={() => undefined} />);
    expect(screen.getByRole("radiogroup", { name: "Fournisseur IA" })).toBeInTheDocument();
    expect(screen.getAllByRole("radio")).toHaveLength(FOURNISSEURS.length);
    expect(screen.getByRole("radio", { name: /Ollama/ })).toHaveAttribute("aria-checked", "true");
    expect(screen.getByRole("radio", { name: "OpenAI" })).toHaveAttribute("aria-checked", "false");
    for (const fournisseur of FOURNISSEURS) {
      const radio = screen.getByRole("radio", { name: fournisseur.label });
      expect(within(radio).getByRole("img", { name: fournisseur.label })).toBeInTheDocument();
    }
  });

  it("signale le fournisseur choisi et notifie le changement", async () => {
    const onChange = vi.fn();
    render(<ProviderGrid value="ollama" onChange={onChange} />);
    await userEvent.click(screen.getByRole("radio", { name: /Claude/ }));
    expect(onChange).toHaveBeenCalledWith("claude");
  });
});
