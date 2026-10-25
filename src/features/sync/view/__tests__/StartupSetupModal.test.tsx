import { beforeEach, describe, expect, it, vi } from "vitest";
import { act, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { QueryWrapper } from "@/shared/lib/test-utils";
import { useUiStore } from "@/shared/lib/ui-store";
import type { SyncStatus } from "@/shared/types/generated/sync";
import { syncService } from "../../services/syncService";
import { StartupSetupModal } from "../StartupSetupModal";

const localStatus: SyncStatus = {
  provider: "local",
  phase: "not_configured",
  auto_sync: false,
  location: null,
  pending_changes: 0,
  conflicts: 0,
  waiting_changes: 0,
  missing_files: 0,
  computers: 0,
  last_success_at: null,
  last_attempt_at: null,
  next_attempt_at: null,
  message: null,
};

function renderStartupSetup() {
  const router = createMemoryRouter(
    [
      {
        path: "/",
        element: (
          <>
            <StartupSetupModal />
            <p>Écran Aujourd’hui</p>
          </>
        ),
      },
      { path: "/ai", element: <p>Écran IA</p> },
    ],
    { initialEntries: ["/"] },
  );
  render(
    <QueryWrapper>
      <RouterProvider router={router} />
    </QueryWrapper>,
  );
  return router;
}

beforeEach(() => {
  vi.restoreAllMocks();
  window.localStorage.clear();
  useUiStore.setState({ settings: null });
  vi.spyOn(syncService, "status").mockResolvedValue(localStatus);
});

describe("parcours de configuration initiale", () => {
  it("mémorise le stockage local et propose l'IA sans imposer sa configuration", async () => {
    renderStartupSetup();
    const dialog = await screen.findByRole("dialog", { name: "Bienvenue dans Candilog" });

    await userEvent.click(screen.getByRole("radio", { name: /Sur cet ordinateur/ }));
    await userEvent.click(screen.getByRole("button", { name: "Continuer" }));

    const aiDialog = await screen.findByRole("dialog", {
      name: "Un assistant IA, quand vous le souhaitez",
    });
    expect(window.localStorage.getItem("candilog.startup-setup")).toBe(
      JSON.stringify({ storage: "local", aiPromptDismissed: false }),
    );
    expect(within(aiDialog).getByRole("button", { name: "Configurer l’IA maintenant" })).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Plus tard" }));

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(screen.getByText("Écran Aujourd’hui")).toBeInTheDocument();
    expect(window.localStorage.getItem("candilog.startup-setup")).toBe(
      JSON.stringify({ storage: "local", aiPromptDismissed: true }),
    );
    expect(dialog).not.toBeInTheDocument();
  });

  it("ne mémorise le cloud qu'après sa configuration et ouvre ensuite la page IA", async () => {
    let status = localStatus;
    vi.spyOn(syncService, "status").mockImplementation(() => Promise.resolve(status));
    const router = renderStartupSetup();

    await screen.findByRole("dialog", { name: "Bienvenue dans Candilog" });
    await userEvent.click(screen.getByRole("radio", { name: /Sur plusieurs appareils/ }));
    await userEvent.click(screen.getByRole("button", { name: "Configurer le cloud" }));

    expect(useUiStore.getState().settings).toBe("data");
    expect(window.localStorage.getItem("candilog.startup-setup")).toBeNull();

    status = { ...localStatus, provider: "folder", phase: "up_to_date" };
    act(() => useUiStore.getState().closeSettings());

    await screen.findByRole("dialog", { name: "Un assistant IA, quand vous le souhaitez" });
    expect(window.localStorage.getItem("candilog.startup-setup")).toBe(
      JSON.stringify({ storage: "cloud", aiPromptDismissed: false }),
    );

    await userEvent.click(screen.getByRole("button", { name: "Configurer l’IA maintenant" }));

    expect(await screen.findByText("Écran IA")).toBeInTheDocument();
    expect(router.state.location.pathname).toBe("/ai");
    expect(window.localStorage.getItem("candilog.startup-setup")).toBe(
      JSON.stringify({ storage: "cloud", aiPromptDismissed: true }),
    );
  });

  it("reprend directement l'étape IA si un espace cloud est déjà configuré", async () => {
    vi.spyOn(syncService, "status").mockResolvedValue({
      ...localStatus,
      provider: "one_drive",
      phase: "up_to_date",
    });

    renderStartupSetup();

    expect(
      await screen.findByRole("dialog", { name: "Un assistant IA, quand vous le souhaitez" }),
    ).toBeInTheDocument();
    expect(window.localStorage.getItem("candilog.startup-setup")).toBe(
      JSON.stringify({ storage: "cloud", aiPromptDismissed: false }),
    );
  });
});
