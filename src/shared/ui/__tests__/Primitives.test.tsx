import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { StatusGlyph } from "../StatusGlyph";
import { Switch } from "../Switch";
import { avatarInitials, avatarTone } from "../Avatar";

describe("primitives v2", () => {
  it("donne un nom accessible au glyphe seul, et le cache quand un libellé l'accompagne", () => {
    const { container } = render(
      <>
        <StatusGlyph tone="g" label="Entretien" />
        <StatusGlyph tone="c" />
      </>,
    );
    expect(screen.getByRole("img", { name: "Entretien" })).toBeInTheDocument();
    expect(container.querySelectorAll('[aria-hidden="true"][data-tone="c"]')).toHaveLength(1);
  });

  it("bascule un interrupteur et annonce son état", async () => {
    const onChange = vi.fn();
    render(<Switch checked={false} onChange={onChange} label="Animations" />);
    const bouton = screen.getByRole("switch", { name: "Animations" });
    expect(bouton).toHaveAttribute("aria-checked", "false");
    await userEvent.click(bouton);
    expect(onChange).toHaveBeenCalledWith(true);
  });

  it("dérive des initiales et une couleur stables d'un nom", () => {
    expect(avatarInitials("Novéa Services")).toBe("NS");
    expect(avatarInitials("Linaïa")).toBe("LI");
    expect(avatarTone("Novéa Services")).toBe(avatarTone("Novéa Services"));
  });
});
