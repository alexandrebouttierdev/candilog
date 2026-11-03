import { AiBenchmarkModal } from "@/features/ai";
import { AiPage } from "@/features/settings/view/pages/AiPage";

/**
 * Écran Intelligence artificielle, avec la surcouche du benchmark. La couche `app` fait le
 * lien : `settings` ne dépend pas de `ai`, qui dépend de `settings` (configuration IA).
 */
export function AiRoute() {
  return <AiPage renderBenchmark={(props) => <AiBenchmarkModal {...props} />} />;
}
