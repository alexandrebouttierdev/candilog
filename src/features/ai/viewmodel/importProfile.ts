import { aiService } from "../services/aiService";

/** Démarre l'import de profil depuis un CV — seul le viewmodel parle au service. */
export function importProfileFromResume(generationId: string) {
  return aiService.importProfile({ generation_id: generationId });
}

export function runUserBenchmark(generationId: string) {
  return aiService.runUserBenchmark(generationId);
}

export function cancelAiOperation(generationId: string) {
  return aiService.cancel(generationId);
}
