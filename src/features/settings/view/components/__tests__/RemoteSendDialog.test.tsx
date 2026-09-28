import { beforeEach, describe, expect, it, vi } from "vitest";
import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import type { AiTask, Settings } from "@/shared/types/generated/settings";
import { settingsService } from "../../../services/settingsService";
import { useRemoteSendGuard } from "../../../viewmodel/useRemoteSendGuard";
import { RemoteSendDialog } from "../RemoteSendDialog";
import { RemoteSendConsents } from "../RemoteSendConsents";

function reglages(overrides: Partial<Settings> = {}): Settings {
  return {
    llm: { provider: "candilog_local", api_key_configured: false, endpoint: null, model: "", temperature: 0.7, mode: "auto" },
    llm_presets: {},
    ai_routes: { analyze_resume: { provider: "claude", model: "claude-sonnet-5" } },
    remote_send_consents: [],
    theme: "system",
    language: "fr",
    ...overrides,
  };
}

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

/** Écran minimal : un bouton qui passe par la garde avant d'« envoyer ». */
function Envoi({ task }: { task: AiTask }) {
  const confirmSend = useRemoteSendGuard();
  const [issue, setIssue] = useState<string>("");
  return (
    <>
      <button type="button" onClick={() => void confirmSend(task, "Votre CV et le texte de l’offre").then((ok) => setIssue(ok ? "envoyé" : "retenu"))}>
        Lancer
      </button>
      <output>{issue}</output>
      <RemoteSendDialog />
    </>
  );
}

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("premier envoi à un service distant", () => {
  it("n'envoie rien si l'on annule", async () => {
    vi.spyOn(settingsService, "load").mockResolvedValue(reglages());
    const save = vi.spyOn(settingsService, "save");
    render(<Envoi task="analyze_resume" />, { wrapper });

    await userEvent.click(screen.getByRole("button", { name: "Lancer" }));
    const dialogue = await screen.findByRole("alertdialog", { name: "Cette tâche sera envoyée à Anthropic" });
    expect(dialogue).toHaveTextContent("1 sur 5");
    // « Ne plus demander » est désactivé par défaut.
    expect(screen.getByRole("switch", { name: "Ne plus demander pour Anthropic" })).toHaveAttribute("aria-checked", "false");
    await userEvent.click(screen.getByRole("button", { name: "Annuler" }));

    expect(await screen.findByText("retenu")).toBeInTheDocument();
    expect(save).not.toHaveBeenCalled();
  });

  it("envoie et mémorise le choix « Ne plus demander »", async () => {
    vi.spyOn(settingsService, "load").mockResolvedValue(reglages());
    const save = vi.spyOn(settingsService, "save").mockResolvedValue(reglages({ remote_send_consents: ["claude"] }));
    render(<Envoi task="analyze_resume" />, { wrapper });

    await userEvent.click(screen.getByRole("button", { name: "Lancer" }));
    await userEvent.click(await screen.findByRole("switch", { name: "Ne plus demander pour Anthropic" }));
    await userEvent.click(screen.getByRole("button", { name: "Envoyer" }));

    expect(await screen.findByText("envoyé")).toBeInTheDocument();
    expect(save).toHaveBeenCalledWith(expect.objectContaining({ remote_send_consents: ["claude"] }));
  });

  it("laisse passer une tâche locale sans rien demander", async () => {
    vi.spyOn(settingsService, "load").mockResolvedValue(reglages());
    render(<Envoi task="generate_resume" />, { wrapper });

    await userEvent.click(screen.getByRole("button", { name: "Lancer" }));

    expect(await screen.findByText("envoyé")).toBeInTheDocument();
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
  });

  it("rétablit la confirmation depuis les réglages", async () => {
    vi.spyOn(settingsService, "load").mockResolvedValue(reglages({ remote_send_consents: ["claude"] }));
    const save = vi.spyOn(settingsService, "save").mockResolvedValue(reglages());
    render(<RemoteSendConsents />, { wrapper });

    expect(await screen.findByText("Plus de confirmation pour : Claude.")).toBeInTheDocument();
    await act(async () => {
      await userEvent.click(screen.getByRole("button", { name: "Redemander" }));
    });

    await waitFor(() => expect(save).toHaveBeenCalledWith(expect.objectContaining({ remote_send_consents: [] })));
  });
});
