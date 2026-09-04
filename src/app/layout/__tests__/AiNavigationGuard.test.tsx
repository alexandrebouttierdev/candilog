import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { Link, Outlet, RouterProvider, createMemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useAiOperationStore, type AiOperationKind } from "@/features/ai/viewmodel/ai-operation-store";
import { useUiStore } from "@/shared/lib/ui-store";
import { AiNavigationGuard } from "../AiNavigationGuard";

function renderGuard(stop: () => Promise<void>, kind: AiOperationKind = "generation") {
  useAiOperationStore.getState().begin({ id: "gen-1", kind, stop });
  const router = createMemoryRouter([
    {
      path: "/",
      element: <><AiNavigationGuard /><Outlet /></>,
      children: [
        { index: true, element: <><p>Écran actif</p><Link to="/next">Suivant</Link></> },
        { path: "next", element: <><p>Écran suivant</p><Link to="/final">Continuer</Link></> },
        { path: "final", element: <p>Écran final</p> },
      ],
    },
  ]);
  render(<RouterProvider router={router} />);
  fireEvent.click(screen.getByRole("link", { name: "Suivant" }));
  return router;
}

describe("AiNavigationGuard", () => {
  beforeEach(() => {
    useAiOperationStore.setState({ active: null });
    useUiStore.setState({ toasts: [] });
  });

  it("annule la navigation et conserve l'opération", () => {
    const stop = vi.fn().mockResolvedValue(undefined);
    const router = renderGuard(stop, "analyse");

    expect(screen.getByRole("alertdialog", { name: "Quitter cet écran ?" }))
      .toHaveTextContent("L’analyse en cours sera arrêtée");
    fireEvent.click(screen.getByRole("button", { name: "Annuler" }));

    expect(router.state.location.pathname).toBe("/");
    expect(stop).not.toHaveBeenCalled();
    expect(useAiOperationStore.getState().active?.id).toBe("gen-1");
  });

  it("attend l'arrêt avant de poursuivre la navigation", async () => {
    let resolveStop: (() => void) | undefined;
    const stop = vi.fn(() => new Promise<void>((resolve) => { resolveStop = resolve; }));
    const router = renderGuard(stop);

    fireEvent.click(screen.getByRole("button", { name: "Quitter et arrêter" }));
    expect(stop).toHaveBeenCalledOnce();
    expect(router.state.location.pathname).toBe("/");
    expect(screen.getByRole("button", { name: "Quitter et arrêter" })).toBeDisabled();

    act(() => { resolveStop?.(); });
    await waitFor(() => expect(router.state.location.pathname).toBe("/next"));
    expect(screen.getByText("Écran suivant")).toBeInTheDocument();
  });

  it("interdit Annuler et Échap pendant l'arrêt", async () => {
    let resolveStop: (() => void) | undefined;
    const stop = vi.fn(() => new Promise<void>((resolve) => { resolveStop = resolve; }));
    const router = renderGuard(stop);

    fireEvent.click(screen.getByRole("button", { name: "Quitter et arrêter" }));
    const cancel = screen.getByRole("button", { name: "Annuler" });
    expect(cancel).toBeDisabled();

    fireEvent.click(cancel);
    fireEvent.keyDown(document, { key: "Escape" });

    expect(router.state.location.pathname).toBe("/");
    expect(screen.getByRole("alertdialog", { name: "Quitter cet écran ?" }))
      .toBeInTheDocument();

    act(() => { resolveStop?.(); });
    await waitFor(() => expect(router.state.location.pathname).toBe("/next"));
  });

  it("poursuit la navigation si l'opération se termine naturellement", async () => {
    const stop = vi.fn().mockResolvedValue(undefined);
    const router = renderGuard(stop);

    act(() => { useAiOperationStore.getState().finish("gen-1"); });

    await waitFor(() => expect(router.state.location.pathname).toBe("/next"));
    expect(stop).not.toHaveBeenCalled();
  });

  it("reste sur place et notifie quand l'arrêt échoue", async () => {
    const stop = vi.fn().mockRejectedValue(new Error("indisponible"));
    const router = renderGuard(stop, "import");

    fireEvent.click(screen.getByRole("button", { name: "Quitter et arrêter" }));

    await waitFor(() => {
      expect(useUiStore.getState().toasts).toContainEqual(expect.objectContaining({
        tone: "error",
        title: "Arrêt impossible",
      }));
    });
    expect(router.state.location.pathname).toBe("/");
    expect(screen.getByRole("alertdialog", { name: "Quitter cet écran ?" }))
      .toHaveTextContent("L’import en cours sera arrêté");
  });

  it("garde deux cycles successifs d'arrêt et de navigation", async () => {
    const firstStop = vi.fn().mockResolvedValue(undefined);
    const router = renderGuard(firstStop);

    fireEvent.click(screen.getByRole("button", { name: "Quitter et arrêter" }));
    await waitFor(() => expect(router.state.location.pathname).toBe("/next"));

    const secondStop = vi.fn().mockResolvedValue(undefined);
    act(() => {
      useAiOperationStore.getState().begin({
        id: "gen-2",
        kind: "analyse",
        stop: secondStop,
      });
    });
    fireEvent.click(screen.getByRole("link", { name: "Continuer" }));
    fireEvent.click(screen.getByRole("button", { name: "Quitter et arrêter" }));

    await waitFor(() => expect(router.state.location.pathname).toBe("/final"));
    expect(firstStop).toHaveBeenCalledOnce();
    expect(secondStop).toHaveBeenCalledOnce();
  });
});
