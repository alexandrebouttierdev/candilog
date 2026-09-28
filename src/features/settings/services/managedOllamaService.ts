import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { ipc } from "@/shared/services/ipc";
import type {
  InstallManagedModelRequest,
  LocalModelProbe,
  ManagedModelDefinition,
  ManagedModelId,
  ManagedOllamaDownloadProgress,
  ManagedOllamaStatus,
} from "@/shared/types/generated/ai";

const DOWNLOAD_PROGRESS_EVENT = "managed-ollama://download-progress";

export const MANAGED_OLLAMA_KEY = ["parametres", "managed-ollama"] as const;

export const managedOllamaService = {
  status: () => ipc<ManagedOllamaStatus>("get_managed_ollama_status"),
  install: (request: InstallManagedModelRequest) =>
    ipc<ManagedOllamaStatus>("install_managed_ollama_model", { request }),
  cancel: () => ipc<void>("cancel_managed_ollama_download"),
  remove: (modelId: ManagedModelId) =>
    ipc<ManagedOllamaStatus>("remove_managed_ollama_model", { model_id: modelId }),
  activate: (modelId: ManagedModelId) =>
    ipc<ManagedModelDefinition>("activate_managed_ollama_model", { model_id: modelId }),
  /** Phrase de test envoyée au modèle local actif ; renvoie l'aller-retour mesuré. */
  probe: () => ipc<LocalModelProbe>("probe_managed_ollama_model"),
  onProgress: (handler: (event: ManagedOllamaDownloadProgress) => void): Promise<UnlistenFn> =>
    listen<ManagedOllamaDownloadProgress>(DOWNLOAD_PROGRESS_EVENT, (event) =>
      handler(event.payload),
    ),
};
