import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryWrapper } from "@/shared/lib/test-utils";
import type { SyncStatus } from "@/shared/types/generated/sync";
import { syncService } from "../../services/syncService";
import { SyncSettings } from "../SyncSettings";

vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(() => Promise.resolve(() => {})) }));

const local: SyncStatus = {
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

beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(syncService, "availability").mockResolvedValue({ google_drive: false, one_drive: true });
  vi.spyOn(syncService, "conflicts").mockResolvedValue([]);
});

describe("section Synchronisation et sauvegarde", () => {
  it("présente le stockage local comme mode actif et les fournisseurs au choix", async () => {
    vi.spyOn(syncService, "status").mockResolvedValue(local);
    render(<SyncSettings />, { wrapper: QueryWrapper });

    expect(await screen.findByText("Stockage local uniquement")).toBeInTheDocument();
    expect(screen.getByText("Actif")).toBeInTheDocument();
    const buttons = screen.getAllByRole("button", { name: "Connecter…" });
    expect(buttons[0]).toBeDisabled();
    expect(buttons[1]).toBeEnabled();
    expect(screen.getByRole("button", { name: "Choisir un dossier…" })).toBeEnabled();
  });

  it("propose de créer l'espace quand le dossier choisi est vide", async () => {
    vi.spyOn(syncService, "status").mockResolvedValue(local);
    vi.spyOn(syncService, "chooseFolder").mockResolvedValue({
      provider: "folder",
      location: "/Users/camille/Dropbox",
      space_exists: false,
      legacy_space_exists: true,
      space_created_at: null,
      local_has_data: true,
    });
    render(<SyncSettings />, { wrapper: QueryWrapper });
    await userEvent.click(await screen.findByRole("button", { name: "Choisir un dossier…" }));

    expect(await screen.findByText("Créer l'espace de synchronisation")).toBeInTheDocument();
    expect(screen.getByText(/envoyées sans chiffrement/)).toBeInTheDocument();
    expect(screen.getByText(/Un ancien espace chiffré existe ici/)).toBeInTheDocument();
  });

  it("exige un scénario explicite pour rejoindre un espace depuis un ordinateur qui a des données", async () => {
    vi.spyOn(syncService, "status").mockResolvedValue(local);
    vi.spyOn(syncService, "chooseFolder").mockResolvedValue({
      provider: "folder",
      location: "/Users/camille/Dropbox",
      space_exists: true,
      legacy_space_exists: false,
      space_created_at: "2026-10-01T10:00:00Z",
      local_has_data: true,
    });
    const setup = vi.spyOn(syncService, "setup").mockResolvedValue({ backup_file: null });
    render(<SyncSettings />, { wrapper: QueryWrapper });
    await userEvent.click(await screen.findByRole("button", { name: "Choisir un dossier…" }));
    await userEvent.click(await screen.findByRole("radio", { name: /Fusionner avec les données/ }));

    expect(await screen.findByText("Cet ordinateur contient déjà des données")).toBeInTheDocument();
    expect(setup).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: "Rejoindre" }));
    expect(setup).toHaveBeenCalledWith({ choice: "join_merge" });
  });

  it("explique l'état et confirme la désactivation sans effacer les données", async () => {
    vi.spyOn(syncService, "status").mockResolvedValue({
      ...local,
      provider: "folder",
      phase: "offline",
      auto_sync: true,
      location: "/Users/camille/Dropbox",
      pending_changes: 2,
      message: "Pas de connexion : vos modifications sont conservées.",
    });
    const disconnect = vi.spyOn(syncService, "disconnect").mockResolvedValue(undefined);
    render(<SyncSettings />, { wrapper: QueryWrapper });

    expect(await screen.findByText("Hors ligne")).toBeInTheDocument();
    expect(screen.getByText("2 modifications")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Désactiver…" }));
    expect(screen.getByText("Vos données restent sur cet ordinateur")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /^Désactiver$/ }));
    expect(disconnect).toHaveBeenCalledOnce();
  });
});
