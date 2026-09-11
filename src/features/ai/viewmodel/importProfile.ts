import { aiService } from "../services/aiService";
import type { CvAnalysisMethod } from "../model/types";

/** Démarre l'import de profil depuis un CV — seul le viewmodel parle au service. */
export function importProfileFromResume(
  generationId: string,
  method: CvAnalysisMethod = "vision",
) {
  return aiService.importProfile({ generation_id: generationId, method });
}

export function runUserBenchmark(
  generationId: string,
  method: CvAnalysisMethod = "vision",
) {
  return aiService.runUserBenchmark({ generation_id: generationId, method });
}

export function fetchActiveModelCapabilities() {
  return aiService.activeModelCapabilities();
}

export function cancelAiOperation(generationId: string) {
  return aiService.cancel(generationId);
}
