import { ipc } from "@/shared/services/ipc";
import type {
  ConflictChoice,
  ProviderAvailability,
  RemoteProbe,
  SetupOutcome,
  SetupRequest,
  SyncConflict,
  SyncProviderKind,
  SyncStatus,
} from "@/shared/types/generated/sync";

export type * from "@/shared/types/generated/sync";

/** Frontière IPC de la synchronisation entre ordinateurs. */
export const syncService = {
  status: () => ipc<SyncStatus>("sync_status"),
  availability: () => ipc<ProviderAvailability>("sync_availability"),
  // Le dossier est choisi dans le dialogue natif côté Rust : aucun chemin ne part d'ici.
  chooseFolder: () => ipc<RemoteProbe | null>("sync_choose_folder"),
  connect: (provider: SyncProviderKind) => ipc<RemoteProbe>("sync_connect", { provider }),
  cancelConnect: () => ipc<void>("sync_cancel_connect"),
  setup: (request: SetupRequest) => ipc<SetupOutcome>("sync_setup", { request }),
  unlock: (passphrase: string | null, recovery_key: string | null) =>
    ipc<void>("sync_unlock", { passphrase, recovery_key }),
  setAuto: (enabled: boolean) => ipc<void>("sync_set_auto", { enabled }),
  now: () => ipc<void>("sync_now"),
  disconnect: () => ipc<void>("sync_disconnect"),
  restoreFromSpace: () => ipc<string>("sync_restore_from_space"),
  conflicts: () => ipc<SyncConflict[]>("sync_conflicts"),
  resolve: (conflict_id: string, choice: ConflictChoice) =>
    ipc<void>("sync_resolve_conflict", { conflict_id, choice }),
  resolveAll: (choice: ConflictChoice) => ipc<number>("sync_resolve_all", { choice }),
};
