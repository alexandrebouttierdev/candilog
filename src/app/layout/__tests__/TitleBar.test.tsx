import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { ChromeProvider, useChrome } from "@/shared/lib/chrome";
import { CommandProvider, useRegisterCommands } from "@/shared/lib/commands";
import { TitleBar } from "../TitleBar";

function Ecran({ run }: { run: () => void }) {
  useRegisterCommands([{ id: "app-group", group: "view", label: "Grouper par entreprise", run }]);
  useChrome({ crumb: "Toutes", action: { label: "Grouper : statut ▾", command: "app-group" } });
  return null;
}

describe("barre de titre", () => {
  it("exécute la commande de l'action d'écran", async () => {
    const run = vi.fn();
    render(
      <MemoryRouter initialEntries={["/applications"]}>
        <CommandProvider>
          <ChromeProvider>
            <Ecran run={run} />
            <TitleBar />
          </ChromeProvider>
        </CommandProvider>
      </MemoryRouter>,
    );

    await userEvent.click(screen.getByRole("button", { name: "Grouper : statut ▾" }));

    expect(run).toHaveBeenCalledOnce();
  });
});
