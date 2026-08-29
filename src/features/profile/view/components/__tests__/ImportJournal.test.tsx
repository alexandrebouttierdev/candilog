import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { ImportJournal } from "../ImportJournal";

const entries = [
  { at: "2026-08-29T14:32:01.000Z", message: "Lecture du fichier" },
  { at: "2026-08-29T14:32:03.000Z", message: "Analyse du profil" },
];

describe("ImportJournal", () => {
  it("est fermé par défaut", () => {
    render(<ImportJournal entries={entries} />);
    expect(screen.queryByText("Lecture du fichier")).not.toBeInTheDocument();
  });

  it("s'ouvre et se referme", async () => {
    render(<ImportJournal entries={entries} />);
    await userEvent.click(screen.getByRole("button", { name: /Journal d'import/ }));
    expect(screen.getByText("Lecture du fichier")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /Journal d'import/ }));
    expect(screen.queryByText("Lecture du fichier")).not.toBeInTheDocument();
  });
});
