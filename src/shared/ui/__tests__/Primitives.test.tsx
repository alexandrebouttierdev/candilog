import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { StatusGlyph } from "../StatusGlyph";
import { Switch } from "../Switch";
import { RunMeter, StepList } from "../WorkSurface";
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

describe("avancement d'un traitement", () => {
  it("affiche le temps écoulé et les tokens rapportés", () => {
    render(
      <RunMeter
        steps={[
          { label: "Lecture de l'offre", state: "done", ms: 2100 },
          { label: "Rédaction", state: "running", ms: null },
        ]}
        elapsedMs={6_000}
        tokens={1272}
      />,
    );

    const avancement = screen.getByRole("status", { name: "Avancement" });
    expect(avancement).toHaveTextContent("écoulé 00:06");
    expect(avancement).toHaveTextContent(/1\s272 tokens/);
  });
});

describe("déroulé des étapes", () => {
  it("affiche les durées en secondes entières", () => {
    render(
      <StepList
        steps={[
          { label: "Rédaction", state: "done", ms: 3000 },
          { label: "Relecture", state: "done", ms: 400 },
        ]}
      />,
    );

    expect(screen.getByText("3 s")).toBeInTheDocument();
    expect(screen.getByText("< 1 s")).toBeInTheDocument();
  });
});
