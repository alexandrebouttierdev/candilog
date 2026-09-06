import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { ipc } from "@/shared/services/ipc";
import type {
  InstallLocalAiRequest,
  LocalAiBenchmark,
  LocalAiDownloadCompleted,
  LocalAiDownloadError,
  LocalAiDownloadProgress,
  LocalAiHardware,
  LocalAiRecommendation,
  LocalAiStatus,
  LocalModelId,
} from "@/shared/types/generated/ai";

const DOWNLOAD_PROGRESS_EVENT = "local-ai://download-progress";
const DOWNLOAD_COMPLETED_EVENT = "local-ai://download-completed";
const DOWNLOAD_ERROR_EVENT = "local-ai://download-error";

export const localAiService = {
  detectHardware: () => ipc<LocalAiHardware>("detect_local_ai_hardware"),
  recommendation: () => ipc<LocalAiRecommendation>("get_local_ai_recommendation"),
  status: () => ipc<LocalAiStatus>("get_local_ai_status"),
  install: (request: InstallLocalAiRequest) =>
    ipc<LocalAiStatus>("install_local_ai_model", { request }),
  cancel: () => ipc<void>("cancel_local_ai_download"),
  remove: (modelId: LocalModelId) =>
    ipc<LocalAiStatus>("remove_local_ai_model", { model_id: modelId }),
  benchmark: () => ipc<LocalAiBenchmark>("benchmark_local_ai_model"),
  test: () => ipc<string>("test_local_ai_model"),
  onProgress: (handler: (event: LocalAiDownloadProgress) => void): Promise<UnlistenFn> =>
    listen<LocalAiDownloadProgress>(DOWNLOAD_PROGRESS_EVENT, (event) => handler(event.payload)),
  onCompleted: (handler: (event: LocalAiDownloadCompleted) => void): Promise<UnlistenFn> =>
    listen<LocalAiDownloadCompleted>(DOWNLOAD_COMPLETED_EVENT, (event) => handler(event.payload)),
  onError: (handler: (event: LocalAiDownloadError) => void): Promise<UnlistenFn> =>
    listen<LocalAiDownloadError>(DOWNLOAD_ERROR_EVENT, (event) => handler(event.payload)),
};
